use super::*;

use crate::hal::{
    gpio::{gpiob::PB15, Alternate, AF5},
    prelude::*,
    rcc::Clocks,
    spi::{NoMiso, NoSck, Spi},
    stm32::SPI2,
};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    style::TextStyleBuilder,
};

use numtoa::NumToA;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub struct LEDs {
    leds: Ws2812<Spi<SPI2, (NoSck, NoMiso, PB15<Alternate<AF5>>)>>,
    i: u8,
    mode: Mode,
    solid_rgb: (u8, u8, u8),
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
    Off,
    Wheel,
    Solid,
}

impl From<Mode> for &str {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Off => "off",
            Mode::Wheel => "wheel",
            Mode::Solid => "solid",
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    SetMode(Mode),
    IncR,
    DecR,
    IncG,
    DecG,
    IncB,
    DecB,
    Solid((u8, u8, u8)),
    Update,
}

impl LEDs {
    pub fn new(spi2: SPI2, pb15: PB15<Alternate<AF5>>, clocks: Clocks) -> Self {
        let spi = Spi::spi2(
            spi2,
            (NoSck, NoMiso, pb15),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );

        let leds = Ws2812::new(spi);

        LEDs {
            leds,
            i: 0,
            mode: Mode::Solid,
            solid_rgb: (0, 128, 200),
        }
    }

    fn choose_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.update_leds();
    }

    fn update_leds(&mut self) {
        match self.mode {
            Mode::Off => self.off(),
            Mode::Wheel => self.wheel(),
            Mode::Solid => self.solid(),
        }
    }

    fn tick(&mut self) {
        match self.mode {
            Mode::Wheel => self.wheel(),
            _ => (),
        }
    }

    fn off(&mut self) {
        self.write_all((0, 0, 0));
    }

    fn solid(&mut self) {
        self.write_all(self.solid_rgb);
    }

    fn wheel(&mut self) {
        fn wheel(mut wheel_pos: u8) -> (u8, u8, u8) {
            wheel_pos = 255 - wheel_pos;
            if wheel_pos < 85 {
                return (255 - wheel_pos * 3, 0, wheel_pos * 3);
            }
            if wheel_pos < 170 {
                wheel_pos -= 85;
                return (0, wheel_pos * 3, 255 - wheel_pos * 3);
            }
            wheel_pos -= 170;
            (wheel_pos * 3, 255 - wheel_pos * 3, 0)
        }

        self.write_all(wheel(self.i));
        self.i = if self.i < 255 { self.i + 1 } else { 0 };
    }

    fn write_all(&mut self, colours: (u8, u8, u8)) {
        let colours: [RGB8; 31] = [colours.into(); 31];
        self.leds.write(colours.iter().cloned()).ok();
    }
}

impl State for LEDs {
    type Messages = Option<Message>;
    #[inline]
    fn handle_event(&mut self, message: Message) -> Self::Messages {
        match message {
            Message::UpdateDisplay => {
                self.tick();
                None
            }
            Message::LED(Action::SetMode(mode)) => {
                self.choose_mode(mode);
                Some(Message::SecondaryLED(Action::SetMode(mode)))
            }
            Message::LED(Action::IncR) => {
                self.solid_rgb.0 = self.solid_rgb.0.saturating_add(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecR) => {
                self.solid_rgb.0 = self.solid_rgb.0.saturating_sub(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::IncG) => {
                self.solid_rgb.1 = self.solid_rgb.1.saturating_add(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecG) => {
                self.solid_rgb.1 = self.solid_rgb.1.saturating_sub(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::IncB) => {
                self.solid_rgb.2 = self.solid_rgb.2.saturating_add(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecB) => {
                self.solid_rgb.2 = self.solid_rgb.2.saturating_sub(1);
                self.update_leds();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::Update) => {
                self.update_leds();
                Some(Message::SecondaryLED(Action::Update))
            }
            Message::SecondaryLED(Action::SetMode(mode)) => {
                self.choose_mode(mode);
                None
            }
            Message::SecondaryLED(Action::Solid(rgb)) => {
                self.solid_rgb = rgb;
                self.update_leds();
                None
            }
            Message::SecondaryLED(Action::Update) => {
                self.update_leds();
                None
            }
            Message::LateInit => {
                self.choose_mode(self.mode);
                None
            }
            _ => None,
        }
    }

    fn write_to_display<DI, DSIZE>(&mut self, display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand,
    {
        display.clear();
        let font_6x8 = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        Text::new("mode: ", Point::new(0, 0))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        Text::new(self.mode.into(), Point::new(36, 0))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        let mut buffer: [u8; 20] = [0; 20];

        Text::new("red: ", Point::new(0, 13))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        Text::new(
            self.solid_rgb.0.numtoa_str(16, &mut buffer),
            Point::new(42, 13),
        )
        .into_styled(font_6x8)
        .draw(display)
        .unwrap();

        Text::new("green: ", Point::new(0, 26))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        Text::new(
            self.solid_rgb.1.numtoa_str(16, &mut buffer),
            Point::new(42, 26),
        )
        .into_styled(font_6x8)
        .draw(display)
        .unwrap();

        Text::new("blue: ", Point::new(0, 39))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        Text::new(
            self.solid_rgb.2.numtoa_str(16, &mut buffer),
            Point::new(42, 39),
        )
        .into_styled(font_6x8)
        .draw(display)
        .unwrap();

        display.flush().unwrap();
    }
}
