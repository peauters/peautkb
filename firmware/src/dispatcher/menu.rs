use super::*;

#[derive(Copy, Clone, Default)]
pub struct Menu {}

impl State for Menu {
    type Messages = Option<Message>;
    fn handle_event(&mut self, _message: Message) -> Self::Messages {
        None
    }

    fn write_to_display<DI, DSIZE>(&self, _display: &mut GraphicsMode<DI, DSIZE>)
    where
        DSIZE: DisplaySize,
        DI: WriteOnlyDataCommand,
    {
    }
}
