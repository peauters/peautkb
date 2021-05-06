use crate::hal::gpio::{gpiob, Input, PullUp};
use embedded_hal::digital::v2::InputPin;

pub struct Rotary {
    pb4: gpiob::PB4<Input<PullUp>>,
    pb5: gpiob::PB5<Input<PullUp>>,
    last: (bool, bool),
    last_direction: Option<Direction>,
}

impl Rotary {
    pub fn new(pb4: gpiob::PB4<Input<PullUp>>, pb5: gpiob::PB5<Input<PullUp>>) -> Self {
        Rotary {
            pb4,
            pb5,
            last: (false, false),
            last_direction: None,
        }
    }

    pub fn poll(&mut self) -> Option<Direction> {
        let next = (self.pb4.is_high().unwrap(), self.pb5.is_high().unwrap());

        match (self.last, next) {
            ((false, false), (false, true))
            | ((false, true), (true, true))
            | ((true, false), (false, false))
            | ((true, true), (true, false)) => {
                self.last = next;
                if self.last_direction == Some(Direction::CW) {
                    self.last_direction = None;
                    Some(Direction::CW)
                } else {
                    self.last_direction = Some(Direction::CW);
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    CW,
    ACW,
}
