use super::*;

use crate::hal::{
    gpio::{gpiob::PB15, Alternate, AF5},
    prelude::*,
    rcc::Clocks,
    spi::{NoMiso, NoSck, Spi},
    stm32::SPI2,
};

use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub struct LEDs {
    leds: Ws2812<Spi<SPI2, (NoSck, NoMiso, PB15<Alternate<AF5>>)>>,
    i: u8,
    on: bool,
    mode: Mode,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
    Wheel,
    Solid(u8, u8, u8),
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
            on: true,
            mode: Mode::Solid(0, 128, 200),
        }
    }

    fn choose_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match self.mode {
            Mode::Solid(_, _, _) => self.solid(),
            _ => (),
        }
    }

    fn update_leds(&mut self) {
        if self.on {
            match self.mode {
                Mode::Wheel => self.wheel(),
                _ => (),
            }
        }
    }

    fn solid(&mut self) {
        if let Mode::Solid(r, g, b) = self.mode {
            let colours: [RGB8; 31] = [(r, g, b).into(); 31];
            self.leds.write(colours.iter().cloned()).ok();
        }
    }

    fn wheel(&mut self) {
        fn wheel(mut wheel_pos: u8) -> RGB8 {
            wheel_pos = 255 - wheel_pos;
            if wheel_pos < 85 {
                return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
            }
            if wheel_pos < 170 {
                wheel_pos -= 85;
                return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
            }
            wheel_pos -= 170;
            (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
        }

        let colours: [RGB8; 31] = [wheel(self.i); 31];
        self.leds.write(colours.iter().cloned()).ok();
        self.i = if self.i < 255 { self.i + 1 } else { 0 };
    }
}

impl State for LEDs {
    type Messages = Option<Message>;
    #[inline]
    fn handle_event(&mut self, message: Message) -> Self::Messages {
        match message {
            Message::UpdateDisplay => {
                self.update_leds();
                None
            }
            Message::LEDMode(mode) => {
                self.choose_mode(mode);
                None
            }
            Message::LateInit => {
                self.choose_mode(self.mode);
                None
            }
            _ => None,
        }
    }

    fn write_to_display<DI, DSIZE>(&mut self, _display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand,
    {
    }
}
