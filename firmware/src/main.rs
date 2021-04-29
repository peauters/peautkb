#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

pub mod dispatcher;
pub mod keymap;
pub mod serial;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [SPI4, SPI5, SPI6])]
mod app {

    use crate::dispatcher::display::OLED;
    use crate::dispatcher::*;
    use crate::keymap::LAYERS;
    use crate::serial::*;

    use core::convert::Infallible;

    use crate::hal::{
        gpio::{gpioa, gpiob, Input, Output, PullUp, PushPull},
        otg_fs,
        prelude::*,
        stm32, timer,
    };
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::Milliseconds;

    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use generic_array::typenum::{U4, U7};
    use keyberon::debounce::Debouncer;
    use keyberon::impl_heterogenous_array;
    use keyberon::key_code::KbHidReport;
    use keyberon::layout::{Event, Layout};
    use keyberon::matrix::{Matrix, PressedKeys};

    use crate::app;

    use usb_device::bus::UsbBusAllocator;
    use usb_device::class::UsbClass as _;
    use usb_device::device::UsbDeviceState;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    type UsbClass = keyberon::Class<'static, otg_fs::UsbBusType, ()>;
    type UsbDevice = usb_device::device::UsbDevice<'static, otg_fs::UsbBusType>;

    pub struct Cols(
        gpioa::PA6<Input<PullUp>>,
        gpioa::PA5<Input<PullUp>>,
        gpioa::PA4<Input<PullUp>>,
        gpioa::PA3<Input<PullUp>>,
        gpioa::PA2<Input<PullUp>>,
        gpioa::PA1<Input<PullUp>>,
        gpioa::PA7<Input<PullUp>>,
    );
    impl_heterogenous_array! {
        Cols,
        dyn InputPin<Error = Infallible>,
        U7,
        [0, 1, 2, 3, 4, 5, 6]
    }
    pub struct Rows(
        gpiob::PB10<Output<PushPull>>,
        gpiob::PB2<Output<PushPull>>,
        gpiob::PB1<Output<PushPull>>,
        gpiob::PB0<Output<PushPull>>,
    );
    impl_heterogenous_array! {
        Rows,
        dyn OutputPin<Error = Infallible>,
        U4,
        [0, 1, 2, 3]
    }

    #[resources]
    struct Resources {
        scan_timer: timer::Timer<stm32::TIM3>,
        tick_timer: timer::Timer<stm32::TIM4>,
        usb_dev: app::UsbDevice,
        usb_class: app::UsbClass,
        tx: TxComms,
        rx: RxComms,
        dispatcher: Dispatcher,
        matrix: Matrix<app::Cols, app::Rows>,
        debouncer: Debouncer<PressedKeys<U4, U7>>,
        layout: Layout,
        initd: bool,
        timer_init: bool,
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

        let scan_timer = timer::Timer::tim3(perfs.TIM3, 500.hz(), clocks);
        let tick_timer = timer::Timer::tim4(perfs.TIM4, 1.hz(), clocks);

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

        let matrix = Matrix::new(
            Cols(
                gpioa.pa6.into_pull_up_input(),
                gpioa.pa5.into_pull_up_input(),
                gpioa.pa4.into_pull_up_input(),
                gpioa.pa3.into_pull_up_input(),
                gpioa.pa2.into_pull_up_input(),
                gpioa.pa1.into_pull_up_input(),
                gpioa.pa7.into_pull_up_input(),
            ),
            Rows(
                gpiob.pb10.into_push_pull_output(),
                gpiob.pb2.into_push_pull_output(),
                gpiob.pb1.into_push_pull_output(),
                gpiob.pb0.into_push_pull_output(),
            ),
        )
        .unwrap();

        let debouncer = Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5);

        let layout = Layout::new(LAYERS);

        (
            init::LateResources {
                scan_timer,
                tick_timer,
                usb_class,
                usb_dev,
                tx,
                rx,
                dispatcher: Dispatcher::new(display),
                initd: false,
                matrix,
                debouncer,
                layout,
                timer_init: false,
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

    #[task(binds = USART1, priority = 3, resources = [rx, initd, layout])]
    fn rx(c: rx::Context) {
        let rx::Resources {
            mut rx,
            mut initd,
            mut layout,
        } = c.resources;

        initd.lock(|b| {
            if !*b {
                late_init::spawn_after(Milliseconds::new(1000_u32)).ok();
                *b = true;
            }
        });
        rx.lock(|rx| {
            if let Some(message) = rx.read_event() {
                dispatch_event::spawn(message).unwrap();
                match message {
                    Message::SecondaryKeyPress(i, j) => {
                        layout.lock(|l| l.event(Event::Press(i, j)));
                        send_hid_report::spawn().unwrap();
                    }
                    Message::SecondaryKeyRelease(i, j) => {
                        layout.lock(|l| l.event(Event::Release(i, j)));
                        send_hid_report::spawn().unwrap();
                    }
                    _ => (),
                }
            }
        });
    }

    #[task(binds = OTG_FS, priority = 4, resources = [usb_dev, usb_class, initd])]
    fn usb_rx(mut c: usb_rx::Context) {
        let mut usb_class = c.resources.usb_class;
        let mut usb_dev = c.resources.usb_dev;
        usb_class.lock(|class| {
            usb_dev.lock(|dev| {
                if dev.poll(&mut [class]) {
                    class.poll();
                }
            })
        });
        c.resources.initd.lock(|b| {
            if !*b {
                late_init::spawn_after(Milliseconds::new(1000_u32)).ok();
                *b = true;
            }
        });
    }

    #[task(binds = OTG_FS_WKUP, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_wkup(c: usb_wkup::Context) {
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
            resources = [scan_timer, debouncer, matrix, layout])]
    fn scan(c: scan::Context) {
        let scan::Resources {
            mut scan_timer,
            mut debouncer,
            mut matrix,
            mut layout,
        } = c.resources;
        scan_timer.lock(|t| t.wait().ok());

        let pressed_keys = matrix.lock(|m| m.get().unwrap());
        layout.lock(|l| {
            debouncer.lock(|d| {
                for event in d.events(pressed_keys) {
                    l.event(event);
                    match event {
                        Event::Press(i, j) => {
                            dispatch_event::spawn(Message::MatrixKeyPress(i, j)).unwrap()
                        }
                        Event::Release(i, j) => {
                            dispatch_event::spawn(Message::MatrixKeyRelease(i, j)).unwrap()
                        }
                    }
                }
            })
        });
        send_hid_report::spawn().unwrap();
    }

    #[task(binds = TIM4,
        priority = 3,
        resources = [tick_timer])]
    fn tick(c: tick::Context) {
        let tick::Resources { mut tick_timer } = c.resources;
        tick_timer.lock(|t| t.wait().ok());

        dispatch_event::spawn(Message::Tick).unwrap();
    }

    #[task(resources = [layout, usb_dev, usb_class], capacity = 64)]
    fn send_hid_report(mut c: send_hid_report::Context) {
        // tick() returns CustomEvent, so ...?
        c.resources.layout.lock(|l| l.tick());
        let report: KbHidReport = c.resources.layout.lock(|l| l.keycodes().collect());
        if !c
            .resources
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        if c.resources.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        while let Ok(0) = c.resources.usb_class.lock(|k| k.write(report.as_bytes())) {}
    }

    #[task(resources = [dispatcher, tx, timer_init, scan_timer, tick_timer], capacity = 20)]
    fn dispatch_event(c: dispatch_event::Context, message: Message) {
        // let mut tx = c.resources.tx;
        let dispatch_event::Resources {
            mut dispatcher,
            mut tx,
            mut timer_init,
            mut scan_timer,
            mut tick_timer,
        } = c.resources;

        dispatcher.lock(|d| {
            d.dispatch(message)
                .map(Message::to_type)
                .for_each(|t| match t {
                    MessageType::Local(m) => {
                        dispatch_event::spawn(m).unwrap();
                        timer_init.lock(|t| {
                            if !*t {
                                scan_timer.lock(|t| t.listen(timer::Event::TimeOut));
                                tick_timer.lock(|t| t.listen(timer::Event::TimeOut));
                                *t = true;
                            }
                        })
                    }
                    MessageType::Remote(m) => tx.lock(|t| t.send_event(m)),
                })
        });
    }

    #[task(resources = [usb_dev])]
    fn late_init(c: late_init::Context) {
        let late_init::Resources { mut usb_dev } = c.resources;
        defmt::info!("late init");
        dispatch_event::spawn(Message::LateInit).unwrap();

        if usb_dev.lock(|d| d.state()) == UsbDeviceState::Configured {
            dispatch_event::spawn(Message::YouArePrimary).unwrap();
            dispatch_event::spawn(Message::UsbConnected(true)).unwrap();
        }
    }
}
