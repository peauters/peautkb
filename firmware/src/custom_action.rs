use crate::dispatcher::Message;
use crate::keyboard::*;

use keyberon::key_code::KeyCode;
use keyberon::layout::CustomEvent;

pub enum PkbAction {
    MediaKey(MediaKey),
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
