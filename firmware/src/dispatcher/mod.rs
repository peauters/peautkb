use serde::{Deserialize, Serialize};

use embedded_graphics::prelude::*;
use ssd1306::{displaysize::DisplaySize, mode::GraphicsMode, prelude::*};

pub mod display;
mod info;
mod menu;

pub struct Dispatcher {
    oled: display::OLED,
    displayed_state: DisplayedState,
    info: info::Info,
    menu: menu::Menu,
}

macro_rules! display {
    ($v:expr, $m:expr, $(($s:pat, $a:expr)),+) => {
        match $v {
            $($s => $m.display($a),)+
        }
    };
}

macro_rules! dispatch {
    ($ms:ident, $m:ident, $($s:expr),*) => {
            $(
                let l = $s.handle_event($m);
                let $ms = $ms.chain(l);
            )*
    };
}

impl Dispatcher {
    pub fn new(oled: display::OLED) -> Self {
        Dispatcher {
            oled,
            displayed_state: DisplayedState::Info,
            info: info::Info::default(),
            menu: menu::Menu::default(),
        }
    }

    pub fn dispatch(&mut self, message: Message) -> impl Iterator<Item = Message> {
        let messages = None.into_iter();
        dispatch!(messages, message, self.oled, self.info);

        if message == Message::Tick {
            // self.update_display();
        }
        messages
    }

    fn update_display(&mut self) {
        display!(
            self.displayed_state,
            self.oled,
            (DisplayedState::Info, self.info),
            (DisplayedState::Menu, self.menu)
        );
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Message {
    LateInit,
    InitTimers,
    UsbConnected(bool),
    YouArePrimary,
    YouAreSecondary,
    UpdateDisplay,
    Tick,
    MatrixKeyPress(u8, u8),
    MatrixKeyRelease(u8, u8),
    SecondaryKeyPress(u8, u8),
    SecondaryKeyRelease(u8, u8),
}

pub enum MessageType {
    Local(Message),
    Remote(Message),
}

impl Message {
    pub fn to_type(self) -> MessageType {
        match self {
            Message::YouAreSecondary
            | Message::SecondaryKeyPress(_, _)
            | Message::SecondaryKeyRelease(_, _) => MessageType::Remote(self),
            _ => MessageType::Local(self),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DisplayedState {
    Info,
    Menu,
}

pub trait State {
    type Messages: IntoIterator<Item = Message>;
    fn handle_event(&mut self, message: Message) -> Self::Messages;
    fn write_to_display<DI, DSIZE>(&self, display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand;
}
