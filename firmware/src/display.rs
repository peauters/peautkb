use crate::hal::{
    gpio::{
        gpiob::{PB8, PB9},
        AlternateOD, AF4,
    },
    i2c::I2c,
    prelude::*,
    rcc::Clocks,
    stm32,
};

use core::fmt::Write;

use ssd1306::{
    displayrotation::DisplayRotation, mode::GraphicsMode, prelude::*, Builder, I2CDIBuilder,
};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::{TextStyle, TextStyleBuilder},
};

use arrayvec::ArrayString;

type DisplayType = GraphicsMode<
    I2CInterface<I2c<stm32::I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>,
    DisplaySize128x64,
>;

pub struct OLED {
    display: DisplayType,
    font_6x8: TextStyle<BinaryColor, Font6x8>,
    status: Status,
    is_dirty: bool,
    initd: bool,
}

impl OLED {
    pub fn new(
        i2c1: stm32::I2C1,
        pb8: PB8<AlternateOD<AF4>>,
        pb9: PB9<AlternateOD<AF4>>,
        clocks: Clocks,
    ) -> Self {
        let i2c = I2c::new(i2c1, (pb8, pb9), 100.khz(), clocks);
        let interface = I2CDIBuilder::new().init(i2c);
        OLED {
            display: Builder::new().connect(interface).into(),
            font_6x8: TextStyleBuilder::new(Font6x8)
                .text_color(BinaryColor::On)
                .build(),
            status: Status::new(),
            is_dirty: true,
            initd: false,
        }
    }

    pub fn is_left(&mut self) {
        self.status.hand = Some(Hand::Left);
        self.display
            .set_rotation(DisplayRotation::Rotate90)
            .unwrap();
        self.init();
    }

    pub fn is_right(&mut self) {
        self.status.hand = Some(Hand::Right);
        self.display
            .set_rotation(DisplayRotation::Rotate270)
            .unwrap();
        self.init();
    }

    pub fn is_usb(&mut self, is_usb: bool) {
        self.status.usb = Some(is_usb);
        self.is_dirty = true;
    }

    pub fn count_is(&mut self, i: u32) {
        self.status.count = i;
        self.is_dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn init(&mut self) {
        self.display.init().unwrap();
        self.display.flush().unwrap();
        self.initd = true;
    }

    pub fn update_display(&mut self) {
        if !self.initd {
            return;
        }

        Text::new("hand:", Point::zero())
            .into_styled(self.font_6x8)
            .draw(&mut self.display)
            .unwrap();

        let hand = match self.status.hand {
            Some(Hand::Left) => "left",
            Some(Hand::Right) => "right",
            None => "",
        };

        Text::new(hand, Point::new(36, 0))
            .into_styled(self.font_6x8)
            .draw(&mut self.display)
            .unwrap();

        Text::new("usb:", Point::new(0, 13))
            .into_styled(self.font_6x8)
            .draw(&mut self.display)
            .unwrap();

        let usb = match self.status.usb {
            Some(true) => "true",
            Some(false) => "false",
            None => "",
        };

        Text::new(usb, Point::new(30, 13))
            .into_styled(self.font_6x8)
            .draw(&mut self.display)
            .unwrap();

        let mut buf = ArrayString::<12>::new();

        write!(buf, "count: {}", self.status.count).unwrap();

        Text::new(&buf, Point::new(0, 26))
            .into_styled(self.font_6x8)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
        self.is_dirty = false;
    }
}

struct Status {
    hand: Option<Hand>,
    usb: Option<bool>,
    count: u32,
}

impl Status {
    fn new() -> Self {
        Status {
            hand: None,
            usb: None,
            count: 0,
        }
    }
}

#[derive(Debug)]
enum Hand {
    Left,
    Right,
}
