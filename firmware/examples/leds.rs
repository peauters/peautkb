#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [TIM2])]
mod app {

    use dwt_systick_monotonic::DwtSystick;

    use crate::hal::{dma, prelude::*, pwm, stm32};
    use rtic::time::duration::Milliseconds;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    use crate::hal::{
        gpio::{gpiob::PB15, Alternate, AF5},
        prelude::*,
        rcc::Clocks,
        spi::{NoMiso, NoSck, Spi},
        stm32::SPI2,
    };
    use smart_leds::{SmartLedsWrite, RGB8};
    use ws2812_spi::Ws2812;

    #[resources]
    struct Resources {
        leds: Ws2812<Spi<SPI2, (NoSck, NoMiso, PB15<Alternate<AF5>>)>>,
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
        let gpiob = perfs.GPIOB.split();
        let pb15 = gpiob.pb15.into_alternate_af5();

        let spi = Spi::spi2(
            perfs.SPI2,
            (NoSck, NoMiso, pb15),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );

        let leds = Ws2812::new(spi);

        tick::spawn_after(Milliseconds::new(1000_u32)).ok();

        (init::LateResources { leds }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(resources = [leds])]
    fn tick(mut c: tick::Context) {
        defmt::info!("Tick");
        tick::spawn_after(Milliseconds::new(1000_u32)).ok();

        fn initial() -> RGB8 {
            RGB8 { r: 255, g: 0, b: 0 }
        }

        let all_blue: [RGB8; 31] = [initial(); 31];
        defmt::info!("writing to leds");
        c.resources
            .leds
            .lock(|l| l.write(all_blue.iter().cloned()).unwrap());
        defmt::info!("done");
    }
}
