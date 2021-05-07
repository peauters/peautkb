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
    pub fn process(&mut self, event: CustomEvent<PkbAction>) -> Option<MediaKeyHidReport> {
        match event {
            CustomEvent::Press(PkbAction::MediaKey(mk)) => Some(mk.into()),
            CustomEvent::Release(PkbAction::MediaKey(_)) => Some(MediaKeyHidReport::default()),
            CustomEvent::Press(PkbAction::HoldCmd) => {
                self.hold_cmd = true;
                None
            }
            CustomEvent::Release(PkbAction::ReleaseCmd) => {
                self.hold_cmd = false;
                None
            }
            CustomEvent::Press(PkbAction::HoldCtrl) => {
                self.hold_ctrl = true;
                None
            }
            CustomEvent::Release(PkbAction::ReleaseCtrl) => {
                self.hold_ctrl = false;
                None
            }
            _ => None,
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
