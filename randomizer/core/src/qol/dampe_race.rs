use crate::bfevfl::{Bf, Event, Flow, Kind, Val};
use crate::error::{Error, Res};

pub const FLOW: &str = "MiniGame_DampeRace";
pub const STATE: &str = "Minigame_DampeRace_TreasureBoxState";
pub const APPEAR: &str = "MiniGame_DampeRace_IsAppearTreasureBox";
pub const SHOW: &str = "MiniGame_DampeRace_TreasureBoxAppear";

pub const NOTE: &str = "One Dampé race: opening the Hookshot chest after the first race makes the Heart Piece chest appear in its place, so both checks come from a single race with no time to beat. In MiniGame_DampeRace, Event201 (TreasureBoxState = OpenedHookShot) sets AppearHeartPiece instead, turns IsAppearTreasureBox on and moves to the TreasureBoxAppear event, as the race result does. Always on.";

fn named(f: &Flow, name: &str) -> Res<usize> {
    let hits: Vec<usize> = (0..f.events.len()).filter(|&i| f.events[i].name == name).collect();
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("one dampe race: expected one {name} in {FLOW}, found {}", hits.len()))),
    }
}

fn param<'a>(e: &'a Event, k: &str) -> Option<&'a Val> {
    e.kind.params().and_then(|c| c.get(k))
}

fn is(e: &Event, k: &str, v: &str) -> bool {
    param(e, k) == Some(&Val::Str(v.to_string()))
}

pub fn patch(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == FLOW).ok_or(Error::Format("one dampe race: MiniGame_DampeRace has no flow of its name"))?;
    let idx = |i: usize| crate::dungeon_credit::idx(i);
    // Opening the Hookshot chest: Event201 marks it opened.
    let opened: usize = named(f, "Event201")?;
    if !(is(&f.events[opened], "GameDataName", STATE) && is(&f.events[opened], "Value", "OpenedHookShot")) {
        return Err(Error::Format("one dampe race: Event201 no longer marks the Hookshot chest opened"));
    }
    // The race result's own appear steps, copied: IsAppearTreasureBox on (Event367), then the
    // move to the TreasureBoxAppear event (Event171).
    let on: usize = named(f, "Event367")?;
    if !(is(&f.events[on], "GameDataName", APPEAR) && param(&f.events[on], "Value").is_some()) {
        return Err(Error::Format("one dampe race: Event367 no longer turns IsAppearTreasureBox on"));
    }
    let show: usize = named(f, "Event171")?;
    if !is(&f.events[show], "EventName", SHOW) {
        return Err(Error::Format("one dampe race: Event171 no longer moves to TreasureBoxAppear"));
    }
    let at: u16 = idx(f.events.len())?;
    let mut set_on: Event = f.events[on].clone();
    set_on.name = "OOTRRHeartChestOn".to_string();
    if let Kind::Action { next, .. } = &mut set_on.kind {
        *next = at + 1;
    }
    let mut go: Event = f.events[show].clone();
    go.name = "OOTRRHeartChestShow".to_string();
    if let Kind::Action { next, .. } = &mut go.kind {
        *next = crate::bfevfl::flow::NONE;
    }
    f.events.push(set_on);
    f.events.push(go);
    let e: &mut Event = &mut f.events[opened];
    if let Some(c) = e.kind.params_mut() {
        for (k, v) in c.items.iter_mut() {
            if k == "Value" {
                *v = Val::Str("AppearHeartPiece".to_string());
            }
        }
    }
    match &mut e.kind {
        Kind::Action { next, .. } => *next = at,
        _ => return Err(Error::Format("one dampe race: Event201 is not an action")),
    }
    Ok(())
}
