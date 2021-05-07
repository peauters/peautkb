use crate::dispatcher::{menu::MenuAction, DisplayedState, Message};
use crate::keyboard::*;

use keyberon::key_code::KeyCode;
use keyberon::layout::CustomEvent;

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
}

impl CustomActionState {
    pub fn new() -> Self {
        CustomActionState {
            hold_cmd: false,
            hold_ctrl: false,
        }
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

    pub fn modify_kb_report(&self, report: &mut KbHidReport) {
        if self.hold_cmd {
            report.pressed(KeyCode::LGui);
        }

        if self.hold_ctrl {
            report.pressed(KeyCode::LCtrl);
        }
    }
}
