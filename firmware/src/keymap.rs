use crate::custom_action::PkbAction;
use crate::keyboard::MediaKey;
use keyberon::action::{d, k, l, m, Action, Action::*};
use keyberon::key_code::KeyCode::*;

const PLAY_PAUSE: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::PlayPause));
const NEXT: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::NextTrack));
const PREVIOUS: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::PrevTrack));

const START_CMDT: Action<PkbAction> = MultipleActions(&[Custom(PkbAction::HoldCmd), d(3), k(Tab)]);
const END_CMDT: Action<PkbAction> = MultipleActions(&[Custom(PkbAction::ReleaseCmd), d(0)]);

const START_CTRLT: Action<PkbAction> =
    MultipleActions(&[Custom(PkbAction::HoldCtrl), d(3), k(Tab)]);
const END_CTRLT: Action<PkbAction> = MultipleActions(&[Custom(PkbAction::ReleaseCtrl), d(0)]);
const SHFT_TAB: Action<PkbAction> = m(&[LShift, Tab]);

const MENU_OPEN: Action<PkbAction> = MultipleActions(&[Custom(PkbAction::MenuOpen), d(4)]);
const MENU_UP: Action<PkbAction> = Custom(PkbAction::MenuUp);
const MENU_DOWN: Action<PkbAction> = Custom(PkbAction::MenuDown);
const MENU_SELECT: Action<PkbAction> = Custom(PkbAction::MenuSelect);
const MENU_CLOSE: Action<PkbAction> = MultipleActions(&[Custom(PkbAction::MenuClose), d(0)]);

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<PkbAction> = &[
    &[
        &[k(Tab),     k(Q),         k(W),     k(F),       k(P),       k(B),      k(Escape),          k(Insert),  k(J),     k(L),        k(U),        k(Y),      k(Quote),   k(SColon)],
        &[k(LCtrl),   k(A),         k(R),     k(S),       k(T),       k(G),      MENU_OPEN,          k(Delete),  k(K),     k(N),        k(E),        k(I),      k(O),       k(Bslash)],
        &[k(LShift),  k(Z),         k(X),     k(C),       k(D),       k(V),      k(Mute),            PLAY_PAUSE, k(M),     k(H),        k(Comma),    k(Dot),    k(Slash),   k(RShift)],
        &[k(VolUp),   k(VolDown),   k(LAlt),  k(Enter),   k(BSpace),  k(LGui),   l(1),               l(2),       k(RGui),  k(Space),    k(RCtrl),    k(RAlt),   PREVIOUS,   NEXT],
    ], 
    &[
        &[Trans,      k(F1),        k(F2),    k(F3),      k(F4),      k(F5),     k(F6),              k(F7),      k(F8),    k(F9),       k(F10),      k(F11),    k(F12),     Trans],
        &[Trans,      k(Kb1),       k(Kb2),   k(Kb3),     k(Kb4),     k(Kb5),    NoOp,               NoOp,       k(Kb6),   k(Kb7),      k(Kb8),      k(Kb9),    k(Kb0),     Trans],
        &[Trans,      k(Grave),     NoOp,     NoOp,       k(Minus),   k(Equal),  Trans,              Trans,      NoOp,     k(LBracket), k(RBracket), NoOp,      NoOp,       Trans],
        &[Trans,      Trans,        Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     Trans,      Trans],
    ], 
    &[
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        k(Up),       NoOp,      NoOp,       NoOp],
        &[Trans,      k(Home),      k(PgUp),  k(PgDown),  k(End),     NoOp,      NoOp,               NoOp,       NoOp,     k(Left),     k(Down),     k(Right),  NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      START_CTRLT,        START_CMDT, NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      Trans,        Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     Trans,      Trans],
    ], 
    &[
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      END_CTRLT,          END_CMDT,   NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[k(Tab),     SHFT_TAB,     Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     k(Left),    k(Right)],
    ], 
    &[
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      MENU_CLOSE,         NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      MENU_SELECT,        NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[MENU_DOWN,  MENU_UP,      Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     NoOp,       NoOp],
    ],
    // CS 
    &[
        &[k(Tab),     k(F),         k(Kb3),   k(W),       k(E),       k(R),      k(Escape),          NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      k(RShift),    k(A),     k(S),       k(D),       k(G),      MENU_OPEN,          NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      k(RCtrl),     k(X),     k(T),       k(Kb5),     k(B),      k(Mute),            NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,       NoOp],
        &[k(VolUp),   k(VolDown),   k(Kb1),   k(Kb2),     k(Space),   k(Kb6),    k(Kb7),             Trans,      Trans,    Trans,       Trans,       Trans,     NoOp,       NoOp],
    ], 
];

// &[
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      Trans,        Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     Trans,    Trans],
//     ],
