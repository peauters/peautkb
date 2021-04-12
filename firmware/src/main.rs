#![no_main]
#![no_std]

use cortex_m;
use stm32f4xx_hal as hal;
use hal::prelude::*;

use rtic::app;

use defmt_rtt as _; // global logger
use panic_probe as _;

use nb::block;

use core::fmt::Write;
use ssd1306::{prelude::*, Builder, I2CDIBuilder, mode::TerminalMode};

use crate::hal::{
                stm32, 
                timer,
                serial,
                interrupt,
                otg_fs,
                spi::*,
                i2c::I2c,
                gpio::{
                    gpioa::{PA5, PA6, PA7, PA1, PA0},
                    gpiob::{PB8, PB9},
                    gpioc::{PC13, PC14},
                    AlternateOD,
                    AF4,
                    AF5,
                    Alternate,
                    ExtiPin,
                    Edge,
                    Input,
                    PullDown,
                    PullUp
                },
};

#[app(device = crate::hal::stm32, peripherals = true)]
const APP : () = 
{

    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
        display: TerminalMode<I2CInterface<I2c<stm32::I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>, 
        DisplaySize128x64>,
        my_count : u8,
        remote_count : u8,
        a0: PA0<Input<PullUp>>,
        tx : serial::Tx<stm32::USART1>,
        rx : serial::Rx<stm32::USART1>

    }

    #[init(spawn = [disp])]
    fn init(c : init::Context) -> init::LateResources {

        let mut perfs : stm32::Peripherals = c.device;
        
        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr
                        .sysclk(48.mhz())
                        .pclk1(24.mhz())
                        .freeze();

        let gpioa = perfs.GPIOA.split();

        let mut a0 = gpioa.pa0.into_pull_up_input();
        a0.enable_interrupt(&mut perfs.EXTI);
        a0.make_interrupt_source(&mut perfs.SYSCFG);
        a0.trigger_on_edge(&mut perfs.EXTI, Edge::FALLING);

        let tx_pin = gpioa.pa9.into_alternate_af7();
        let rx_pin = gpioa.pa10.into_alternate_af7();

        let mut serial = serial::Serial::usart1(
            perfs.USART1,
            (tx_pin, rx_pin),
            serial::config::Config::default().baudrate(9600.bps()),
            clocks).unwrap();
        serial.listen(serial::Event::Rxne);

        let (tx, rx) = serial.split();


        let gpiob = perfs.GPIOB.split();

        let mut timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let scl = gpiob.pb8.into_alternate_af4().internal_pull_up(true).set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().internal_pull_up(true).set_open_drain();
        let i2c = I2c::i2c1(perfs.I2C1, (scl, sda), 100.khz(), clocks);
        
        let interface = I2CDIBuilder::new().init(i2c);
        let mut display: TerminalMode<_, _> = Builder::new().connect(interface).into();
        defmt::info!("Initializing display");
        display.init().unwrap();
        display.flush().unwrap();
        display.clear().unwrap();
        display.flush().unwrap();


        defmt::info!("I've init'd");

        c.spawn.disp().unwrap();

        init::LateResources {
            timer,
            display,
            my_count : 0,
            remote_count : 0,
            a0,
            tx,
            rx,
        }

    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(binds = EXTI0, resources = [a0, my_count, tx], spawn = [disp])]
    fn button_press(mut c : button_press::Context) {
        c.resources.a0.clear_interrupt_pending_bit();

        *c.resources.my_count +=1;
        
        let count = *c.resources.my_count;

        defmt::info!("Button has been pressed. my count is {}", &count);

        block!(c.resources.tx.write(count)).unwrap();
        c.spawn.disp().unwrap();

    }

    #[task(binds = USART1, resources = [rx, remote_count], spawn = [disp])]
    fn rx(mut c : rx::Context) {

        defmt::info!("Received comms");

        if let Ok(count) = c.resources.rx.read() {
            *c.resources.remote_count = count;
            c.spawn.disp().unwrap();
        }
        
    }

    #[task(resources = [display, my_count, remote_count])]
    fn disp(mut c : disp::Context) {
        let disp::Resources {display, my_count, remote_count } = c.resources;
        display.clear().unwrap();
        write!(display, "My count: {}\nRemote count: {}", my_count, remote_count).unwrap();
    }


    // #[task(binds = TIM3,
    //         priority = 2,
    //         resources = [timer, display, my_count, remote_count])]
    // fn tick(c : tick::Context) {

    //     let tick::Resources {timer, display, my_count, remote_count } = c.resources;

    //     timer.wait().ok();

    //     display.clear().unwrap();
    //     write!(display, "My count: {}\nRemote count: {}", my_count, remote_count).unwrap();

    // }

    extern "C" {
        fn DMA2_STREAM0();
    }

};
