use super::*;

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    style::TextStyleBuilder,
};

use crate::dispatcher::leds::{Action, Mode};
use crate::multi::{Multi, Multi::*};

#[rustfmt::skip]
const MENU : &[&[MenuItem]] = &[
    &[i("ping", Message::Ping), sm("display", 1), sm("leds", 5), sm("keymap", 4)],
    &[sm("left", 2), sm("right", 3)],
    &[i("info", Message::DisplaySelect(DisplayedState::Info)), i("bongo", Message::DisplaySelect(DisplayedState::Bongo)), i("leds", Message::DisplaySelect(DisplayedState::Leds))],
    &[i("info", Message::SecondaryDisplaySelect(DisplayedState::Info)), i("bongo", Message::SecondaryDisplaySelect(DisplayedState::Bongo)), i("leds", Message::SecondaryDisplaySelect(DisplayedState::Leds))],
    &[i("default", Message::SetDefaultLayer(0)), i("cs", Message::SetDefaultLayer(Layer::CS as usize))],
    &[i("off", Message::LED(Action::SetMode(leds::Mode::Off))), smn("solid", 6, DisplayedState::Leds, Message::LED(Action::SetMode(Mode::Solid))), i("wheel", Message::LED(Action::SetMode(leds::Mode::Wheel)))],
    &[d("red", Message::LED(Action::DecR), Message::LED(Action::IncR)), d("green", Message::LED(Action::DecG), Message::LED(Action::IncG)), d("blue", Message::LED(Action::DecB), Message::LED(Action::IncB)), d("update", Message::LED(Action::Update), Message::LED(Action::Update))]];

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuAction {
    Up,
    Down,
    Select,
    Close,
    Left,
    Right,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecondaryMenuAction {
    Open(DisplayedState),
    Close,
}

#[derive(Copy, Clone, Default)]
pub struct Menu {
    current_menu: usize,
    current_item: usize,
    last_display_state: DisplayedState,
    default_layer: usize,
    previous_menu: Multi<usize>,
    previous_item: Multi<usize>,
}

impl Menu {
    fn up(&mut self) -> Multi<Message> {
        if self.current_item > 0 {
            self.current_item -= 1;
        }
        None
    }

    fn down(&mut self) -> Multi<Message> {
        if self.current_item < MENU[self.current_menu].len() {
            self.current_item += 1;
        }
        None
    }

    fn left(&mut self) -> Multi<Message> {
        match MENU[self.current_menu][self.current_item - 1].menu_type {
            Type::Dial(left, _) => One(left),
            _ => None,
        }
    }

    fn right(&mut self) -> Multi<Message> {
        match MENU[self.current_menu][self.current_item - 1].menu_type {
            Type::Dial(_, right) => One(right),
            _ => None,
        }
    }

    fn select(&mut self) -> Multi<Message> {
        match (self.current_menu, self.current_item) {
            (0, 0) => self.close(),
            (_, 0) => {
                self.current_menu = self.previous_menu.take().unwrap_or(0);
                self.current_item = self.previous_item.take().unwrap_or(0);
                None
            }
            _ => {
                let item = &MENU[self.current_menu][self.current_item - 1];
                let messages = match item.menu_type {
                    Type::Item => self.close(),
                    Type::SubMenu(i) => {
                        self.previous_menu.push(self.current_menu);
                        self.previous_item.push(self.current_item);
                        self.current_menu = i;
                        self.current_item = 0;
                        None
                    }
                    Type::SecondaryMenu(i, secondary) => {
                        self.previous_menu.push(self.current_menu);
                        self.current_menu = i;
                        self.current_item = 0;
                        One(Message::SecondaryMenu(SecondaryMenuAction::Open(secondary)))
                    }
                    _ => None,
                };
                messages.add(item.message)
            }
        }
    }

    fn close(&mut self) -> Multi<Message> {
        self.previous_menu = None;
        self.previous_item = None;
        self.current_menu = 0;
        self.current_item = 0;
        Three(
            Message::SetDefaultLayer(self.default_layer),
            Message::DisplaySelect(self.last_display_state),
            Message::SecondaryMenu(SecondaryMenuAction::Close),
        )
    }

    fn last_display_state(&mut self, s: DisplayedState) -> Multi<Message> {
        if s != DisplayedState::Menu {
            self.last_display_state = s;
        }
        None
    }
}

impl State for Menu {
    type Messages = Multi<Message>;

    #[inline]
    fn handle_event(&mut self, message: Message) -> Self::Messages {
        match message {
            Message::Menu(MenuAction::Up) => self.up(),
            Message::Menu(MenuAction::Down) => self.down(),
            Message::Menu(MenuAction::Left) => self.left(),
            Message::Menu(MenuAction::Right) => self.right(),
            Message::Menu(MenuAction::Select) => self.select(),
            Message::Menu(MenuAction::Close) => self.close(),
            Message::DisplaySelect(s) => self.last_display_state(s),
            Message::SecondaryMenu(SecondaryMenuAction::Open(s)) => {
                self.last_display_state(s);
                One(Message::DisplaySelect(s))
            }
            Message::SecondaryMenu(SecondaryMenuAction::Close) => {
                self.close();
                None
            }
            Message::SetDefaultLayer(l) => {
                self.default_layer = l;
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
        let menu = MENU[self.current_menu];

        display.clear();
        let font_6x8 = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        Text::new("back", Point::new(7, 0))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();
        for (i, item) in menu.iter().enumerate() {
            Text::new(item.name, Point::new(7, ((i + 1) * 13) as i32))
                .into_styled(font_6x8)
                .draw(display)
                .unwrap();
        }

        Text::new("-", Point::new(0, (self.current_item * 13) as i32))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        display.flush().unwrap();
    }
}

#[derive(Copy, Clone)]
struct MenuItem {
    name: &'static str,
    menu_type: Type,
    message: Multi<Message>,
}

const fn i(name: &'static str, message: Message) -> MenuItem {
    MenuItem {
        name,
        menu_type: Type::Item,
        message: One(message),
    }
}

const fn sm(name: &'static str, index: usize) -> MenuItem {
    MenuItem {
        name,
        menu_type: Type::SubMenu(index),
        message: None,
    }
}

const fn smn(
    name: &'static str,
    index: usize,
    snd_display: DisplayedState,
    message: Message,
) -> MenuItem {
    MenuItem {
        name,
        menu_type: Type::SecondaryMenu(index, snd_display),
        message: One(message),
    }
}

const fn d(name: &'static str, left: Message, right: Message) -> MenuItem {
    MenuItem {
        name,
        menu_type: Type::Dial(left, right),
        message: None,
    }
}

#[derive(Copy, Clone)]
enum Type {
    Item,
    SubMenu(usize),
    SecondaryMenu(usize, DisplayedState),
    Dial(Message, Message),
}
