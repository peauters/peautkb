use smart_leds::RGB8;
use stm32f4xx_hal::dma::{
    config,
    traits::{Channel, DMASet, PeriAddress, Stream},
    MemoryToPeripheral, Transfer,
};

pub struct Ws2812<STREAM, CHANNEL, PERIPHERAL, BUF>
where
    STREAM: Stream,
    PERIPHERAL: PeriAddress,
{
    transfer: Transfer<STREAM, CHANNEL, PERIPHERAL, MemoryToPeripheral, BUF>,
    next_buf: Option<BUF>,
}

impl<STREAM, CHANNEL, PERIPHERAL> Ws2812<STREAM, CHANNEL, PERIPHERAL, &'static mut [u8; 512]>
where
    STREAM: Stream,
    CHANNEL: Channel,
    PERIPHERAL: PeriAddress<MemSize = u8> + DMASet<STREAM, CHANNEL, MemoryToPeripheral>,
{
    pub fn new(
        stream: STREAM,
        peripheral: PERIPHERAL,
        first_buf: &'static mut [u8; 512],
        second_buf: &'static mut [u8; 512],
    ) -> Self {
        let mut transfer = Transfer::init_memory_to_peripheral(
            stream,
            peripheral,
            first_buf,
            None,
            config::DmaConfig::default()
                .memory_increment(true)
                .fifo_error_interrupt(true)
                .transfer_complete_interrupt(true),
        );
        transfer.start(|_| ());

        Ws2812 {
            transfer,
            next_buf: Some(second_buf),
        }
    }

    pub fn write<'a, I>(&mut self, values: I)
    where
        I: IntoIterator<Item = RGB8>,
    {
        let next = self.next_buf.take().unwrap();

        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
        let mut index: usize = 0;
        for rgb in values.into_iter() {
            let RGB8 {
                mut r,
                mut g,
                mut b,
            } = rgb;

            for _ in 0..4 {
                let bits = (g & 0b1100_0000) >> 6;
                next[index] = patterns[bits as usize];
                index += 1;
                g <<= 2;
            }

            for _ in 0..4 {
                let bits = (r & 0b1100_0000) >> 6;
                next[index] = patterns[bits as usize];
                index += 1;
                r <<= 2;
            }

            for _ in 0..4 {
                let bits = (b & 0b1100_0000) >> 6;
                next[index] = patterns[bits as usize];
                index += 1;
                b <<= 2;
            }
        }

        let (old, _) = self.transfer.next_transfer(next).unwrap();

        self.next_buf = Some(old);
    }
}
