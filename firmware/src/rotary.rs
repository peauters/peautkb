use crate::hal::gpio::{gpiob, Input, PullUp};
use embedded_hal::digital::v2::InputPin;
use keyberon::layout::Event;

pub struct Rotary {
    pb4: gpiob::PB4<Input<PullUp>>,
    pb5: gpiob::PB5<Input<PullUp>>,
    last: (bool, bool),
    last_direction: Option<Direction>,
    last_event: Option<Event>,
    cw_coord: (u8, u8),
    acw_coord: (u8, u8),
}

impl Rotary {
    pub fn new(
        pb4: gpiob::PB4<Input<PullUp>>,
        pb5: gpiob::PB5<Input<PullUp>>,
        cw_coord: (u8, u8),
        acw_coord: (u8, u8),
    ) -> Self {
        Rotary {
            pb4,
            pb5,
            last: (false, false),
            last_direction: None,
            last_event: None,
            cw_coord,
            acw_coord,
        }
    }

    pub fn poll<'a>(&'a mut self) -> impl Iterator<Item = Event> + 'a {
        let release = match self.last_event {
            Some(Event::Press(i, j)) => {
                self.last_event = None;
                Some(Event::Release(i, j))
            }
            _ => None,
        };
        let press = match self.read_and_debounce() {
            Some(Direction::CW) => Some(self.update_last_event(self.cw_coord)),
            Some(Direction::ACW) => Some(self.update_last_event(self.acw_coord)),
            None => None,
        };
        release.into_iter().chain(press.into_iter())
    }

    fn update_last_event(&mut self, coord: (u8, u8)) -> Event {
        let event = Event::Press(coord.0, coord.1);
        self.last_event = Some(event.clone());
        event
    }

    fn read_and_debounce(&mut self) -> Option<Direction> {
        let next = (self.pb4.is_high().unwrap(), self.pb5.is_high().unwrap());
        match (self.last, next) {
            ((false, false), (false, true))
            | ((false, true), (true, true))
            | ((true, false), (false, false))
            | ((true, true), (true, false)) => {
                self.last = next;
                if self.last_direction == Some(Direction::ACW) {
                    self.last_direction = None;
                    Some(Direction::ACW)
                } else {
                    self.last_direction = Some(Direction::ACW);
                    None
                }
            }
            ((false, false), (true, false))
            | ((false, true), (false, false))
            | ((true, false), (true, true))
            | ((true, true), (false, true)) => {
                self.last = next;
                if self.last_direction == Some(Direction::CW) {
                    self.last_direction = None;
                    Some(Direction::CW)
                } else {
                    self.last_direction = Some(Direction::CW);
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    CW,
    ACW,
}
