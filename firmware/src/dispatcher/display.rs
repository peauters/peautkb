use crate::hal::{
    gpio::{
        gpiob::{PB6, PB7},
        AlternateOD, AF4,
    },
    i2c::I2c,
    prelude::*,
    rcc::Clocks,
    stm32,
};

use ssd1306::{
    displayrotation::DisplayRotation, mode::GraphicsMode, prelude::*, Builder, I2CDIBuilder,
};

use super::*;

type DisplayType = GraphicsMode<
    I2CInterface<I2c<stm32::I2C1, (PB6<AlternateOD<AF4>>, PB7<AlternateOD<AF4>>)>>,
    DisplaySize128x64,
>;

pub struct OLED {
    display: DisplayType,
    initd: bool,
}

impl OLED {
    pub fn new(
        i2c1: stm32::I2C1,
        pb6: PB6<AlternateOD<AF4>>,
        pb7: PB7<AlternateOD<AF4>>,
        clocks: Clocks,
    ) -> Self {
        let i2c = I2c::new(i2c1, (pb6, pb7), 400.khz(), clocks);
        let interface = I2CDIBuilder::new().init(i2c);
        OLED {
            display: Builder::new().connect(interface).into(),
            initd: false,
        }
    }

    pub fn display<S: super::State>(&mut self, state: &mut S) {
        if self.initd {
            state.write_to_display(&mut self.display);
        }
    }

    fn is_left(&mut self) {
        defmt::info!("is_left");
        self.display
            .set_rotation(DisplayRotation::Rotate270)
            .unwrap();
        self.display.clear();
        self.display.flush().unwrap();
    }

    fn is_right(&mut self) {
        defmt::info!("is_right");
        self.display
            .set_rotation(DisplayRotation::Rotate90)
            .unwrap();
        self.display.clear();
        self.display.flush().unwrap();
    }

    fn init(&mut self) {
        defmt::info!("display init");
        self.display.init().unwrap();
        self.display.clear();
        self.display.flush().unwrap();
        self.initd = true;

        defmt::info!("done");
    }
}

impl super::State for OLED {
    type Messages = Option<Message>;
    fn handle_event(&mut self, message: Message) -> Self::Messages {
        match message {
            Message::LateInit => self.init(),
            Message::YouArePrimary => self.is_left(),
            Message::YouAreSecondary => self.is_right(),
            _ => (),
        }

        if message == Message::LateInit {
            Some(Message::InitTimers)
        } else {
            None
        }
    }

    fn write_to_display<DI, DSIZE>(&mut self, _display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand,
    {
    }
}
