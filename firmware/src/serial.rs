use crate::hal::{
    gpio::{
        gpioa::{PA10, PA9},
        Alternate, AF7,
    },
    prelude::*,
    rcc::Clocks,
    serial,
    stm32::USART1,
};

use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};

pub struct TxComms {
    tx: serial::Tx<USART1>,
    buffer: [u8; 64],
}

pub fn create_comms(
    usart1: USART1,
    pa9: PA9<Alternate<AF7>>,
    pa10: PA10<Alternate<AF7>>,
    clocks: Clocks,
) -> (TxComms, RxComms) {
    let mut serial = serial::Serial::usart1(
        usart1,
        (pa9, pa10),
        serial::config::Config::default().baudrate(9600.bps()),
        clocks,
    )
    .unwrap();
    serial.listen(serial::Event::Rxne);

    let (tx, rx) = serial.split();

    (
        TxComms {
            tx,
            buffer: [0; 64],
        },
        RxComms {
            rx,
            buffer: [0; 64],
            offset: 0,
        },
    )
}

impl TxComms {
    pub fn send_event(&mut self, message: Message) {
        let used = to_slice(&message, &mut self.buffer).unwrap();
        self.tx.bwrite_all(used).unwrap();
    }
}

pub struct RxComms {
    rx: serial::Rx<USART1>,
    buffer: [u8; 64],
    offset: usize,
}

impl RxComms {
    pub fn read_event(&mut self) -> Option<Message> {
        self.buffer[self.offset] = self.rx.read().unwrap();
        self.offset += 1;

        match from_bytes::<Message>(&self.buffer[0..self.offset]) {
            Ok(message) => {
                self.offset = 0;
                Some(message)
            }
            Err(_) => None,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    URSecondary,
    ACount(u32),
}
