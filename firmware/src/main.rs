#![no_main]
#![no_std]

use cortex_m;
use stm32f4xx_hal as hal;
use hal::prelude::*;
use rtic::app;

use defmt_rtt as _; // global logger
use panic_probe as _;

use crate::hal::{prelude::*, stm32, timer};

#[app(device = crate::hal::stm32, peripherals = true)]
const APP : () = 
{

    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
    }

    #[init]
    fn init(mut c : init::Context) -> init::LateResources {

        let mut perfs : stm32::Peripherals = c.device;
        
        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let mut timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);
        timer.listen(timer::Event::TimeOut);

        defmt::info!("I've init'd");

        init::LateResources {
            timer,
        }

    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }


    #[task(binds = TIM3,
            priority = 2,
            resources = [timer])]
    fn tick(c : tick::Context) {

        c.resources.timer.wait().ok();

        defmt::info!("a tick happend");

    }


};
