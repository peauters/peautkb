use crate::custom_action::PkbAction;
use crate::media_keys::MediaKey;
use keyberon::action::{k, l, Action, Action::*};
use keyberon::key_code::KeyCode::*;

const PLAY_PAUSE: Action<PkbAction> = Custom(PkbAction::MediaKey(MediaKey::PlayPause));

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<PkbAction> = &[
    &[
        &[k(Tab),     k(Q),         k(W),    k(F),   k(P),       k(B),      k(Escape),          k(Insert),  k(J),     k(L),     k(U),     k(Y),    k(Quote), k(SColon)],
        &[k(LCtrl),   k(A),         k(R),    k(S),   k(T),       k(G),      NoOp,               k(Delete),  k(K),     k(N),     k(E),     k(I),    k(O),     k(Bslash)],
        &[k(LShift),  k(Z),         k(X),    k(C),   k(D),       k(V),      k(Mute),            NoOp,       k(M),     k(H),     k(Comma), k(Dot),  k(Slash), k(RShift)],
        &[k(VolUp),   k(VolDown),   k(LAlt), l(1),   k(BSpace),  k(Enter),  k(LGui),            k(RGui),    k(RCtrl), k(Space), l(2),     k(RAlt), NoOp,     NoOp],
    ], &[
        &[k(Tab),     k(F1),    k(F2),    k(F3),   k(F4),     k(F5),     k(F6),              k(F7),   k(F8),       k(F9),       k(F10),      k(F11),    k(F12),   Trans],
        &[k(LCtrl),   k(Kb1),   k(Kb2),   k(Kb3),  k(Kb4),    k(Kb5),    NoOp,               NoOp,    k(Kb6),      k(Kb7),      k(Kb8),      k(Kb9),    k(Kb0),   Trans],
        &[k(LShift),  k(Grave), NoOp,     NoOp,    k(Minus),  k(Equal),  PLAY_PAUSE       ,  NoOp,    NoOp,        k(LBracket), k(RBracket), NoOp,      NoOp,     Trans],
        &[Trans,      Trans,    Trans,    Trans,   Trans,     Trans,     Trans,              Trans,   Trans,       Trans,       Trans,       Trans,     Trans,    Trans],
    ], &[
        &[k(Tab),     NoOp,     NoOp,     NoOp,       NoOp,      NoOp,      NoOp,    NoOp,    NoOp,     NoOp,     k(Up),   NoOp,      NoOp,    NoOp],
        &[k(LCtrl),   k(Home),  k(PgUp),  k(PgDown),  k(End),    NoOp,      NoOp,    NoOp,    NoOp,     k(Left),  k(Down), k(Right),  NoOp,    NoOp],
        &[k(LShift),  NoOp,     NoOp,     NoOp,       NoOp,      NoOp,      NoOp,    NoOp,    NoOp,     NoOp,     NoOp,    NoOp,      NoOp,    NoOp],
        &[Trans,      Trans,    Trans,    Trans,      Trans,     Trans,     Trans,   Trans,   Trans,    Trans,    Trans,   Trans,     Trans,   Trans],

    ], 
];
