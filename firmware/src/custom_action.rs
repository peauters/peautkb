use crate::dispatcher::{menu::MenuAction, DisplayedState, Message};
use crate::keyboard::*;
use crate::keymap::Layer;

use keyberon::key_code::KeyCode;
use keyberon::layout::{CustomEvent, Layout};

use heapless::{consts::U8, spsc::Queue};

pub enum PkbAction {
    MediaKey(MediaKey),
    MenuOpen,
    MenuClose,
    MenuUp,
    MenuDown,
    MenuSelect,
    MenuLeft,
    MenuRight,
    HoldCmd,
    ReleaseCmd,
    HoldCtrl,
    ReleaseCtrl,
}

pub struct CustomActionState {
    hold_cmd: bool,
    hold_ctrl: bool,
    current_layer: usize,
    is_primary: bool,
    mk_reports: Queue<MediaKeyHidReport, U8>,
}

impl CustomActionState {
    pub fn new() -> Self {
        CustomActionState {
            hold_cmd: false,
            hold_ctrl: false,
            current_layer: 0,
            is_primary: false,
            mk_reports: Queue::new(),
        }
    }

    pub fn is_primary(&mut self) {
        self.is_primary = true;
    }

    #[inline]
    pub fn process(&mut self, event: CustomEvent<PkbAction>) -> impl IntoIterator<Item = Message> {
        match event {
            CustomEvent::Press(PkbAction::MediaKey(mk)) => {
                self.mk_reports.enqueue(mk.into()).ok();
                None
            }
            CustomEvent::Release(PkbAction::MediaKey(_)) => {
                self.mk_reports.enqueue(MediaKeyHidReport::default()).ok();
                None
            }
            CustomEvent::Press(PkbAction::HoldCmd) => {
                self.hold_cmd = true;
                Some(Message::CmdHeld)
            }
            CustomEvent::Release(PkbAction::ReleaseCmd) => {
                self.hold_cmd = false;
                Some(Message::CmdReleased)
            }
            CustomEvent::Press(PkbAction::HoldCtrl) => {
                self.hold_ctrl = true;
                Some(Message::CtrlHeld)
            }
            CustomEvent::Release(PkbAction::ReleaseCtrl) => {
                self.hold_ctrl = false;
                Some(Message::CtrlReleased)
            }
            CustomEvent::Release(PkbAction::MenuOpen) => {
                if self.is_primary {
                    Some(Message::DisplaySelect(DisplayedState::Menu))
                } else {
                    None
                }
            }
            CustomEvent::Release(PkbAction::MenuUp) => Some(Message::Menu(MenuAction::Up)),
            CustomEvent::Release(PkbAction::MenuDown) => Some(Message::Menu(MenuAction::Down)),
            CustomEvent::Release(PkbAction::MenuSelect) => Some(Message::Menu(MenuAction::Select)),
            CustomEvent::Release(PkbAction::MenuClose) => Some(Message::Menu(MenuAction::Close)),
            CustomEvent::Release(PkbAction::MenuLeft) => Some(Message::Menu(MenuAction::Left)),
            CustomEvent::Release(PkbAction::MenuRight) => Some(Message::Menu(MenuAction::Right)),
            _ => None,
        }
    }

    pub fn get_mk_report(&mut self) -> Option<MediaKeyHidReport> {
        self.mk_reports.dequeue()
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
