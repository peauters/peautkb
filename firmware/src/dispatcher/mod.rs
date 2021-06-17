use serde::{Deserialize, Serialize};

use embedded_graphics::prelude::*;
use ssd1306::{displaysize::DisplaySize, mode::GraphicsMode, prelude::*};

use crate::keymap::Layer;

mod bongo;
pub mod display;
mod info;
pub mod leds;
pub mod menu;

pub struct Dispatcher {
    oled: display::OLED,
    displayed_state: DisplayedState,
    info: info::Info,
    menu: menu::Menu,
    leds: leds::LEDs,
    bongo: bongo::Bongo,
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
    pub fn new(oled: display::OLED, leds: leds::LEDs) -> Self {
        Dispatcher {
            oled,
            displayed_state: DisplayedState::default(),
            info: info::Info::default(),
            menu: menu::Menu::default(),
            leds,
            bongo: bongo::Bongo::default(),
        }
    }

    #[inline]
    pub fn dispatch(&mut self, message: Message) -> impl Iterator<Item = Message> {
        let messages = None.into_iter();

        dispatch!(messages, message, self.oled, self.info, self.leds, self.menu, self.bongo);

        match message {
            Message::DisplaySelect(d) => self.displayed_state = d,
            Message::SecondaryDisplaySelect(d) => self.displayed_state = d,
            _ => (),
        }
        messages
    }

    pub fn update_display(&mut self) {
        display!(
            self.displayed_state,
            self.oled,
            (DisplayedState::Info, &mut self.info),
            (DisplayedState::Menu, &mut self.menu),
            (DisplayedState::Bongo, &mut self.bongo),
            (DisplayedState::Leds, &mut self.leds)
        );
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    Ping,
    Pong,
    CmdHeld,
    CmdReleased,
    CtrlHeld,
    CtrlReleased,
    CurrentLayer(Layer),
    SecondaryCurrentLayer(Layer),
    DisplaySelect(DisplayedState),
    SecondaryDisplaySelect(DisplayedState),
    Menu(menu::MenuAction),
    SecondaryMenu(menu::SecondaryMenuAction),
    SetDefaultLayer(usize),
    Bongo,
    LED(leds::Action),
    SecondaryLED(leds::Action),
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
            | Message::SecondaryKeyRelease(_, _)
            | Message::SecondaryDisplaySelect(_)
            | Message::SecondaryCurrentLayer(_)
            | Message::SecondaryLED(_)
            | Message::SecondaryMenu(_)
            | Message::Bongo
            | Message::Pong => MessageType::Remote(self),
            _ => MessageType::Local(self),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DisplayedState {
    Info,
    Menu,
    Bongo,
    Leds,
}

impl Default for DisplayedState {
    fn default() -> Self {
        DisplayedState::Info
    }
}

pub trait State {
    type Messages: IntoIterator<Item = Message>;
    fn handle_event(&mut self, message: Message) -> Self::Messages;
    fn write_to_display<DI, DSIZE>(&mut self, display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand;
}
