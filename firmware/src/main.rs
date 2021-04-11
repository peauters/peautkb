#![no_main]
#![no_std]

use cortex_m;
use stm32f4xx_hal as hal;
use hal::prelude::*;

use rtic::app;

use defmt_rtt as _; // global logger
use panic_probe as _;

use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use usb_device::device::UsbDeviceState;

use core::fmt::Write;
use ssd1306::{prelude::*, Builder, I2CDIBuilder, mode::TerminalMode};

use embedded_hal::spi::MODE_0;

use  ws2812_spi::{*, devices::Sk6812w};
use smart_leds::{SmartLedsWrite, RGBW, RGB8, White };

use crate::hal::{prelude::*, 
                stm32, 
                pwm,
                timer,
                interrupt,
                otg_fs,
                spi::*,
                i2c::I2c,
                gpio::{
                    gpioa::{PA5, PA6, PA7, PA1},
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

type UsbClass = keyberon::Class<'static, otg_fs::UsbBusType, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, otg_fs::UsbBusType>;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[app(device = crate::hal::stm32, peripherals = true)]
const APP : () = 
{

    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
        // usb_dev : UsbDevice,
        // usb_class : UsbClass,
        // display: TerminalMode<I2CInterface<I2c<stm32::I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>, 
        // DisplaySize128x64>,
        // leds : Ws2812<Spi<stm32::SPI1, (PA5<Alternate<AF5>>, PA6<Alternate<AF5>>, PA7<Alternate<AF5>>)>, Sk6812w>,
        // leg_a : PC14<Input<PullUp>>,
        // leg_b : PC13<Input<PullDown>>,
        a1 : PA1<Input<PullUp>>,
        exti : stm32::EXTI,
    }

    #[init]
    fn init(c : init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<otg_fs::UsbBusType>> = None;

        let mut perfs : stm32::Peripherals = c.device;
        
        let mut syscfg = perfs.SYSCFG;

        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr
                        .sysclk(48.mhz())
                        .pclk1(24.mhz())
                        .freeze();

        let gpioa = perfs.GPIOA.split();
        let gpiob = perfs.GPIOB.split();
        let gpioc = perfs.GPIOC.split();

        let mut timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let mut a1 = gpioa.pa1.into_pull_up_input();
        a1.make_interrupt_source(&mut syscfg);
        a1.enable_interrupt(&mut perfs.EXTI);
        a1.trigger_on_edge(&mut perfs.EXTI, Edge::FALLING);
 
        // let mut leg_a = gpioc.pc14.into_pull_up_input();


        // let leg_b = gpioc.pc13.into_pull_down_input();



        let spi = Spi::spi1(
            perfs.SPI1, 
            (
                gpioa.pa5.into_alternate_af5(),
                gpioa.pa6.into_alternate_af5(),
                gpioa.pa7.into_alternate_af5()), 
            ws2812_spi::MODE, 
            3_000_000.hz(), 
            clocks);
        
        let mut leds = Ws2812::new_sk6812w(spi);

        let mut data : [RGBW<u8, u8>; 5] = [RGBW::default(); 5]; 

        data[0] = RGBW::new_alpha(255, 0, 0, White::default());
        data[1] = RGBW::new_alpha(0, 255, 0, White::default());
        data[2] = RGBW::new_alpha(0, 0, 255, White::default());
        data[3] = RGBW::new_alpha(255, 255, 0, White::default());
        data[4] = RGBW::new_alpha(0, 0, 255, White::default());
        leds.write(data.iter().cloned()).unwrap();

        // let pwm = pwm::tim1(perfs.TIM1, (gpioa.pa8.into_alternate_af1(), gpioa.pa9.into_alternate_af1()), clocks, 3.mhz());

        // let (mut ch1, _ch2) = pwm;

        // let max_duty = ch1.get_max_duty();
        // ch1.set_duty((max_duty as f64 * 0.64) as u16);
        // // ch1.set_duty(0);
        // ch1.enable();


        // let usb = otg_fs::USB {
        //     usb_global : perfs.OTG_FS_GLOBAL,
        //     usb_device : perfs.OTG_FS_DEVICE,
        //     usb_pwrclk : perfs.OTG_FS_PWRCLK,
        //     pin_dm: gpioa.pa11.into_alternate_af10(),
        //     pin_dp: gpioa.pa12.into_alternate_af10(),
        // };
        // *USB_BUS = Some(otg_fs::UsbBusType::new(usb, unsafe { &mut EP_MEMORY }));
        // let usb_bus = USB_BUS.as_ref().unwrap();

        // let usb_class = keyberon::new_class(usb_bus, ());
        // let usb_dev = keyberon::new_device(usb_bus);

        // defmt::info!("Initialized USB");

        // let scl = gpiob.pb8.into_alternate_af4().internal_pull_up(true).set_open_drain();
        // let sda = gpiob.pb9.into_alternate_af4().internal_pull_up(true).set_open_drain();
        // let i2c = I2c::i2c1(perfs.I2C1, (scl, sda), 100.khz(), clocks);
        
        // let interface = I2CDIBuilder::new().init(i2c);
        // let mut display: TerminalMode<_, _> = Builder::new().connect(interface).into();
        // defmt::info!("Initializing display");
        // display.init().unwrap();
        // display.flush().unwrap();
        // display.clear().unwrap();
        // display.flush().unwrap();


        defmt::info!("I've init'd");

        init::LateResources {
            timer,
            // usb_dev,
            // usb_class,
            // display,
            // leds,
            // leg_a,
            // leg_b,
            exti: perfs.EXTI,
            a1,
        }

    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    // #[task(binds = OTG_FS, priority = 4, resources = [usb_dev, usb_class])]
    // fn usb_rx(c: usb_rx::Context) {
    //     if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
    //         c.resources.usb_class.poll();
    //     }
    // }

    #[task(binds = TIM3,
            priority = 2,
            resources = [timer])]
    fn tick(c : tick::Context) {

        c.resources.timer.wait().ok();

        // c.resources.display.write_str("blah ").unwrap();

        // if c.resources.a1.is_high().unwrap() {
        //    defmt::info!("high");

        // }
        // else {
        //     defmt::info!("low");

                // defmt::info!("a tick happend");

        // let leds = c.resources.leds;

        // let mut data : [RGBW<u8, u8>; 5] = [RGBW::default(); 5]; 

        // data[0] = RGBW::new_alpha(255, 0, 0, White::default());
        // data[1] = RGBW::new_alpha(0, 255, 0, White::default());
        // data[2] = RGBW::new_alpha(0, 0, 255, White::default());
        // data[3] = RGBW::new_alpha(255, 255, 0, White::default());
        // data[4] = RGBW::new_alpha(0, 0, 255, White::default());
        // leds.write(data.iter().cloned()).unwrap();
        // defmt::info!("a tick happend");


        }

        // defmt::info!("a tick happend");

        // let leds = c.resources.leds;

        // let mut data : [RGBW<u8, u8>; 5] = [RGBW::default(); 5]; 

        // data[0] = RGBW::new_alpha(255, 0, 0, White::default());
        // data[1] = RGBW::new_alpha(0, 255, 0, White::default());
        // data[2] = RGBW::new_alpha(0, 0, 255, White::default());
        // data[3] = RGBW::new_alpha(255, 255, 0, White::default());
        // data[4] = RGBW::new_alpha(0, 0, 255, White::default());
    //    leds.write(data.iter().cloned()).unwrap();

    // }

    #[task(binds = EXTI1,
        resources = [a1])]
    fn rotate(c : rotate::Context)
    {
        defmt::info!("rotate");
        c.resources.a1.clear_interrupt_pending_bit(); 
    }


};
