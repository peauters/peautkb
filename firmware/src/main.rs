#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;

use rtic::app;

use stm32f4xx_hal as hal;

pub mod custom_action;
pub mod dispatcher;
pub mod keyboard;
pub mod keymap;
pub(crate) mod multi;
pub mod rotary;
pub mod serial;

#[app(device = crate::hal::stm32, peripherals = true, dispatchers = [SPI4, SPI5, SPI6])]
mod app {

    use crate::custom_action::*;
    use crate::dispatcher::display::OLED;
    use crate::dispatcher::*;
    use crate::keyboard::*;
    use crate::keymap::LAYERS;
    use crate::rotary::*;
    use crate::serial::*;

    use core::convert::Infallible;

    use crate::hal::{
        gpio::{gpioa, gpiob, Edge, Input, Output, PullUp, PushPull},
        otg_fs,
        prelude::*,
        stm32, timer,
    };
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::Milliseconds;

    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use generic_array::typenum::{U4, U7};
    use keyberon::debounce::Debouncer;
    use keyberon::hid;
    use keyberon::impl_heterogenous_array;
    use keyberon::layout::{Event, Layout};
    use keyberon::matrix::{Matrix, PressedKeys};
    use stm32f4xx_hal::dma::StreamsTuple;

    use crate::app;

    use usb_device::bus::UsbBusAllocator;
    use usb_device::class::UsbClass as _;
    use usb_device::device::UsbDeviceState;
    use usb_device::prelude::*;

    #[monotonic(binds = SysTick, default = true)]
    type Mono = DwtSystick<48_000_000>; // 48 MHz

    type UsbMediaKeysClass = hid::HidClass<'static, otg_fs::UsbBusType, Peautkb>;
    type UsbDevice = usb_device::device::UsbDevice<'static, otg_fs::UsbBusType>;
    /// USB VIP for a generic keyboard from
    /// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
    const VID: u16 = 0x16c0;

    /// USB PID for a generic keyboard from
    /// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
    const PID: u16 = 0x27db;

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
        usb_mediakeys_class: app::UsbMediaKeysClass,
        tx: TxComms,
        rx: RxComms,
        dispatcher: Dispatcher,
        matrix: Matrix<app::Cols, app::Rows>,
        debouncer: Debouncer<PressedKeys<U4, U7>>,
        layout: Layout<PkbAction>,
        initd: bool,
        timer_init: bool,
        rotary: Rotary,
        custom_action_state: CustomActionState,
    }

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    #[init]
    fn init(mut c: init::Context) -> (init::LateResources, init::Monotonics) {
        static mut USB_BUS: Option<UsbBusAllocator<otg_fs::UsbBusType>> = None;
        let mut perfs: stm32::Peripherals = c.device;

        let rcc = perfs.RCC.constrain();
        let mut syscfg = perfs.SYSCFG.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .use_hse(25.mhz())
            .freeze();

        let mono = DwtSystick::new(&mut c.core.DCB, c.core.DWT, c.core.SYST, clocks.hclk().0);

        let gpioa = perfs.GPIOA.split();
        let gpiob = perfs.GPIOB.split();

        let scan_timer = timer::Timer::tim3(perfs.TIM3, 1000.hz(), clocks);
        let tick_timer = timer::Timer::tim4(perfs.TIM4, 24.hz(), clocks);

        // I2C for SSD1306 display
        let pb6 = gpiob
            .pb6
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let pb7 = gpiob
            .pb7
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let display = OLED::new(perfs.I2C1, pb6, pb7, clocks);

        // Rotary encoder pins
        let pb4 = gpiob.pb4.into_pull_up_input();
        let mut pb5 = gpiob.pb5.into_pull_up_input();

        pb5.make_interrupt_source(&mut syscfg);
        pb5.enable_interrupt(&mut perfs.EXTI);
        pb5.trigger_on_edge(&mut perfs.EXTI, Edge::RISING_FALLING);

        let rotary = Rotary::new(pb4, pb5, (3, 0), (3, 1));

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

        let usb_mediakeys_class = hid::HidClass::new(Peautkb::default(), usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("peauters.dev")
            .product("peautkb")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        // Inter-board comms
        let pa9 = gpioa.pa9.into_alternate_af7();
        let pa10 = gpioa.pa10.into_alternate_af7();

        let (tx, rx) = create_comms(perfs.USART1, pa9, pa10, clocks);

        // Keyberon setup
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

        let steams = StreamsTuple::new(perfs.DMA1);
        let stream = steams.4;

        let leds = leds::LEDs::new(perfs.SPI2, gpiob.pb15.into_alternate_af5(), clocks, stream);

        ping::spawn_after(Milliseconds::new(4000_u32)).ok();

        (
            init::LateResources {
                scan_timer,
                tick_timer,
                usb_mediakeys_class,
                usb_dev,
                tx,
                rx,
                dispatcher: Dispatcher::new(display, leds),
                initd: false,
                matrix,
                debouncer,
                layout,
                timer_init: false,
                rotary,
                custom_action_state: CustomActionState::new(),
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

    #[task(binds = USART1, priority = 3, resources = [rx, initd, layout, custom_action_state])]
    fn rx(c: rx::Context) {
        let rx::Resources {
            mut rx,
            mut initd,
            mut layout,
            mut custom_action_state,
        } = c.resources;

        initd.lock(|b| {
            if !*b {
                late_init::spawn_after(Milliseconds::new(4000_u32)).ok();
                *b = true;
            } else {
                rx.lock(|rx| {
                    if let Some(message) = rx.read_event() {
                        dispatch_event::spawn(message).ok();
                        match message {
                            Message::SecondaryKeyPress(i, j) => {
                                layout.lock(|l| {
                                    l.event(Event::Press(i, j));
                                    custom_action_state.lock(|c| {
                                        let messages = c.process(l.tick());
                                        for m in messages.into_iter() {
                                            dispatch_event::spawn(m).ok();
                                        }
                                    });
                                });
                                send_hid_report::spawn().ok();
                            }
                            Message::SecondaryKeyRelease(i, j) => {
                                layout.lock(|l| {
                                    l.event(Event::Release(i, j));
                                    custom_action_state.lock(|c| {
                                        let messages = c.process(l.tick());
                                        for m in messages.into_iter() {
                                            dispatch_event::spawn(m).ok();
                                        }
                                    });
                                });
                                send_hid_report::spawn().ok();
                            }
                            _ => (),
                        }
                    }
                });
            }
        });
    }

    #[task(binds = OTG_FS, priority = 4, resources = [usb_dev, usb_mediakeys_class, initd])]
    fn usb_rx(c: usb_rx::Context) {
        let usb_rx::Resources {
            mut usb_dev,
            mut usb_mediakeys_class,
            mut initd,
        } = c.resources;
        usb_dev.lock(|dev| {
            usb_mediakeys_class.lock(|mk| {
                if dev.poll(&mut [mk]) {
                    mk.poll();
                }
            })
        });
        initd.lock(|b| {
            if !*b {
                late_init::spawn_after(Milliseconds::new(1000_u32)).ok();
                *b = true;
            }
        });
    }

    #[task(binds = OTG_FS_WKUP, priority = 2, resources = [usb_dev, usb_mediakeys_class])]
    fn usb_wkup(c: usb_wkup::Context) {
        let usb_wkup::Resources {
            mut usb_dev,
            mut usb_mediakeys_class,
        } = c.resources;
        usb_dev.lock(|dev| {
            usb_mediakeys_class.lock(|mk| {
                if dev.poll(&mut [mk]) {
                    mk.poll();
                }
            })
        });
    }

    #[task(binds = EXTI9_5, priority = 2, resources = [rotary, layout, custom_action_state])]
    fn rot5(c: rot5::Context) {
        let rot5::Resources {
            mut rotary,
            mut layout,
            mut custom_action_state,
        } = c.resources;
        let mut dirty = false;
        layout.lock(|l| {
            rotary.lock(|r| {
                r.clear_interrupt();
                for event in r.poll() {
                    dirty = true;
                    l.event(event);
                    match event {
                        Event::Press(i, j) => {
                            dispatch_event::spawn(Message::MatrixKeyPress(i, j)).ok();
                        }
                        Event::Release(i, j) => {
                            dispatch_event::spawn(Message::MatrixKeyRelease(i, j)).ok();
                        }
                    }
                    custom_action_state.lock(|c| {
                        let messages = c.process(l.tick());
                        for m in messages.into_iter() {
                            dispatch_event::spawn(m).ok();
                        }
                    });
                }
            });
        });

        if dirty {
            send_hid_report::spawn().ok();
        }
    }

    #[task(binds = TIM3,
            priority = 3,
            resources = [scan_timer, debouncer, matrix, layout, custom_action_state, rotary])]
    fn scan(c: scan::Context) {
        let scan::Resources {
            mut scan_timer,
            mut debouncer,
            mut matrix,
            mut layout,
            mut custom_action_state,
            mut rotary,
        } = c.resources;
        scan_timer.lock(|t| t.wait().ok());

        let mut dirty = false;

        let pressed_keys = matrix.lock(|m| m.get().unwrap());
        layout.lock(|l| {
            debouncer.lock(|d| {
                rotary.lock(|r| {
                    for event in d.events(pressed_keys).chain(r.release()) {
                        dirty = true;
                        l.event(event);
                        match event {
                            Event::Press(i, j) => {
                                dispatch_event::spawn(Message::MatrixKeyPress(i, j)).ok();
                            }
                            Event::Release(i, j) => {
                                dispatch_event::spawn(Message::MatrixKeyRelease(i, j)).ok();
                            }
                        }
                    }
                    custom_action_state.lock(|c| {
                        let messages = c.process(l.tick());

                        for m in messages.into_iter() {
                            dispatch_event::spawn(m).ok();
                        }
                    });
                    custom_action_state.lock(|c| {
                        for m in c.check_layout_for_events(l) {
                            dispatch_event::spawn(m).ok();
                        }
                    });
                })
            });
        });

        send_hid_report::spawn().ok();
    }

    #[task(binds = TIM4,
        priority = 3,
        resources = [tick_timer])]
    fn tick(c: tick::Context) {
        let tick::Resources { mut tick_timer } = c.resources;
        tick_timer.lock(|t| t.wait().ok());

        dispatch_event::spawn(Message::UpdateDisplay).ok();
    }

    #[task(resources = [layout, usb_dev, usb_mediakeys_class, custom_action_state], priority = 2, capacity = 64)]
    fn send_hid_report(c: send_hid_report::Context) {
        let send_hid_report::Resources {
            mut layout,
            mut usb_dev,
            mut usb_mediakeys_class,
            mut custom_action_state,
        } = c.resources;

        while let Some(mut mk_report) = custom_action_state.lock(|c| c.get_mk_report()) {
            if usb_mediakeys_class.lock(|k| k.device_mut().set_report(mk_report.clone()))
                && usb_dev.lock(|d| d.state()) == UsbDeviceState::Configured
            {
                while let Ok(0) = usb_mediakeys_class.lock(|m| m.write(mk_report.as_bytes())) {}
            }
        }
        let mut report: KbHidReport = layout.lock(|l| l.keycodes().collect());
        custom_action_state.lock(|c| c.modify_kb_report(&mut report));
        if usb_mediakeys_class.lock(|k| k.device_mut().set_kb_report(report.clone()))
            && usb_dev.lock(|d| d.state()) == UsbDeviceState::Configured
        {
            while let Ok(0) = usb_mediakeys_class.lock(|k| k.write(report.as_bytes())) {}
        }
    }

    #[task(resources = [dispatcher, tx, timer_init, scan_timer, tick_timer, layout], priority = 1, capacity = 30)]
    fn dispatch_event(c: dispatch_event::Context, message: Message) {
        let dispatch_event::Resources {
            mut dispatcher,
            mut tx,
            mut timer_init,
            mut scan_timer,
            mut tick_timer,
            mut layout,
        } = c.resources;

        dispatcher.lock(|d| {
            if message == Message::UpdateDisplay {
                d.update_display();
            }
            d.dispatch(message)
                .map(Message::to_type)
                .for_each(|t| match t {
                    MessageType::Local(m) => {
                        dispatch_event::spawn(m).ok();
                        match m {
                            Message::InitTimers => timer_init.lock(|t| {
                                if !*t {
                                    scan_timer.lock(|t| t.listen(timer::Event::TimeOut));
                                    tick_timer.lock(|t| t.listen(timer::Event::TimeOut));
                                    *t = true;
                                }
                            }),
                            Message::SetDefaultLayer(i) => {
                                layout.lock(|l| l.set_default_layer(i));
                            }
                            _ => (),
                        }
                    }
                    MessageType::Remote(m) => tx.lock(|t| t.send_event(m)),
                })
        });
    }

    #[task(resources = [tx, initd])]
    fn ping(c: ping::Context) {
        defmt::info!("Pinging ... ");
        let ping::Resources { mut tx, mut initd } = c.resources;
        initd.lock(|i| {
            if !*i {
                tx.lock(|t| t.send_event(Message::Ping))
            }
        });
    }

    #[task(resources = [usb_dev, custom_action_state])]
    fn late_init(c: late_init::Context) {
        let late_init::Resources {
            mut usb_dev,
            mut custom_action_state,
        } = c.resources;
        defmt::info!("late init");
        dispatch_event::spawn(Message::LateInit).ok();

        if usb_dev.lock(|d| d.state()) == UsbDeviceState::Configured {
            dispatch_event::spawn(Message::YouArePrimary).ok();
            dispatch_event::spawn(Message::UsbConnected(true)).ok();
            custom_action_state.lock(|l| l.is_primary());
        } else {
            dispatch_event::spawn(Message::YouAreSecondary).ok();
        }
    }
}
