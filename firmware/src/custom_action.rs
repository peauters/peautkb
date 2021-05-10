use crate::dispatcher::{menu::MenuAction, DisplayedState, Layer, Message};
use crate::keyboard::*;

use keyberon::key_code::KeyCode;
use keyberon::layout::{CustomEvent, Layout};

pub enum PkbAction {
    MediaKey(MediaKey),
    MenuOpen,
    MenuClose,
    MenuUp,
    MenuDown,
    MenuSelect,
    HoldCmd,
    ReleaseCmd,
    HoldCtrl,
    ReleaseCtrl,
}

#[derive(Default)]
pub struct CustomActionState {
    hold_cmd: bool,
    hold_ctrl: bool,
    current_layer: usize,
    is_primary: bool,
}

impl CustomActionState {
    pub fn new() -> Self {
        CustomActionState {
            hold_cmd: false,
            hold_ctrl: false,
            current_layer: 0,
            is_primary: false,
        }
    }

    pub fn is_primary(&mut self) {
        self.is_primary = true;
    }

    pub fn process(
        &mut self,
        event: CustomEvent<PkbAction>,
    ) -> (Option<MediaKeyHidReport>, impl IntoIterator<Item = Message>) {
        match event {
            CustomEvent::Press(PkbAction::MediaKey(mk)) => (Some(mk.into()), None),
            CustomEvent::Release(PkbAction::MediaKey(_)) => {
                (Some(MediaKeyHidReport::default()), None)
            }
            CustomEvent::Press(PkbAction::HoldCmd) => {
                self.hold_cmd = true;
                (None, Some(Message::CmdHeld))
            }
            CustomEvent::Release(PkbAction::ReleaseCmd) => {
                self.hold_cmd = false;
                (None, Some(Message::CmdReleased))
            }
            CustomEvent::Press(PkbAction::HoldCtrl) => {
                self.hold_ctrl = true;
                (None, Some(Message::CtrlHeld))
            }
            CustomEvent::Release(PkbAction::ReleaseCtrl) => {
                self.hold_ctrl = false;
                (None, Some(Message::CtrlReleased))
            }
            CustomEvent::Release(PkbAction::MenuOpen) => {
                (None, Some(Message::DisplaySelect(DisplayedState::Menu)))
            }
            CustomEvent::Release(PkbAction::MenuUp) => (None, Some(Message::Menu(MenuAction::Up))),
            CustomEvent::Release(PkbAction::MenuDown) => {
                (None, Some(Message::Menu(MenuAction::Down)))
            }
            CustomEvent::Release(PkbAction::MenuSelect) => {
                (None, Some(Message::Menu(MenuAction::Select)))
            }
            CustomEvent::Release(PkbAction::MenuClose) => {
                (None, Some(Message::Menu(MenuAction::Close)))
            }
            _ => (None, None),
        }
    }

    pub fn check_layout_for_events(
        &mut self,
        layout: &Layout<PkbAction>,
    ) -> impl IntoIterator<Item = Message> {
        if self.is_primary {
            let new_layer = layout.current_layer();
            if self.current_layer != new_layer {
                self.current_layer = new_layer;
                Some(Message::CurrentLayer(Layer::from(new_layer)))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn modify_kb_report(&self, report: &mut KbHidReport) {
        if self.hold_cmd {
            report.pressed(KeyCode::LGui);
        }

        if self.hold_ctrl {
            report.pressed(KeyCode::LCtrl);
        }
    }
}
