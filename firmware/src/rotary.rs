use crate::hal::gpio::{gpiob, Input, PullUp};
use crate::multi::Multi;
use embedded_hal::digital::v2::InputPin;
use keyberon::layout::Event;
use stm32f4xx_hal::gpio::ExtiPin;

pub struct Rotary {
    pb4: gpiob::PB4<Input<PullUp>>,
    pb5: gpiob::PB5<Input<PullUp>>,
    last: (bool, bool),
    cw_coord: (u8, u8),
    acw_coord: (u8, u8),
    release: Multi<Event>,
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
            cw_coord,
            acw_coord,
            release: Multi::None,
        }
    }

    pub fn poll<'a>(&'a mut self) -> impl IntoIterator<Item = Event> + 'a {
        match self.read_and_debounce() {
            Some(d) => self.event_for(d),
            None => Multi::None,
        }
    }

    pub fn event_for(&mut self, d: Direction) -> Multi<Event> {
        match d {
            Direction::CW => {
                self.release
                    .append(Event::Release(self.cw_coord.0, self.cw_coord.1));
                Multi::One(Event::Press(self.cw_coord.0, self.cw_coord.1))
            }
            Direction::ACW => {
                self.release
                    .append(Event::Release(self.acw_coord.0, self.acw_coord.1));
                Multi::One(Event::Press(self.acw_coord.0, self.acw_coord.1))
            }
        }
    }

    pub fn release<'a>(&'a mut self) -> impl IntoIterator<Item = Event> + 'a {
        let out = self.release;
        self.release = Multi::None;
        out
    }

    pub fn clear_interrupt(&mut self) {
        self.pb5.clear_interrupt_pending_bit();
    }

    fn read_and_debounce(&mut self) -> Option<Direction> {
        let next = (self.pb4.is_high().unwrap(), self.pb5.is_high().unwrap());
        match (self.last, next) {
            ((true, false), (false, true))
            | ((false, true), (true, false))
            | ((true, true), (true, false))
            | ((false, false), (false, true)) => {
                self.last = next;
                Some(Direction::ACW)
            }
            ((false, false), (true, true))
            | ((true, true), (false, false))
            | ((false, true), (false, false))
            | ((true, false), (true, true)) => {
                self.last = next;
                Some(Direction::CW)
            }
            _ => {
                self.last = next;
                None
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    CW,
    ACW,
}
