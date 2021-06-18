use crate::custom_action::PkbAction;
use crate::keyboard::MediaKey;
use keyberon::action::{d, k, l, m, Action, Action::*, HoldTapConfig};
use keyberon::key_code::KeyCode::*;
use serde::{Deserialize, Serialize};

const PLAY_PAUSE: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::PlayPause));
const NEXT: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::NextTrack));
const PREVIOUS: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::PrevTrack));

const START_CMDT: Action<PkbAction> = MultipleActions(&[
    Custom(PkbAction::HoldCmd),
    d(Layer::Tabbing as usize),
    k(Tab),
]);
const END_CMDT: Action<PkbAction> =
    MultipleActions(&[Custom(PkbAction::ReleaseCmd), d(Layer::Default as usize)]);

const START_CTRLT: Action<PkbAction> = MultipleActions(&[
    Custom(PkbAction::HoldCtrl),
    d(Layer::Tabbing as usize),
    k(Tab),
]);
const END_CTRLT: Action<PkbAction> =
    MultipleActions(&[Custom(PkbAction::ReleaseCtrl), d(Layer::Default as usize)]);
const SHFT_TAB: Action<PkbAction> = m(&[LShift, Tab]);

const MENU_OPEN: Action<PkbAction> =
    MultipleActions(&[Custom(PkbAction::MenuOpen), d(Layer::Menu as usize)]);
const MENU_UP: Action<PkbAction> = Custom(PkbAction::MenuUp);
const MENU_DOWN: Action<PkbAction> = Custom(PkbAction::MenuDown);
const MENU_LEFT: Action<PkbAction> = Custom(PkbAction::MenuLeft);
const MENU_RIGHT: Action<PkbAction> = Custom(PkbAction::MenuRight);
const MENU_SELECT: Action<PkbAction> = Custom(PkbAction::MenuSelect);
const MENU_CLOSE: Action<PkbAction> =
    MultipleActions(&[Custom(PkbAction::MenuClose), d(Layer::Default as usize)]);

const ENTER_SHIFT: Action<PkbAction> = HoldTap {
    timeout: 200,
    hold: &k(RShift),
    tap: &k(Enter),
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
};

const SPACE_L4: Action<PkbAction> = HoldTap {
    timeout: 200,
    hold: &l(4),
    tap: &k(Space),
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
};

macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k])
    };
}

const LC: Action<PkbAction> = s!(LBracket);
const RC: Action<PkbAction> = s!(RBracket);

const LS: Action<PkbAction> = k(LBracket);
const RS: Action<PkbAction> = k(RBracket);

const LB: Action<PkbAction> = s!(Kb9);
const RB: Action<PkbAction> = s!(Kb0);

const SC: Action<PkbAction> = k(SColon);
const CO: Action<PkbAction> = s!(SColon);

const DQ: Action<PkbAction> = s!(Quote);
const QU: Action<PkbAction> = k(Quote);

const HASH: Action<PkbAction> = m(&[LAlt, Kb3]);
const TILDA: Action<PkbAction> = s!(Grave);

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<PkbAction> = &[
    &[
        &[k(Tab),     k(Q),         k(W),     k(F),       k(P),          k(B),      k(Escape),          k(Insert),  k(J),     k(L),              k(U),        k(Y),      k(Quote),   k(SColon)],
        &[k(LCtrl),   k(A),         k(R),     k(S),       k(T),          k(G),      MENU_OPEN,          k(Delete),  k(M),     k(N),              k(E),        k(I),      k(O),       k(Bslash)],
        &[k(LShift),  k(Z),         k(X),     k(C),       k(D),          k(V),      k(Mute),            PLAY_PAUSE, k(K),     k(H),              k(Comma),    k(Dot),    k(Slash),   k(RShift)],
        &[k(VolUp),   k(VolDown),   k(LAlt),  k(BSpace),  ENTER_SHIFT,   l(1),      k(LGui),            k(RGui),    l(2),     SPACE_L4,          k(RCtrl),    k(RAlt),   PREVIOUS,   NEXT],
    ], 
    &[
        &[Trans,      k(F1),        k(F2),    k(F3),      k(F4),         k(F5),     k(F6),              k(F7),      k(F8),    k(F9),             k(F10),      k(F11),    k(F12),     Trans],
        &[Trans,      k(Kb1),       k(Kb2),   k(Kb3),     k(Kb4),        k(Kb5),    NoOp,               NoOp,       k(Kb6),   k(Kb7),            k(Kb8),      k(Kb9),    k(Kb0),     Trans],
        &[Trans,      k(Grave),     NoOp,     NoOp,       k(Minus),      k(Equal),  Trans,              Trans,      NoOp,     k(LBracket),       k(RBracket), NoOp,      NoOp,       Trans],
        &[Trans,      Trans,        Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     Trans,      Trans],
    ],    
    &[   
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      NoOp,               NoOp,       NoOp,     NoOp,              k(Up),       NoOp,      NoOp,       NoOp],
        &[Trans,      k(Home),      k(PgUp),  k(PgDown),  k(End),        NoOp,      NoOp,               NoOp,       NoOp,     k(Left),           k(Down),     k(Right),  NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      START_CTRLT,        START_CMDT, NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      Trans,        Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     Trans,      Trans],
    ],    
    &[
        &[Trans,      s!(F1),       s!(F2),   s!(F3),     s!(F4),        s!(F5),    s!(F6),             s!(F7),     s!(F8),   s!(F9),            s!(F10),     s!(F11),   s!(F12),    Trans],
        &[Trans,      s!(Kb1),      s!(Kb2),  s!(Kb3),    s!(Kb4),       s!(Kb5),   NoOp,               NoOp,       s!(Kb6),  s!(Kb7),           s!(Kb8),     s!(Kb9),   s!(Kb0),    Trans],
        &[Trans,      s!(Grave),    NoOp,     NoOp,       s!(Minus),     s!(Equal), Trans,              Trans,      NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       Trans],
        &[Trans,      Trans,        Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     Trans,      Trans],
    ],    
    &[
        &[Trans,      NoOp,         NoOp,     HASH,       DQ,            NoOp,      NoOp,               NoOp,       NoOp,     QU,                TILDA,       NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         LS,       LB,         LC,            CO,        NoOp,               NoOp,       SC,       RC,                RB,          RS,        NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      NoOp,               NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      Trans,        Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     Trans,      Trans],
    ],
    &[   
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      NoOp,               NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      NoOp,               NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      END_CTRLT,          END_CMDT,   NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[k(Tab),     SHFT_TAB,     Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     k(Left),    k(Right)],
    ],    
    &[   
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      NoOp,               NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      MENU_CLOSE,         NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,          NoOp,      MENU_SELECT,        NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[MENU_DOWN,  MENU_UP,      Trans,    Trans,      Trans,         Trans,     Trans,              Trans,      Trans,    Trans,             Trans,       Trans,     MENU_LEFT,  MENU_RIGHT],
    ],   
    // CS    
    &[   
        &[k(Tab),     k(F),         k(Kb3),   k(W),       k(E),          k(R),      k(Escape),          NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      k(LShift),    k(A),     k(S),       k(D),          k(G),      MENU_OPEN,          NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[Trans,      k(LCtrl),     k(X),     k(T),       k(Kb5),        k(B),      k(Mute),            NoOp,       NoOp,     NoOp,              NoOp,        NoOp,      NoOp,       NoOp],
        &[k(VolUp),   k(VolDown),   k(Kb1),   k(Kb2),     k(Space),      k(Kb6),    k(Kb7),             Trans,      Trans,    Trans,             Trans,       Trans,     NoOp,       NoOp],
    ], 
];

// &[
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      NoOp,         NoOp,     NoOp,       NoOp,       NoOp,      NoOp,               NoOp,       NoOp,     NoOp,        NoOp,        NoOp,      NoOp,     NoOp],
//         &[Trans,      Trans,        Trans,    Trans,      Trans,      Trans,     Trans,              Trans,      Trans,    Trans,       Trans,       Trans,     Trans,    Trans],
//     ],

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Layer {
    Default,
    Numbers,
    Navigation,
    ShiftNumbers,
    Symbols,
    Tabbing,
    Menu,
    CS,
    Missing,
}

impl Default for Layer {
    fn default() -> Self {
        Layer::Default
    }
}

impl From<usize> for Layer {
    fn from(i: usize) -> Layer {
        match i {
            0 => Layer::Default,
            1 => Layer::Numbers,
            2 => Layer::Navigation,
            3 => Layer::ShiftNumbers,
            4 => Layer::Symbols,
            5 => Layer::Tabbing,
            6 => Layer::Menu,
            7 => Layer::CS,
            _ => Layer::Missing,
        }
    }
}

impl From<Layer> for usize {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Default => 0,
            Layer::Numbers => 1,
            Layer::Navigation => 2,
            Layer::ShiftNumbers => 3,
            Layer::Symbols => 4,
            Layer::Tabbing => 5,
            Layer::Menu => 6,
            Layer::CS => 7,
            Layer::Missing => 8,
        }
    }
}

impl From<Layer> for &str {
    fn from(layer: Layer) -> &'static str {
        match layer {
            Layer::Default => "default",
            Layer::Numbers => "numbers",
            Layer::Navigation => "nav",
            Layer::ShiftNumbers => "shift nums",
            Layer::Symbols => "symbols",
            Layer::Tabbing => "tabbing",
            Layer::Menu => "menu",
            Layer::CS => "CS",
            Layer::Missing => "missing",
        }
    }
}
