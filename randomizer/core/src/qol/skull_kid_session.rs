use crate::bfevfl::{Bf, Event, Flow, Kind, Val};
use crate::error::{Error, Res};

pub const SETTING: &str = "faster_ocarina_minigame";
pub const NUM: u16 = 165;
pub const FLOW: &str = "Npc_SkullKid_Child_02";
pub const ENTRY: &str = "SessionStart";
pub const GRADE: &str = "Minigame_SkullKidSession_CurrentGradeSoundNumForSession";
pub const PLAYED: i32 = 6;
pub const LAST: i32 = 8;
pub const HEART: &str = "Minigame_SkullKidSession_IsGetHeartPiece";
const SET_INT: &str = "EventTriggerSetGameDataInt";
const BOOL_QUERY: &str = "EventQueryIsGameDataBool";
const DATA: &str = "GameDataActor";
const INT_QUERY: &str = "EventQueryIsGameDataInt";
const LESS: u32 = 0;

pub const NOTE: &str = "Faster Ocarina minigame: the Skull Kids in the Lost Woods play only the 6-note game, and winning it gives the check. CurrentGradeSoundNumForSession starts at 6 (save default, and SessionStart in Npc_SkullKid_Child_02 sets it too), and clearing 6 goes to the Heart Piece branch instead of raising the game to 8 with the Red Rupee. The Blue Rupee and Red Rupee prizes of the 5 and 6 note games are gone. Experimental, untested in the game.";

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

pub fn default() -> crate::start_defaults::Edit {
    crate::start_defaults::Edit::Int(GRADE.to_string(), PLAYED)
}

pub fn patch(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == FLOW).ok_or(Error::Format("faster ocarina minigame: Npc_SkullKid_Child_02 has no flow of its name"))?;
    let idx = |i: usize| crate::dungeon_credit::idx(i);
    // The Heart Piece check (Event64), reached from the 8-note clear through Event68 (CurrentRallyCount = 1).
    let ask: usize = named(f, "Event64")?;
    let ok: bool = call(f, &f.events[ask]).as_deref() == Some(&format!("{DATA}.{BOOL_QUERY}")) && param(&f.events[ask], "GameDataName") == Some(&Val::Str(HEART.to_string()));
    if !ok {
        return Err(Error::Format("faster ocarina minigame: Event64 no longer asks for the Heart Piece flag"));
    }
    let way: usize = named(f, "Event68")?;
    let reaches: bool = matches!(&f.events[way].kind, Kind::Action { next, .. } if f.events.get(*next as usize).is_some_and(|e| matches!(e.kind, Kind::Action { next, .. } if next as usize == ask)));
    if !reaches {
        return Err(Error::Format("faster ocarina minigame: Event68 no longer leads to the Heart Piece check"));
    }
    // After a clear, Event10 asks whether the game is below 8 notes: it was the 6-note game, so
    // that case goes the 8-note way, to the check, and the game stays at 6 (no Red Rupee).
    let eight: usize = named(f, "Event10")?;
    let ok: bool = call(f, &f.events[eight]).as_deref() == Some(&format!("{DATA}.{INT_QUERY}")) && param(&f.events[eight], "GameDataName") == Some(&Val::Str(GRADE.to_string())) && param(&f.events[eight], "Value") == Some(&Val::Int(LAST));
    if !ok {
        return Err(Error::Format("faster ocarina minigame: Event10 no longer asks for the 8-note game"));
    }
    let to: u16 = idx(way)?;
    match &mut f.events[eight].kind {
        Kind::Switch { cases, .. } if cases.iter().any(|c| c.val == LESS) => {
            for c in cases.iter_mut().filter(|c| c.val == LESS) {
                c.ev = to;
            }
        }
        _ => return Err(Error::Format("faster ocarina minigame: Event10 has no below-8 case")),
    }
    // Saves from before the setting still hold 5: every session starts at 6.
    let (a, set): (u16, u16) = crate::dungeon_credit::action_slot(f, DATA, SET_INT)?;
    let e: usize = f.entries.iter().position(|e| e.name == ENTRY).ok_or(Error::Format("faster ocarina minigame: no SessionStart entry"))?;
    let at: u16 = idx(f.events.len())?;
    let params = crate::bfevfl::Cont {
        items: vec![
            ("GameDataName".to_string(), Val::Str(GRADE.to_string())),
            ("Index".to_string(), Val::Int(-1)),
            ("Value".to_string(), Val::Int(PLAYED)),
        ],
    };
    f.events.push(Event { name: "OOTRRSixNotes".to_string(), kind: Kind::Action { next: f.entries[e].main, actor: a, act: set, params: Some(params) } });
    f.entries[e].main = at;
    Ok(())
}
