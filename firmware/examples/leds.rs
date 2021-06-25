#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [TIM2])]
mod app {

    use dwt_systick_monotonic::DwtSystick;

    use crate::hal::{prelude::*, stm32};
    use rtic::time::duration::Milliseconds;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    use crate::hal::{
        gpio::{gpiob::PB13, gpiob::PB15, Alternate, AF5},
        spi::{NoMiso, Spi},
        stm32::SPI2,
    };
    use smart_leds::{SmartLedsWrite, RGB8};
    use ws2812_spi::Ws2812;

    #[resources]
    struct Resources {
        leds: Ws2812<Spi<SPI2, (PB13<Alternate<AF5>>, NoMiso, PB15<Alternate<AF5>>)>>,
        i: u8,
    }

    #[init]
    fn init(mut c: init::Context) -> (init::LateResources, init::Monotonics) {
        let perfs: stm32::Peripherals = c.device;

        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).pclk1(48.mhz()).freeze();

        let mono = DwtSystick::new(&mut c.core.DCB, c.core.DWT, c.core.SYST, clocks.hclk().0);
        let gpiob = perfs.GPIOB.split();
        let pb15 = gpiob.pb15.into_alternate_af5().internal_pull_up(true);
        let pb13 = gpiob.pb13.into_alternate_af5();

        let spi = Spi::spi2(
            perfs.SPI2,
            (pb13, NoMiso, pb15),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );

        let leds = Ws2812::new(spi);

        tick::spawn_after(Milliseconds::new(1000_u32)).ok();

        (init::LateResources { leds, i: 0 }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(resources = [leds, i])]
    fn tick(mut c: tick::Context) {
        tick::spawn_after(Milliseconds::new(10_u32)).ok();

        fn wheel(mut wheel_pos: u8) -> RGB8 {
            wheel_pos = 255 - wheel_pos;
            if wheel_pos < 85 {
                return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
            }
            if wheel_pos < 170 {
                wheel_pos -= 85;
                return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
            }
            wheel_pos -= 170;
            (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
        }

        let j = c.resources.i.lock(|i| *i);

        // let all_off: [RGB8; 32] = [wheel(j); 32];
        let all_off: [RGB8; 32] = [(128, 0, 0).into(); 32];

        c.resources
            .leds
            .lock(|l| l.write(all_off.iter().cloned()).unwrap());

        c.resources
            .i
            .lock(|i| if *i < 255 { *i += 1 } else { *i = 0 })
    }
}
