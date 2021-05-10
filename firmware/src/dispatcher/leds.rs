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

        LEDs { leds }
    }

    fn initial() -> RGB8 {
        RGB8 { r: 255, g: 0, b: 0 }
    }
}

impl State for LEDs {
    type Messages = Option<Message>;
    fn handle_event(&mut self, message: Message) -> Self::Messages {
        match message {
            Message::LateInit => {
                let all_blue: [RGB8; 31] = [LEDs::initial(); 31];
                self.leds.write(all_blue.iter().cloned()).unwrap();
                defmt::info!("sent to leds");
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
