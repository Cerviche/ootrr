use crate::bfevfl::{Bf, Event, Flow, Kind, Val};
use crate::error::{Error, Res};

pub const SETTING: &str = "faster_ocarina_minigame";
pub const NUM: u16 = 165;
pub const FLOW: &str = "Npc_SkullKid_Child_02";
pub const ENTRY: &str = "SessionStart";
pub const GRADE: &str = "Minigame_SkullKidSession_CurrentGradeSoundNumForSession";
pub const LAST: i32 = 8;
pub const HEART: &str = "Minigame_SkullKidSession_IsGetHeartPiece";
const SET_INT: &str = "EventTriggerSetGameDataInt";
const BOOL_QUERY: &str = "EventQueryIsGameDataBool";
const DATA: &str = "GameDataActor";
const GET: &str = "Dm_SY_GetInEvent";
const CLOSE: &str = "UISystemActor.EventCloseRupeeDisplay";
const NO: u32 = 1;

pub const NOTE: &str = "Faster Ocarina minigame: the Skull Kids in the Lost Woods start straight at the last round (8 notes), and clearing it the first time gives all three prizes in a row: the Blue Rupee, the Red Rupee, then the check. SessionStart in Npc_SkullKid_Child_02 sets CurrentGradeSoundNumForSession to 8 first, and the not-yet-won branch of the Heart Piece check runs the Blue Rupee and Red Rupee gets before it. Experimental, untested in the game.";

fn named(f: &Flow, name: &str) -> Res<usize> {
    let hits: Vec<usize> = (0..f.events.len()).filter(|&i| f.events[i].name == name).collect();
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("faster ocarina minigame: expected one {name} in {FLOW}, found {}", hits.len()))),
    }
}

fn call(f: &Flow, e: &Event) -> Option<String> {
    match &e.kind {
        Kind::Switch { actor, query, .. } => f.actors.get(*actor as usize).and_then(|a| a.queries.get(*query as usize).map(|q| format!("{}.{q}", a.name))),
        Kind::Action { actor, act, .. } => f.actors.get(*actor as usize).and_then(|a| a.actions.get(*act as usize).map(|q| format!("{}.{q}", a.name))),
        _ => None,
    }
}

fn param<'a>(e: &'a Event, k: &str) -> Option<&'a Val> {
    e.kind.params().and_then(|c| c.get(k))
}

/// A rupee get: the GetInEvent sub giving `rupee`, followed by the rupee display closing.
fn rupee(f: &Flow, name: &str, rupee: &str) -> Res<(usize, usize)> {
    let i: usize = named(f, name)?;
    let e: &Event = &f.events[i];
    let next: u16 = match &e.kind {
        Kind::Sub { next, flow, .. } if flow == GET && param(e, "ActorName") == Some(&Val::Str(rupee.to_string())) => *next,
        _ => return Err(Error::Msg(format!("faster ocarina minigame: {name} no longer gives {rupee}, the game files differ"))),
    };
    let c: usize = next as usize;
    if f.events.get(c).and_then(|x| call(f, x)).as_deref() != Some(CLOSE) {
        return Err(Error::Msg(format!("faster ocarina minigame: {name} is not followed by the rupee display closing")));
    }
    Ok((i, c))
}

fn set_next(e: &mut Event, to: u16) {
    if let Kind::Action { next, .. } = &mut e.kind {
        *next = to;
    }
}

pub fn patch(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == FLOW).ok_or(Error::Format("faster ocarina minigame: Npc_SkullKid_Child_02 has no flow of its name"))?;
    let idx = |i: usize| crate::dungeon_credit::idx(i);
    // The Heart Piece check (Event64): its not-yet-won case leads to the check's get.
    let ask: usize = named(f, "Event64")?;
    let ok: bool = call(f, &f.events[ask]).as_deref() == Some(&format!("{DATA}.{BOOL_QUERY}")) && param(&f.events[ask], "GameDataName") == Some(&Val::Str(HEART.to_string()));
    if !ok {
        return Err(Error::Format("faster ocarina minigame: Event64 no longer asks for the Heart Piece flag"));
    }
    let check: u16 = match &f.events[ask].kind {
        Kind::Switch { cases, .. } => cases.iter().find(|c| c.val == NO).map(|c| c.ev),
        _ => None,
    }
    .ok_or(Error::Format("faster ocarina minigame: Event64 has no not-yet-won case"))?;
    // Blue Rupee (first round prize) then Red Rupee (second round prize), then the check.
    let (blue, blue_close): (usize, usize) = rupee(f, "Event22", "Obj_Rupee_Blue")?;
    let (red, red_close): (usize, usize) = rupee(f, "Event23", "Obj_Rupee_Red")?;
    set_next(&mut f.events[blue_close], idx(red)?);
    set_next(&mut f.events[red_close], check);
    if let Kind::Switch { cases, .. } = &mut f.events[ask].kind {
        for c in cases.iter_mut().filter(|c| c.val == NO) {
            c.ev = idx(blue)?;
        }
    }
    // Every session starts at the last round.
    let (a, set): (u16, u16) = crate::dungeon_credit::action_slot(f, DATA, SET_INT)?;
    let e: usize = f.entries.iter().position(|e| e.name == ENTRY).ok_or(Error::Format("faster ocarina minigame: no SessionStart entry"))?;
    let at: u16 = idx(f.events.len())?;
    let params = crate::bfevfl::Cont {
        items: vec![
            ("GameDataName".to_string(), Val::Str(GRADE.to_string())),
            ("Index".to_string(), Val::Int(-1)),
            ("Value".to_string(), Val::Int(LAST)),
        ],
    };
    f.events.push(Event { name: "OOTRRLastRound".to_string(), kind: Kind::Action { next: f.entries[e].main, actor: a, act: set, params: Some(params) } });
    f.entries[e].main = at;
    Ok(())
}
