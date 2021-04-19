#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

pub mod display;
pub mod serial;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [DMA2_STREAM0, DMA2_STREAM1])]
mod app {

    use crate::display::OLED;
    use crate::serial::*;

    use crate::hal::{otg_fs, prelude::*, stm32, timer};
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::Milliseconds;

    use crate::app;

    use usb_device::bus::UsbBusAllocator;
    use usb_device::class::UsbClass as _;
    use usb_device::device::UsbDeviceState;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    type UsbClass = keyberon::Class<'static, otg_fs::UsbBusType, ()>;
    type UsbDevice = usb_device::device::UsbDevice<'static, otg_fs::UsbBusType>;

    #[resources]
    struct Resources {
        timer: timer::Timer<stm32::TIM3>,
        display: OLED,
        usb_dev: app::UsbDevice,
        usb_class: app::UsbClass,
        tx: TxComms,
        rx: RxComms,
        primary: Option<bool>,
        conn_established: bool,
    }

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    #[init]
    fn init(mut c: init::Context) -> (init::LateResources, init::Monotonics) {
        static mut USB_BUS: Option<UsbBusAllocator<otg_fs::UsbBusType>> = None;
        let perfs: stm32::Peripherals = c.device;

        let rcc = perfs.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .use_hse(25.mhz())
            .freeze();

        let mono = DwtSystick::new(&mut c.core.DCB, c.core.DWT, c.core.SYST, clocks.hclk().0);

        let gpioa = perfs.GPIOA.split();
        let gpiob = perfs.GPIOB.split();

        let timer = timer::Timer::tim3(perfs.TIM3, 1.hz(), clocks);

        // I2C for SSD1306 display
        let pb8 = gpiob
            .pb8
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let pb9 = gpiob
            .pb9
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let display = OLED::new(perfs.I2C1, pb8, pb9, clocks);

        // USB keyboard
        let usb = otg_fs::USB {
            usb_global: perfs.OTG_FS_GLOBAL,
            usb_device: perfs.OTG_FS_DEVICE,
            usb_pwrclk: perfs.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
            hclk: clocks.hclk(),
        };
        *USB_BUS = Some(otg_fs::UsbBusType::new(usb, unsafe { &mut EP_MEMORY }));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);

        let pa9 = gpioa.pa9.into_alternate_af7();
        let pa10 = gpioa.pa10.into_alternate_af7();

        let (tx, rx) = create_comms(perfs.USART1, pa9, pa10, clocks);
        defmt::info!("I've init'd");

        late_init::spawn_after(Milliseconds::new(500_u32)).ok();
        (
            init::LateResources {
                timer,
                display,
                usb_class,
                usb_dev,
                tx,
                rx,
                primary: None,
                conn_established: false,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop()
        }
    }

    #[task(binds = USART1, priority = 3, resources = [rx, display])]
    fn rx(mut c: rx::Context) {
        match c.resources.rx.lock(RxComms::read_event) {
            Some(message) => match message {
                Message::URSecondary => {
                    defmt::info!("Got message URSecondart message");
                    c.resources.display.lock(OLED::is_right);
                    c.resources.display.lock(|o| o.is_usb(false));
                }
                Message::ACount(i) => c.resources.display.lock(|o| o.count_is(i)),
            },
            None => {}
        }
    }

    #[task(binds = OTG_FS, priority = 4, resources = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        let mut usb_class = c.resources.usb_class;
        let mut usb_dev = c.resources.usb_dev;
        usb_class.lock(|class| {
            usb_dev.lock(|dev| {
                if dev.poll(&mut [class]) {
                    class.poll();
                }
            })
        });
    }

    #[task(binds = TIM3,
            priority = 3,
            resources = [timer, display])]
    fn tick(c: tick::Context) {
        let tick::Resources {
            mut timer,
            mut display,
        } = c.resources;
        timer.lock(|t| t.wait().ok());
        display.lock(|o| {
            if o.is_dirty() {
                o.update_display();
            }
        });
    }

    #[task(resources = [display, timer, usb_dev, tx])]
    fn late_init(mut c: late_init::Context) {
        defmt::info!("Late initializing");
        c.resources.display.lock(OLED::init);
        c.resources.timer.lock(|t| t.listen(timer::Event::TimeOut));
        if c.resources.usb_dev.lock(|d| d.state()) == UsbDeviceState::Configured {
            c.resources.display.lock(OLED::is_left);
            c.resources.display.lock(|d| d.is_usb(true));
            c.resources.tx.lock(|t| {
                t.send_event(Message::URSecondary);
            });
        }

        defmt::info!("Initialized");
    }
}
