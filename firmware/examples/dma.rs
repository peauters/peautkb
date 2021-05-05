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
        let stream = steams.3;

        static mut BUFFER: [u8; 128] = [0; 128];

        let trans = dma::Transfer::init(
            stream,
            perfs.TIM2.ccr2,
            BUFFER,
            None,
            dma::config::DmaConfig::default(),
        );

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
    fn tick(c: tick::Context) {
        defmt::info!("Tick");
        tick::spawn_after(Milliseconds::new(1000_u32)).ok();
    }
}
