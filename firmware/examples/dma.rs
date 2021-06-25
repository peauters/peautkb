#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [TIM2])]
mod app {

    use dwt_systick_monotonic::DwtSystick;
    use embedded_dma::ReadBuffer;
    use stm32f4xx_hal::dma::Channel0;

    use crate::hal::{
        dma::traits::Stream,
        dma::*,
        pac,
        prelude::*,
        pwm,
        spi::{NoMiso, Spi, Tx},
        stm32,
    };
    use rtic::{export::lock, time::duration::Milliseconds};

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    const ARRAY_SIZE: usize = 512;

    #[resources]
    struct Resources {
        transfer: Transfer<
            Stream4<stm32::DMA1>,
            Channel0,
            Tx<stm32::SPI2>,
            MemoryToPeripheral,
            &'static mut [u8; 512],
        >,
        next_buffer: Option<&'static mut [u8; 512]>,
        i: u8,
    }

    #[init]
    fn init(mut c: init::Context) -> (init::LateResources, init::Monotonics) {
        let perfs: stm32::Peripherals = c.device;

        let rcc = perfs.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .use_hse(25.mhz())
            .freeze();

        let mono = DwtSystick::new(&mut c.core.DCB, c.core.DWT, c.core.SYST, clocks.hclk().0);

        let steams = StreamsTuple::new(perfs.DMA1);
        let stream = steams.4;

        let gpiob = perfs.GPIOB.split();
        let pb15 = gpiob.pb15.into_alternate_af5().internal_pull_up(true);
        let pb13 = gpiob.pb13.into_alternate_af5();

        let spi2 = Spi::spi2(
            perfs.SPI2,
            (pb13, NoMiso, pb15),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );

        let buffer = cortex_m::singleton!(: [u8; ARRAY_SIZE] = [0; ARRAY_SIZE]).unwrap();
        let next_buffer = cortex_m::singleton!(: [u8; ARRAY_SIZE] = [0; ARRAY_SIZE]).unwrap();

        let tx = spi2.use_dma().tx();

        let mut transfer = Transfer::init_memory_to_peripheral(
            stream,
            tx,
            buffer,
            None,
            config::DmaConfig::default()
                .memory_increment(true)
                // .fifo_enable(true)
                // .fifo_threshold(config::FifoThreshold::ThreeQuarterFull)
                .fifo_error_interrupt(true)
                // .transfer_error_interrupt(true),
                .transfer_complete_interrupt(true),
        );

        transfer.start(|_tx| {
            defmt::info!("Transfer Starting");
        });

        tick::spawn_after(Milliseconds::new(1000_u32)).ok();

        (
            init::LateResources {
                transfer,
                next_buffer: Some(next_buffer),
                i: 0,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = DMA1_STREAM4, resources = [transfer, next_buffer, i])]
    fn dmaint(ctx: dmaint::Context) {
        let dmaint::Resources {
            mut transfer,
            mut next_buffer,
            mut i,
        } = ctx.resources;

        let j = i.lock(|i| *i);
        transfer.lock(|transfer| {
            if Stream4::<stm32::DMA1>::get_fifo_error_flag() {
                transfer.clear_fifo_error_interrupt();
            }
            if Stream4::<stm32::DMA1>::get_transfer_complete_flag() {
                fn wheel(mut wheel_pos: u8) -> (u8, u8, u8) {
                    wheel_pos = 255 - wheel_pos;
                    if wheel_pos < 85 {
                        return (255 - wheel_pos * 3, 0, wheel_pos * 3);
                    }
                    if wheel_pos < 170 {
                        wheel_pos -= 85;
                        return (0, wheel_pos * 3, 255 - wheel_pos * 3);
                    }
                    wheel_pos -= 170;
                    (wheel_pos * 3, 255 - wheel_pos * 3, 0)
                }

                transfer.clear_transfer_complete_interrupt();
                let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
                next_buffer.lock(|b| {
                    let next = b.take().unwrap();

                    let mut index = 0;
                    for _ in 0..32 {
                        let (mut r, mut g, mut b) = wheel(j);

                        for _ in 0..4 {
                            let bits = (g & 0b1100_0000) >> 6;
                            next[index] = patterns[bits as usize];
                            index += 1;
                            g <<= 2;
                        }

                        for _ in 0..4 {
                            let bits = (r & 0b1100_0000) >> 6;
                            next[index] = patterns[bits as usize];
                            index += 1;
                            r <<= 2;
                        }

                        for _ in 0..4 {
                            let bits = (b & 0b1100_0000) >> 6;
                            next[index] = patterns[bits as usize];
                            index += 1;
                            b <<= 2;
                        }
                    }

                    let (old, _) = transfer.next_transfer(next).unwrap();
                    *b = Some(old);
                });
            }
        });
        i.lock(|i| *i = if *i < 255 { *i + 1 } else { 0 });
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task()]
    fn tick(_c: tick::Context) {
        // defmt::info!("Tick");
        tick::spawn_after(Milliseconds::new(1000_u32)).ok();
    }
}
