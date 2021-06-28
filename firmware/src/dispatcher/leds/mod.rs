use super::*;

use crate::hal::{
    gpio::{gpiob::PB15, Alternate, AF5},
    prelude::*,
    rcc::Clocks,
    spi::{NoMiso, NoSck, Spi},
    stm32,
};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    style::TextStyleBuilder,
};

use numtoa::NumToA;
use smart_leds::RGB8;

mod driver;
mod fade;
mod off;
mod solid;
mod wheel;

use driver::Ws2812;
use stm32f4xx_hal::{
    dma::{Channel0, Stream4},
    spi::Tx,
};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
    Off,
    Wheel,
    Solid,
    Fade,
}

impl From<Mode> for &str {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Off => "off",
            Mode::Wheel => "wheel",
            Mode::Solid => "solid",
            Mode::Fade => "fade",
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    SetMode(Mode),
    IncrementRed,
    DecrementRed,
    IncrementGreen,
    DecrementGreen,
    IncrementBlue,
    DecrementBlue,
    Solid(solid::Solid),
    Update,
}
#[derive(Copy, Clone, Default)]
struct LEDMatrix {
    keys: [[RGB8; 7]; 3],
    thumb: [RGB8; 5],
    underglow: [RGB8; 6],
}

impl LEDMatrix {
    fn iter(self) -> Iter {
        Iter { matrix: self, i: 0 }
    }
}

impl IntoIterator for LEDMatrix {
    type Item = RGB8;

    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

struct Iter {
    matrix: LEDMatrix,
    i: usize,
}
impl Iterator for Iter {
    type Item = RGB8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 32 {
            let i = self.i;
            self.i += 1;
            match i {
                0 => Some(self.matrix.underglow[2]),
                1..=3 => Some(self.matrix.keys[0][7 - i]),
                4 => Some(self.matrix.underglow[1]),
                5..=6 => Some(self.matrix.keys[0][8 - i]),
                7 => Some(self.matrix.underglow[0]),
                8..=9 => Some(self.matrix.keys[0][9 - i]),
                10..=16 => Some(self.matrix.keys[1][i - 10]),
                17..=18 => Some(self.matrix.keys[2][22 - i]),
                19 => Some(self.matrix.underglow[4]),
                20..=21 => Some(self.matrix.keys[2][23 - i]),
                22 => Some(self.matrix.underglow[3]),
                23..=24 => Some(self.matrix.keys[2][24 - i]),
                25..=29 => Some(self.matrix.thumb[i - 25]),
                30 => Some(self.matrix.underglow[5]),
                _ => None,
            }
        } else {
            None
        }
    }
}

trait LEDMode {
    fn next_matrix(&mut self, last: LEDMatrix) -> Option<LEDMatrix>;
}

pub struct LEDs {
    leds: Ws2812<Stream4<stm32::DMA1>, Channel0, Tx<stm32::SPI2>, &'static mut [u8; 512]>,
    last: LEDMatrix,
    mode: Mode,
    solid_rgb: solid::Solid,
    off: off::Off,
    wheel: wheel::Wheel,
    fade: fade::FadeAfterRelease,
    sleep: bool,
}

impl LEDs {
    pub fn new(
        spi2: stm32::SPI2,
        pb15: PB15<Alternate<AF5>>,
        clocks: Clocks,
        stream: Stream4<stm32::DMA1>,
    ) -> Self {
        let spi = Spi::spi2(
            spi2,
            (NoSck, NoMiso, pb15),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );

        let buffer = cortex_m::singleton!(: [u8; 512] = [0; 512]).unwrap();
        let next_buffer = cortex_m::singleton!(: [u8; 512] = [0; 512]).unwrap();

        let leds = Ws2812::new(stream, spi.use_dma().tx(), buffer, next_buffer);

        LEDs {
            leds,
            last: LEDMatrix::default(),
            mode: Mode::Solid,
            solid_rgb: solid::Solid::new(),
            off: off::Off::new(),
            wheel: wheel::Wheel::new(),
            fade: fade::FadeAfterRelease::new(),
            sleep: false,
        }
    }

    fn choose_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.update_leds();
    }

    fn update_leds(&mut self) {
        match (self.sleep, self.mode) {
            (true, _) => (),
            (_, Mode::Off) => self.off(),
            (_, Mode::Solid) => self.solid(),
            (_, Mode::Wheel) => self.wheel(),
            (_, Mode::Fade) => self.fade(),
        }
    }

    fn tick(&mut self) {
        match self.mode {
            _ => self.update_leds(),
        }
    }

    fn off(&mut self) {
        let matrix = self.off.next_matrix(self.last);
        self.write_all(matrix);
    }

    fn solid(&mut self) {
        let matrix = self.solid_rgb.next_matrix(self.last);
        self.write_all(matrix);
    }

    fn wheel(&mut self) {
        let matrix = self.wheel.next_matrix(self.last);
        self.write_all(matrix);
    }

    fn fade(&mut self) {
        let matrix = self.fade.next_matrix(self.last);
        self.write_all(matrix);
    }

    fn write_all(&mut self, matrix: Option<LEDMatrix>) {
        if let Some(next) = matrix {
            self.leds.write(next);
            self.last = next;
        }
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
            Message::LED(Action::IncrementRed) => {
                self.solid_rgb.increment_red();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecrementRed) => {
                self.solid_rgb.decrement_red();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::IncrementGreen) => {
                self.solid_rgb.increment_green();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecrementGreen) => {
                self.solid_rgb.decrement_green();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::IncrementBlue) => {
                self.solid_rgb.increment_blue();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::LED(Action::DecrementBlue) => {
                self.solid_rgb.decrement_blue();
                Some(Message::SecondaryLED(Action::Solid(self.solid_rgb.clone())))
            }
            Message::SecondaryLED(Action::SetMode(mode)) => {
                self.choose_mode(mode);
                None
            }
            Message::SecondaryLED(Action::Solid(rgb)) => {
                self.solid_rgb = rgb;
                None
            }
            Message::LateInit => {
                self.choose_mode(self.mode);
                None
            }
            Message::MatrixKeyRelease(i, j) => {
                self.fade.key_release(i as usize, j as usize);
                None
            }
            Message::Sleep => {
                self.off();
                self.sleep = true;
                None
            }
            Message::Wake => {
                self.sleep = false;
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
            self.solid_rgb.red().numtoa_str(16, &mut buffer),
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
            self.solid_rgb.green().numtoa_str(16, &mut buffer),
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
            self.solid_rgb.blue().numtoa_str(16, &mut buffer),
            Point::new(42, 39),
        )
        .into_styled(font_6x8)
        .draw(display)
        .unwrap();

        display.flush().unwrap();
    }
}
