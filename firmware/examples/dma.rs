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
        dma, pac,
        prelude::*,
        pwm,
        spi::{NoMiso, Spi},
        stm32,
    };
    use rtic::time::duration::Milliseconds;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    #[resources]
    struct Resources {}

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

        let steams = dma::StreamsTuple::new(perfs.DMA1);
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

        const ARRAY_SIZE: usize = 100;

        let buffer = cortex_m::singleton!(: [u8; ARRAY_SIZE] = [1; ARRAY_SIZE]).unwrap();

        for i in (0..ARRAY_SIZE) {
            buffer[i] = i as u8;
        }

        defmt::info!("buffer is {}", buffer);

        let (db_ptr, db_len) = unsafe { buffer.read_buffer() };

        defmt::info!("buffer is {}", db_len);

        let tx = spi2.use_dma().tx();

        let mut trans = dma::Transfer::init_memory_to_peripheral(
            stream,
            tx,
            buffer,
            None,
            dma::config::DmaConfig::default().fifo_enable(true),
        );

        trans.start(|tx| {
            defmt::info!("Transfer Starting");
        });

        tick::spawn_after(Milliseconds::new(1000_u32)).ok();

        (init::LateResources {}, init::Monotonics(mono))
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
