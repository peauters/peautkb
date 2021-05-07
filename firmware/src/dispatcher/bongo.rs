use super::*;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    style::TextStyleBuilder,
};

#[derive(Default)]
pub struct Bongo {}

impl State for Bongo {
    type Messages = Option<Message>;
    fn handle_event(&mut self, _message: Message) -> Self::Messages {
        None
    }
    fn write_to_display<DI, DSIZE>(&self, display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand,
    {
        display.clear();
        let font_6x8 = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();
        Text::new("bongo!", Point::new(10, 50))
            .into_styled(font_6x8)
            .draw(display)
            .unwrap();

        display.flush().unwrap();
    }
}
