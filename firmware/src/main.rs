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

use crate::hal::{prelude::*, 
                stm32, 
                timer,
                otg_fs};

type UsbClass = keyberon::Class<'static, otg_fs::UsbBusType, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, otg_fs::UsbBusType>;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[app(device = crate::hal::stm32, peripherals = true)]
const APP : () = 
{

    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
        usb_dev : UsbDevice,
        usb_class : UsbClass,
    }

    #[init]
    fn init(mut c : init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<otg_fs::UsbBusType>> = None;

        let mut perfs : stm32::Peripherals = c.device;
        
        let rcc = perfs.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let gpioa = perfs.GPIOA.split();

        let mut timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let usb = otg_fs::USB {
            usb_global : perfs.OTG_FS_GLOBAL,
            usb_device : perfs.OTG_FS_DEVICE,
            usb_pwrclk : perfs.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
        };
        *USB_BUS = Some(otg_fs::UsbBusType::new(usb, unsafe { &mut EP_MEMORY }));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);

        defmt::info!("I've init'd");

        init::LateResources {
            timer,
            usb_dev,
            usb_class,
        }

    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(binds = OTG_FS, priority = 4, resources = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
            c.resources.usb_class.poll();
        }
    }

    #[task(binds = TIM3,
            priority = 2,
            resources = [timer, usb_dev])]
    fn tick(c : tick::Context) {

        c.resources.timer.wait().ok();

        defmt::info!("a tick happend");
        
        let mut usb_dev = c.resources.usb_dev;

        if usb_dev.lock( |d| d.state()) == UsbDeviceState::Configured {
            defmt::info!("I'm a HID");
        }

    }


};
