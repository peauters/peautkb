#![no_main]
#![no_std]

use cortex_m;
use hal::prelude::*;
use stm32f4xx_hal as hal;

use rtic::app;

use defmt_rtt as _; // global logger
use panic_probe as _;

use nb::block;

use core::fmt::Write;
use ssd1306::{mode::TerminalMode, prelude::*, Builder, I2CDIBuilder};

use crate::hal::{
    prelude::*,
    gpio::{
        gpioa::{PA0, PA1, PA5, PA6, PA7},
        gpiob::{PB7, PB8, PB9},
        gpioc::{PC13, PC14},
        Alternate, AlternateOD, Edge, ExtiPin, Input, Output, PullDown, PullUp, PushPull, AF4, AF5,
    },
    i2c::I2c,
    interrupt, otg_fs, serial,
    spi::*,
    stm32, timer,
};

#[app(device = crate::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
        display: TerminalMode<
            I2CInterface<I2c<stm32::I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>,
            DisplaySize128x64,
        >,
        my_count: u8,
        a0: PA0<Input<PullUp>>,
        display_initd : bool,
        display_init : timer::Timer<stm32::TIM2>,
    }

    #[init(spawn = [disp])]
    fn init(c: init::Context) -> init::LateResources {
        let mut perfs: stm32::Peripherals = c.device;

        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).pclk1(24.mhz()).freeze();

        let gpioa = perfs.GPIOA.split();

        let mut a0 = gpioa.pa0.into_pull_up_input();
        a0.enable_interrupt(&mut perfs.EXTI);
        a0.make_interrupt_source(&mut perfs.SYSCFG);
        a0.trigger_on_edge(&mut perfs.EXTI, Edge::FALLING);

        let gpiob = perfs.GPIOB.split();

        let mut timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let mut display_init = timer::Timer::tim2(perfs.TIM2, 5.hz(), clocks);
        display_init.listen(timer::Event::TimeOut);

        let scl = gpiob
            .pb8
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiob
            .pb9
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let i2c = I2c::i2c1(perfs.I2C1, (scl, sda), 100.khz(), clocks);
        let interface = I2CDIBuilder::new().init(i2c);
        let display: TerminalMode<_, _> = Builder::new().connect(interface).into();

        defmt::info!("I've init'd");

        init::LateResources {
            timer,
            display,
            my_count: 0,
            a0,
            display_initd : false,
            display_init,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(binds = EXTI0, priority = 1, resources = [a0, my_count], spawn = [disp])]
    fn button_press(mut c: button_press::Context) {
        c.resources.a0.clear_interrupt_pending_bit();
        
        c.resources.my_count.lock(|i| *i = 0);        

        c.spawn.disp().unwrap();
    }

            
    #[task(priority = 2, resources = [display, my_count, display_initd])]
    fn disp(c: disp::Context) {
        let disp::Resources {
            display,
            mut my_count,
            mut display_initd,
        } = c.resources;

        if display_initd.lock(|b| *b) {
            display.clear().unwrap();
            write!(display, "My count: {}\n", my_count.lock(|i| *i),).unwrap();
        }
        // write!(display, "My count: {}\n", my_count,).unwrap();
    }

    #[task(binds = TIM3,
            priority = 3,
            resources = [timer, my_count, display_initd], spawn = [disp])]
    fn tick(c: tick::Context) {
        let tick::Resources {
            timer,
            my_count,
            display_initd,
        } = c.resources;

        timer.wait().ok();

        *my_count += 1;
        c.spawn.disp().unwrap();
    }

    #[task(binds = TIM2, priority = 2, resources = [display, display_init, display_initd])]
    fn init_display(mut c: init_display::Context)
    {
        static mut COUNT : u8 = 1;

        if *COUNT == 0 {
            defmt::info!("Initializing display");
            let mut display = c.resources.display;
            display.init().unwrap();
            display.flush().unwrap();
            display.clear().unwrap();
            display.flush().unwrap();
            defmt::info!("Display init'd");
            
            c.resources.display_initd.lock(|b| *b = true);
            c.resources.display_init.unlisten(timer::Event::TimeOut);    
        }
        else
        {
            *COUNT -= 1;
            c.resources.display_init.wait().ok();
        }
    }

    extern "C" {
        fn DMA2_STREAM0();
    }
};
