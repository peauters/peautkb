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
            on: false,
        }
    }

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

    fn update_leds(&mut self) {
        if self.on {
            let colours: [RGB8; 31] = [LEDs::wheel(self.i); 31];
            self.leds.write(colours.iter().cloned()).ok();
            self.i = if self.i < 255 { self.i + 1 } else { 0 };
        }
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
