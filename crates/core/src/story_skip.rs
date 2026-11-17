//! Story cutscenes skipped by writing what they would have written.
//!
//! - Deku Tree (with an open forest): the meeting (Dm_Kokiri_0070/0075) and the first arrival
//!   inside the tree (Dm_Deku_0000 plus Navi's dungeon lesson) are replaced by their writes at
//!   new game, so the mouth is open and the quest is where the arrival leaves it.
//! - Saria's gift: the bridge event goes straight to Dm_Kokiri_0155, the short gift
//!   scene that hands over the check's item, the way it does when the long Dm_Kokiri_0150 scene
//!   is skipped with the button. 0150 is marked as played there, which retires the trigger.
//! - Temple of Time as a child: the first entry and the altar scenes are marked as
//!   played at new game, with the flags the altar scene leaves behind.
//! - First Master Sword draw: the draw gives the sword and the item of Rauru's check,
//!   writes what the Rauru and Sheik scenes write, and hands over to the game's own later-draw
//!   event (Market_Child_Field_ChildToAdult), which swaps the equipment, makes Link an adult and
//!   loads the adult temple.
//!
//! Nothing here adds a call into another flow file. Such a call is only loaded when the event's
//! row in RSDB/GameAnalyzedEventInfo lists the file, and even with the row edited a get-item
//! call spliced into Saria's bridge event hung the game (2026-11-15, twice).

use std::fs;

use crate::bfevfl::{Bf, Cont, Event, Flow, Kind, Val};
use crate::error::{Error, Res};
use crate::qol::{edit_zs, Have};
use crate::romfs::Romfs;
use crate::staging::Out;
use crate::start_state::Ops;

pub const NOTE_DEKU: &str = "The Deku Tree is met at new game: its mouth is open, the Deku Tree quest is complete and the Forest Stone quest is where Link's first arrival inside the tree leaves it, so the meeting, the arrival scene and Navi's dungeon lesson are skipped.";
pub const NOTE: &str = "Story cutscenes are skipped. Saria's bridge scenes are cut: crossing the bridge gives the check's item with a plain item get window. The Temple of Time's first entry and altar scenes are marked as played. The first Master Sword draw gives the sword and the item of Rauru's check and then plays the short scene of a later draw, without the sword pull, Ganondorf, Rauru or Sheik scenes. Zelda's escape at the drawbridge is cut, so the Ocarina of Time waits in the moat right away. The Light Arrows scene only plays the short scene that hands over its check, and it no longer takes out the Master Sword, so Link keeps his equipment. The owls that stop Link (Hyrule Field, Zora's River, the Lost Woods) are cut and marked as met; the Death Mountain and Lake Hylia owls still offer their flights when spoken to. The burning village and Sheik's flashback before the Nocturne are cut: the village loads once, already burned, straight into the Nocturne scene, and the scene no longer takes out the Master Sword. Sheik's song scenes go straight to the song: the talk and the three ocarina lessons before it and the talk after it are cut, and the song check is given as usual.";

const TRUE: u32 = 0x8000_0001;
const FALSE: u32 = 0;
const NONE: u16 = 0xFFFF;

const DATA_ACTOR: &str = "GameDataActor";
const SET_BOOL: &str = "EventTriggerSetGameDataBool";
const CHALLENGE_ACTOR: &str = "ChallengeSystemActor";
const PROGRESS: &str = "EventExecuteProgressChallengeStep";

// ---------------------------------------------------------------- Deku Tree (open forest)

/// Steps after the anchor (Meet_DekuTree, which the open forest already moves to Step03):
/// the meeting completes the Deku Tree quest and opens the Forest Stone quest, the first
/// arrival inside the tree moves it to MeetBoss.
pub const DEKU_STEPS: [(&str, &str); 3] = [("Meet_DekuTree", "Complete"), ("Get_StoneOfForest", "GoInside"), ("Get_StoneOfForest", "MeetBoss")];
pub const DEKU_ENUMS: [(&str, &str); 2] = [("Step_Meet_DekuTree", "Complete"), ("Step_Get_StoneOfForest", "MeetBoss")];
/// The mouth flag is what Dm_Kokiri_0075 writes (the tree model reads it). The dungeon entrance
/// plays its arrival only while Dm_Deku_0000 is unplayed; that arrival also runs Navi's dungeon
/// lesson, which gates the D-pad Navi talk, so the lesson's markers are written too (the same
/// four the Kakariko state writes).
pub const DEKU_FLAGS: [&str; 8] = [
    "KokiriForest_Meet_DekuTree_OpenDekuTreeMouth",
    "HavePlayedEvent.Dm_Kokiri_0070",
    "HavePlayedEvent.Dm_Kokiri_0075",
    "HavePlayedEvent.Dm_Deku_0000",
    "HavePlayedEvent.NaviHint_Trial",
    "HaveTalked.Npc_Navi",
    "ActionGuide_IsEnd.Navi",
    "ActionGuide_IsNew.Navi",
];

pub fn deku() -> Ops {
    Ops {
        steps: DEKU_STEPS.iter().map(|(c, s)| ((*c).to_string(), (*s).to_string())).collect(),
        enums: DEKU_ENUMS.iter().map(|(c, s)| ((*c).to_string(), (*s).to_string())).collect(),
        flags: DEKU_FLAGS.iter().map(|n| ((*n).to_string(), true)).collect(),
        ..Ops::default()
    }
}

// ---------------------------------------------------------------- new game state

/// The child Temple of Time: each trigger area exists only while its scene is unplayed. The
/// altar scene's flow clears the first-time light shaft and its area's logic sets the found flag.
pub const TEMPLE_FLAGS: [&str; 4] = [
    "HavePlayedEvent.Dm_TimeTemple_0000",
    "HavePlayedEvent.Dm_TimeTemple_0020",
    "Market_Child_TimeTemple_DeleteLightShaft_FirstTime_00",
    "Market_Child_FoundMasterSword",
];

/// Owls the maps start on their own while the scene is unplayed (Hyrule Field near the castle,
/// toward Gerudo Valley and toward Lake Hylia, Zora's River after Saria's Song). They write nothing
/// but their played flag. The Death Mountain and Lake Hylia flight owls only talk when spoken to
/// and are left alone, as are the dream owl (it opens the dream world) and the owl after the
/// Spirit Temple (other flows branch on its scene).
pub const OWL_FLAGS: [&str; 4] = [
    "HavePlayedEvent.Dm_HrlField_0030",
    "HavePlayedEvent.Dm_HrlField_0040",
    "HavePlayedEvent.Dm_HrlField_0050",
    "HavePlayedEvent.Dm_ZoraRiver_0005",
];

/// The always-on new game flags: the child Temple of Time and the owls.
pub fn start() -> Ops {
    Ops { flags: TEMPLE_FLAGS.iter().chain(OWL_FLAGS.iter()).map(|n| ((*n).to_string(), true)).collect(), ..Ops::default() }
}

// ---------------------------------------------------------------- flow helpers

fn flag(v: bool) -> Val {
    Val::Bool(if v { TRUE } else { FALSE })
}

fn item(k: &str, v: Val) -> (String, Val) {
    (k.to_string(), v)
}

fn idx(n: usize) -> Res<u16> {
    if n >= NONE as usize {
        return Err(Error::Format("story skip: too many events in the flow"));
    }
    Ok(n as u16)
}

fn find(f: &Flow, name: &str) -> Res<usize> {
    f.events
        .iter()
        .position(|e| e.name == name)
        .ok_or_else(|| Error::Msg(format!("story skip: {} has no event {name}, the game files differ", f.name)))
}

fn text<'a>(e: &'a Event, k: &str) -> Option<&'a str> {
    match e.kind.params()?.get(k)? {
        Val::Str(s) => Some(s.as_str()),
        _ => None,
    }
}

/// The action an event runs, as `actor.action`.
fn action(f: &Flow, i: usize) -> Option<String> {
    let Kind::Action { actor, act, .. } = &f.events[i].kind else {
        return None;
    };
    let a = f.actors.get(*actor as usize)?;
    Some(format!("{}.{}", a.name, a.actions.get(*act as usize)?))
}

fn next_of(k: &Kind) -> Option<u16> {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => Some(*next),
        _ => None,
    }
}

fn set_next(k: &mut Kind, to: u16) -> Res<()> {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => {
            *next = to;
            Ok(())
        }
        _ => Err(Error::Format("story skip: the event has no single next")),
    }
}

/// Appends a chain of events, each pointing at the following one and the last at `then`.
/// Returns the index of the first.
fn chain(f: &mut Flow, prefix: &str, kinds: Vec<Kind>, then: u16) -> Res<u16> {
    let base: usize = f.events.len();
    idx(base + kinds.len())?;
    let n: usize = kinds.len();
    for (i, mut k) in kinds.into_iter().enumerate() {
        set_next(&mut k, if i + 1 < n { idx(base + i + 1)? } else { then })?;
        f.events.push(Event { name: format!("{prefix}{}", base + i), kind: k });
    }
    idx(base)
}

fn set_bool(f: &mut Flow, name: &str, v: bool) -> Res<Kind> {
    let (a, x) = crate::start_state::slot(f, DATA_ACTOR, SET_BOOL)?;
    Ok(Kind::Action { next: NONE, actor: a, act: x, params: Some(Cont { items: vec![item("Index", Val::Int(-1)), item("GameDataName", Val::Str(name.to_string())), item("Value", flag(v))] }) })
}

fn step(f: &mut Flow, quest: &str, step: &str) -> Res<Kind> {
    let (a, x) = crate::start_state::slot(f, CHALLENGE_ACTOR, PROGRESS)?;
    Ok(Kind::Action {
        next: NONE,
        actor: a,
        act: x,
        params: Some(Cont {
            items: vec![
                item("ChallengeName", Val::Str(quest.to_string())),
                item("StepName", Val::Str(step.to_string())),
                item("IsIgnoreCompletedChallenge", flag(false)),
                item("IsIgnoreChallengeProgressUI", flag(true)),
                item("IsWaitEnd", flag(true)),
            ],
        }),
    })
}

fn guard(f: &Flow, prefix: &str) -> Res<()> {
    if f.events.iter().any(|e| e.name.starts_with(prefix)) {
        return Err(Error::Msg(format!("story skip: {} was already patched", f.name)));
    }
    Ok(())
}

fn flow_mut<'a>(b: &'a mut Bf, name: &str) -> Res<&'a mut Flow> {
    b.flows.iter_mut().find(|f| f.name == name).ok_or_else(|| Error::Msg(format!("story skip: no flow {name}")))
}

fn rebuild(raw: &[u8], edit: impl FnOnce(&mut Bf) -> Res<()>) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    edit(&mut b)?;
    let out: Vec<u8> = b.write()?;
    if Bf::parse(&out)? != b {
        return Err(Error::Format("story skip: the patched flow does not parse back to the edited model"));
    }
    Ok(out)
}

/// The string parameter `param` of event `index` (checked by name) in a flow file.
fn param_of(raw: &[u8], index: usize, event: &str, param: &str) -> Res<String> {
    let b: Bf = Bf::parse(raw)?;
    let f: &Flow = b.flows.first().ok_or(Error::Format("story skip: the file holds no flow"))?;
    let e: &Event = f.events.get(index).ok_or_else(|| Error::Msg(format!("story skip: {} has no event {index}", f.name)))?;
    if e.name != event {
        return Err(Error::Msg(format!("story skip: event {index} of {} is {} and not {event}, the game files differ", f.name, e.name)));
    }
    text(e, param).map(str::to_string).ok_or_else(|| Error::Msg(format!("story skip: {event} of {} has no {param}", f.name)))
}

fn copy(f: &Flow, name: &str, want: &str) -> Res<Kind> {
    let i: usize = find(f, name)?;
    let k: Kind = f.events[i].kind.clone();
    let ok: bool = match &k {
        Kind::Sub { entry, .. } => entry == want,
        Kind::Action { .. } => action(f, i).as_deref() == Some(want),
        _ => false,
    };
    if !ok {
        return Err(Error::Msg(format!("story skip: {name} in {} is not {want}, the game files differ", f.name)));
    }
    Ok(k)
}

fn set_param(k: &mut Kind, key: &str, v: Val) -> Res<()> {
    let c: &mut Cont = k.params_mut().ok_or(Error::Format("story skip: the event has no parameters"))?;
    let slot: &mut Val = c.items.iter_mut().find(|(n, _)| n == key).map(|(_, v)| v).ok_or_else(|| Error::Msg(format!("story skip: the event has no {key}")))?;
    *slot = v;
    Ok(())
}

// ---------------------------------------------------------------- Saria's gift

pub const BRIDGE_FILE: &str = "Event/EventFlow/Goto_HyruleCastle.bfevfl.zs";
pub const BRIDGE_FLOW: &str = "Goto_HyruleCastle";
pub const BRIDGE_PREFIX: &str = "OOTRRSaria";
/// KokiriForest_ArriveLostWood: sound ducking -> Dm_Kokiri_0150 -> (skipped? fade out) ->
/// sound release -> Dm_Kokiri_0155 (the gift) -> ...
const DUCKING: &str = "Event58";
const LONG_SCENE: &str = "Event43";
const RELEASE: &str = "Event115";
const GIFT_SCENE: &str = "Event44";
/// The bridge trigger (KokiriForest_Child_Level_All_02_ddcd) is deleted once Dm_Kokiri_0150 has
/// played. Skipping the transition means the game never marks it, so the flow does.
pub const LONG_FLAG: &str = "HavePlayedEvent.Dm_Kokiri_0150";

/// The ducking now leads to a write of the 0150 flag and then to the sound release, which goes
/// on to the gift scene. Both scenes stay in the event's preload row.
pub fn patch_bridge(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, BRIDGE_FLOW)?;
    guard(f, BRIDGE_PREFIX)?;
    let duck: usize = find(f, DUCKING)?;
    let long: usize = find(f, LONG_SCENE)?;
    let release: usize = find(f, RELEASE)?;
    let gift: usize = find(f, GIFT_SCENE)?;
    if text(&f.events[long], "EventName") != Some("Dm_Kokiri_0150") || text(&f.events[gift], "EventName") != Some("Dm_Kokiri_0155") {
        return Err(Error::Format("story skip: the bridge scenes are not where they were, the game files differ"));
    }
    if next_of(&f.events[duck].kind) != Some(idx(long)?) || next_of(&f.events[release].kind) != Some(idx(gift)?) {
        return Err(Error::Format("story skip: the bridge flow no longer runs ducking -> 0150 and release -> 0155"));
    }
    let k: Kind = set_bool(f, LONG_FLAG, true)?;
    let first: u16 = chain(f, BRIDGE_PREFIX, vec![k], idx(release)?)?;
    set_next(&mut f.events[duck].kind, first)?;
    Ok(())
}

/// Ported from Ontico's REALAZK/ootrr pull request 11 ("Let Saria's gift hold any pouch item").
///
/// The short gift scene. Its C01 cut shows the get window (Event77) and then, inside a fork,
/// adds the item (Event68); the location patcher points both at the check's item.
pub const GIFT_FILE: &str = "Event/EventFlow/Dm_Kokiri_0155.bfevfl.zs";
pub const GIFT_FLOW: &str = "Dm_Kokiri_0155";
pub const GIFT_ENTRY: &str = "EntryPoint0";
const GIFT_GRANT: &str = "Event68";
const GIFT_SHOW: &str = "Event77";
const POUCH_ADD: &str = "PouchSystemActor.EventTriggerAddOrRemovePouchContent";

/// Turns the gift scene into a plain item get: the entry point runs the grant, then the get
/// window, then ends, with no cuts, camera or animation.
///
/// The gift stays in its own scene file on purpose. A get-item call spliced into the bridge
/// event (Goto_HyruleCastle) hung the game twice, even with its preload row edited. This scene
/// already calls GetPouchContent:DisplayGetByActorName, so nothing new has to be loaded.
///
/// The window runs before the grant, as in the scene's own C01 cut. A progressive item placed
/// here has both events wrapped into tier checks (crate::progressive::wrap_grant, before this
/// patch runs), and both checks then see the pouch before the grant, so they pick the same tier.
pub fn patch_gift(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, GIFT_FLOW)?;
    let grant: usize = find(f, GIFT_GRANT)?;
    let show: usize = find(f, GIFT_SHOW)?;
    let wrapped = |i: usize| matches!(f.events[i].kind, Kind::Switch { .. });
    if !wrapped(grant) && action(f, grant).as_deref() != Some(POUCH_ADD) {
        return Err(Error::Format("story skip: the gift scene's grant is not a pouch add, the game files differ"));
    }
    if !wrapped(show) && !matches!(&f.events[show].kind, Kind::Sub { flow, entry, .. } if flow == "GetPouchContent" && entry == "DisplayGetByActorName") {
        return Err(Error::Format("story skip: the gift scene's get window is not GetPouchContent:DisplayGetByActorName, the game files differ"));
    }
    let (g, s): (u16, u16) = (idx(grant)?, idx(show)?);
    // C01 holds the window's sub calls (one, or one per tier once wrapped).
    let subs: Vec<u16> = f.entries.iter().find(|e| e.name == "C01").map(|e| e.subs.clone()).ok_or(Error::Format("story skip: the gift scene has no C01 cut"))?;
    let e: &mut crate::bfevfl::Entry = f.entries.iter_mut().find(|e| e.name == GIFT_ENTRY).ok_or(Error::Format("story skip: the gift scene has no EntryPoint0"))?;
    if e.main == s {
        return Err(Error::Format("story skip: the gift scene was already patched"));
    }
    e.main = s;
    e.subs = subs;
    let after: u16 = ends(f, show)?;
    retarget(f, show, after, g)?;
    retarget(f, grant, ends(f, grant)?, NONE)?;
    Ok(())
}

/// The single `next` every path from `start` ends on: the event's own next, or, through a
/// tier check, the shared next of its tier copies.
fn ends(f: &Flow, start: usize) -> Res<u16> {
    let mut seen: Vec<u16> = Vec::new();
    let mut todo: Vec<usize> = vec![start];
    while let Some(i) = todo.pop() {
        match &f.events[i].kind {
            Kind::Switch { cases, .. } => todo.extend(cases.iter().map(|c| c.ev as usize)),
            k => {
                let n: u16 = next_of(k).ok_or(Error::Format("story skip: a gift event has no single next"))?;
                if !seen.contains(&n) {
                    seen.push(n);
                }
            }
        }
    }
    match seen.as_slice() {
        [n] => Ok(*n),
        _ => Err(Error::Msg(format!("story skip: the gift event {} does not end on one next ({seen:?})", f.events[start].name))),
    }
}

/// Points every path from `start` that ends on `from` at `to` instead.
fn retarget(f: &mut Flow, start: usize, from: u16, to: u16) -> Res<()> {
    let mut todo: Vec<usize> = vec![start];
    while let Some(i) = todo.pop() {
        if let Kind::Switch { cases, .. } = &f.events[i].kind {
            todo.extend(cases.iter().map(|c| c.ev as usize));
        } else if next_of(&f.events[i].kind) == Some(from) {
            set_next(&mut f.events[i].kind, to)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- first Master Sword draw

pub const DOOR_FILE: &str = "Event/EventFlow/DoorOfTime.bfevfl.zs";
pub const DOOR_FLOW: &str = "DoorOfTime";
pub const DOOR_PREFIX: &str = "OOTRRDraw";
/// DoorOfTime_Draw: true when Ganondorf's scene has played (a later draw), false on the first.
const DRAW_SWITCH: &str = "Event23";
/// The later-draw branch: a transition to Market_Child_Field_ChildToAdult.
const LATER_DRAW: &str = "Event22";
const GIVE_SWORD: &str = "Event115";
/// The Recovery sub that follows the adult temple load in Rauru's path; door_fix guards the
/// quest completion after it.
const RECOVERY: &str = "Event102";
pub const LATER_EVENT: &str = "Market_Child_Field_ChildToAdult";
/// Rauru's grant, which the location patcher points at the check's item.
pub const RAURU_FILE: &str = "Event/EventFlow/Dm_SageRoom_0005.bfevfl.zs";
pub const RAURU_INDEX: usize = 182;
pub const RAURU_EVENT: &str = "Event48";
pub const RAURU_PARAM: &str = "ActorName";
/// Every later draw tells itself apart from the first by Dm_TimeTemple_0040. The rest are the
/// skipped scenes (Sheik's altar talk 0065 would start while the sages quest is at Step01) and
/// what Market_Adult_AllSage_Start writes after Sheik's scenes.
pub const DRAW_FLAGS: [&str; 9] = [
    "HavePlayedEvent.Dm_TimeTemple_0030",
    "HavePlayedEvent.Dm_TimeTemple_0040",
    "HavePlayedEvent.Dm_SageRoom_0000",
    "HavePlayedEvent.Dm_SageRoom_0005",
    "HavePlayedEvent.Dm_TimeTemple_0050",
    "HavePlayedEvent.Dm_TimeTemple_0055",
    "HavePlayedEvent.Dm_TimeTemple_0065",
    "Market_Child_Firsttime_GotoAdult",
    "HavePlayedEvent.NaviHint_Shield_Hylian",
];
/// The quest steps Market_Adult_AllSage_Start writes, in its order.
pub const DRAW_STEPS: [(&str, &str); 3] = [("SageOfLight", "Complete"), ("AllSage", "Step01"), ("SageOfForest", "Step01")];

/// The first draw went pull scene -> Chamber of Sages (Ganondorf, the sword, the era change)
/// -> Chamber of Sages (Rauru, his check) -> adult temple (Sheik, quest steps). It now writes
/// the flags, gives the sword and `medal` (plus its IsGet flag `got`, which a pouch add without
/// the get event may leave unset), writes the quest steps and transitions to the later-draw
/// event, which the draw's own event table row already lists.
///
/// The transition is the last thing the draw does, as in the later draw. A first version ran
/// Rauru's Recovery sub (a wait and a heal) after it and the game crashed while the child temple
/// was reloaded with Link already an adult (2026-11-15). The sword is not equipped directly
/// either: the era swap equips it for the adult, as on every later draw. The guarded quest
/// completion that followed Recovery (door_fix) runs before the transition instead.
pub fn patch_draw(b: &mut Bf, medal: &str, got: Option<&str>) -> Res<()> {
    let f: &mut Flow = flow_mut(b, DOOR_FLOW)?;
    guard(f, DOOR_PREFIX)?;
    let sw: usize = find(f, DRAW_SWITCH)?;
    let Kind::Switch { cases, params, .. } = &f.events[sw].kind else {
        return Err(Error::Format("story skip: the draw check is not a switch"));
    };
    if params.as_ref().and_then(|c| c.get("EventName")) != Some(&Val::Str("Dm_TimeTemple_0040".to_string())) || cases.len() != 2 {
        return Err(Error::Format("story skip: the draw check does not test Dm_TimeTemple_0040, the game files differ"));
    }
    let later: usize = find(f, LATER_DRAW)?;
    if text(&f.events[later], "EventName") != Some(LATER_EVENT) || action(f, later).as_deref() != Some("EventSystemActor.EventTransition") {
        return Err(Error::Format("story skip: the later draw no longer transitions to Market_Child_Field_ChildToAdult"));
    }
    let mut kinds: Vec<Kind> = Vec::new();
    for n in DRAW_FLAGS {
        kinds.push(set_bool(f, n, true)?);
    }
    let mut sword: Kind = copy(f, GIVE_SWORD, "PouchSystemActor.EventTriggerAddOrRemovePouchContent")?;
    if text(&f.events[find(f, GIVE_SWORD)?], "Actor") != Some("Weapon_Sword_Master") {
        return Err(Error::Format("story skip: the Ganondorf path no longer gives the Master Sword"));
    }
    set_param(&mut sword, "IsEquipDIrect", flag(false))?;
    let mut gift: Kind = sword.clone();
    set_param(&mut gift, "Actor", Val::Str(medal.to_string()))?;
    kinds.push(sword);
    kinds.push(gift);
    if let Some(n) = got {
        kinds.push(set_bool(f, n, true)?);
    }
    for (q, s) in DRAW_STEPS {
        kinds.push(step(f, q, s)?);
    }
    // The tail: [guard ->] [DoorOfTime Complete ->] transition (last, no next).
    let mut go: Kind = f.events[later].kind.clone();
    set_next(&mut go, NONE)?;
    let t: u16 = chain(f, DOOR_PREFIX, vec![go], NONE)?;
    let rec: usize = find(f, RECOVERY)?;
    if !matches!(&f.events[rec].kind, Kind::Sub { entry, .. } if entry == "Recovery") {
        return Err(Error::Format("story skip: Event102 is not the Recovery call, the game files differ"));
    }
    let after: u16 = next_of(&f.events[rec].kind).ok_or(Error::Format("story skip: the Recovery call has no next"))?;
    let is_complete = |f: &Flow, i: usize| text(&f.events[i], "ChallengeName") == Some("DoorOfTime") && text(&f.events[i], "StepName") == Some("Complete");
    let tail: u16 = match f.events.get(after as usize).map(|e| e.kind.clone()) {
        // door_fix's guard: case 0 (step06 or later) asks for Complete, the other case ends.
        Some(Kind::Switch { actor, query, cases, params }) => {
            let yes: u16 = cases.iter().find(|c| c.val == 0).map(|c| c.ev).ok_or(Error::Format("story skip: the quest guard has no true case"))?;
            if !is_complete(f, yes as usize) {
                return Err(Error::Format("story skip: the quest guard does not lead to the DoorOfTime Complete request"));
            }
            let mut done: Kind = f.events[yes as usize].kind.clone();
            set_next(&mut done, t)?;
            let c: u16 = chain(f, DOOR_PREFIX, vec![done], t)?;
            let cases = cases.iter().map(|x| crate::bfevfl::Case { val: x.val, ev: if x.val == 0 { c } else { t } }).collect();
            let g: u16 = idx(f.events.len())?;
            f.events.push(Event { name: format!("{DOOR_PREFIX}{g}"), kind: Kind::Switch { actor, query, cases, params } });
            g
        }
        Some(Kind::Action { .. }) if is_complete(f, after as usize) => {
            let mut done: Kind = f.events[after as usize].kind.clone();
            set_next(&mut done, t)?;
            chain(f, DOOR_PREFIX, vec![done], t)?
        }
        _ => return Err(Error::Format("story skip: Recovery is not followed by the DoorOfTime Complete request")),
    };
    let first: u16 = chain(f, DOOR_PREFIX, kinds, tail)?;
    let Kind::Switch { cases, .. } = &mut f.events[sw].kind else { unreachable!() };
    let c = cases.iter_mut().find(|c| c.val == 1).ok_or(Error::Format("story skip: the draw check has no false case"))?;
    c.ev = first;
    Ok(())
}

// ---------------------------------------------------------------- dungeon endings

pub const NOTE_ENDINGS: &str = "Dungeon ending cutscenes are shortened. After each dungeon the long scenes are cut and only the short scene that hands over the Spiritual Stone or Medallion check plays, so the reward (shuffled or not) is still shown. The Deku Tree's farewell, the Goron and Zora celebrations, Darunia's talk after he hands over the Goron's Ruby, each sage's awakening and parting words on the white screen after the medallion, the forest farewell, the village scene after the Fire Temple and Lake Hylia refilling are cut, and their scenes are marked as played.";
pub const ENDING_FILE: &str = "Event/EventFlow/Dm_SY_BancChange_EscapeDungeon.bfevfl.zs";
pub const ENDING_FLOW: &str = "Dm_SY_BancChange_EscapeDungeon";
pub const ENDING_PREFIX: &str = "OOTRREnding";
/// The scenes cut from the dungeon ending events in that flow. Each is a pre-rendered timeline
/// (the button skip goes past them too) with no flag, quest or item write of its own, except
/// Dm_Kokiri_0080 and Dm_Kokiri_0162, scripted scenes that only talk. The scene that hands over
/// the reward stays in each chain (Dm_Kokiri_0130, Dm_DeathMtTrail_0025, Dm_ZoraFountain_0030,
/// Dm_Awake_*_Get), as do the quest steps and flags the events write around them.
pub const CUT_SCENES: [&str; 16] = [
    // Deku Tree: his last words, farewell and death, and Link leaving the forest.
    "Dm_Kokiri_0080",
    "Dm_Kokiri_0095",
    "Dm_Kokiri_0090",
    "Dm_Kokiri_0140",
    // Dodongo's Cavern: the Goron celebration.
    "Dm_DeathMtTrail_0020",
    // Jabu-Jabu: Ruto at the fountain.
    "Dm_ZoraFountain_0020",
    // Forest Temple: Saria awakens, then the forest scenes outside.
    "Dm_Awake_Forest",
    "Dm_Kokiri_0158",
    "Dm_Kokiri_0160",
    "Dm_Kokiri_0162",
    // Fire Temple: Kakariko and Darunia awakens.
    "Dm_Kakariko_0025",
    "Dm_Awake_Fire",
    // Water Temple: Ruto awakens, Lake Hylia refills.
    "Dm_Awake_Water",
    "Dm_LakeHrl_0020",
    // Shadow and Spirit Temples: Impa and Nabooru awaken.
    "Dm_Awake_Shadow",
    "Dm_Awake_Spirit",
];

/// Turns every transition to a cut scene into a write of that scene's played flag, keeping its
/// next. Several maps read these flags, so they are set as if the scene had played.
pub fn patch_endings(b: &mut Bf) -> Res<usize> {
    let f: &mut Flow = flow_mut(b, ENDING_FLOW)?;
    guard(f, ENDING_PREFIX)?;
    let mut n: usize = 0;
    for scene in CUT_SCENES {
        let hits: Vec<usize> = (0..f.events.len())
            .filter(|&i| action(f, i).as_deref() == Some("EventSystemActor.EventTransition") && text(&f.events[i], "EventName") == Some(scene))
            .collect();
        if hits.is_empty() {
            return Err(Error::Msg(format!("story skip: no ending plays {scene}, the game files differ")));
        }
        for i in hits {
            let next: u16 = next_of(&f.events[i].kind).ok_or(Error::Format("story skip: a transition has no next"))?;
            let mut k: Kind = set_bool(f, &format!("HavePlayedEvent.{scene}"), true)?;
            set_next(&mut k, next)?;
            f.events[i].kind = k;
            f.events[i].name = format!("{ENDING_PREFIX}{i}");
            n += 1;
        }
    }
    Ok(n)
}

/// After the Fire Temple boss, SageOfFire_ClearEvent plays Dm_FireTemple_0030 and changes to
/// KakarikoVillage_Adult booting FireTemple_SageOfFire_Kakariko, which preloads the Chamber of
/// Sages, plays Dm_Kakariko_0025 (the village scene) and changes to SageRoom booting
/// FireTemple_SageOfFire_SageRoom. With 0025 cut, that boot event reserved the next change while
/// the village was still loading and Link froze at the village entrance. The Kakariko stop is
/// removed: the clear event writes 0025's played flag, preloads the Chamber and changes straight
/// to it with the Chamber boot (a copy of the boot event's own change).
pub const FIRE_PREFIX: &str = "OOTRRFire";
const FIRE_EXIT: (&str, &str) = ("Event19", "Dm_FireTemple_0030");
const FIRE_TO_VILLAGE: (&str, &str) = ("Event30", "FireTemple_SageOfFire_Kakariko");
const FIRE_PRELOAD: (&str, &str) = ("Event298", "GameSystemActor.EventTriggerBancRequestBeforehandLoad");
const FIRE_TO_CHAMBER: (&str, &str) = ("Event35", "FireTemple_SageOfFire_SageRoom");
const BANC_CHANGE: &str = "GameSystemActor.EventReserveBancChange";

pub fn patch_fire_ending(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, ENDING_FLOW)?;
    guard(f, FIRE_PREFIX)?;
    let exit: usize = find_action(f, (FIRE_EXIT.0, "EventSystemActor.EventTransition"))?;
    if text(&f.events[exit], "EventName") != Some(FIRE_EXIT.1) {
        return Err(Error::Format("story skip: the Fire Temple exit no longer plays its scene, the game files differ"));
    }
    let village: usize = find_action(f, (FIRE_TO_VILLAGE.0, BANC_CHANGE))?;
    let chamber: usize = find_action(f, (FIRE_TO_CHAMBER.0, BANC_CHANGE))?;
    for (i, boot) in [(village, FIRE_TO_VILLAGE.1), (chamber, FIRE_TO_CHAMBER.1)] {
        if text(&f.events[i], "BootEventName") != Some(boot) {
            return Err(Error::Msg(format!("story skip: {} no longer boots {boot}, the game files differ", f.events[i].name)));
        }
    }
    if next_of(&f.events[exit].kind) != Some(idx(village)?) {
        return Err(Error::Format("story skip: the Fire Temple exit no longer leads to the village change"));
    }
    let preload: Kind = f.events[find_action(f, FIRE_PRELOAD)?].kind.clone();
    let mut to_chamber: Kind = f.events[chamber].kind.clone();
    set_next(&mut to_chamber, NONE)?;
    f.events[village].kind = to_chamber;
    f.events[village].name = format!("{FIRE_PREFIX}{village}");
    let flag: Kind = set_bool(f, "HavePlayedEvent.Dm_Kakariko_0025", true)?;
    let first: u16 = chain(f, FIRE_PREFIX, vec![flag, preload], idx(village)?)?;
    set_next(&mut f.events[exit].kind, first)?;
    Ok(())
}

/// The medallion scenes of the Chamber of Sages. Each gives its check, fades to white, restores
/// the boots, then waits a second, shows the sage's parting line(s) on the white screen and
/// waits about three more seconds (Forest and Fire then set AllSage_AwakeSage_Fire_Or_Forest).
pub const MEDAL_SCENES: [&str; 5] = ["Dm_Awake_Forest_Get", "Dm_Awake_Fire_Get", "Dm_Awake_Water_Get", "Dm_Awake_Shadow_Get", "Dm_Awake_Spirit_Get"];
const RESTORE_BOOTS: &str = "Player_Event.EventTriggerPlayerRestoreBootsChange";

/// Points the boots restore past the waits and talks that follow it, at the first event that is
/// neither (the flag write, or the end). Returns how many events were passed over.
pub fn cut_white_screen(b: &mut Bf) -> Res<usize> {
    let f: &mut Flow = b.flows.first_mut().ok_or(Error::Format("story skip: the medallion scene holds no flow"))?;
    let hits: Vec<usize> = (0..f.events.len()).filter(|&i| action(f, i).as_deref() == Some(RESTORE_BOOTS)).collect();
    let [r] = hits.as_slice() else {
        return Err(Error::Msg(format!("story skip: {} has {} boots restores, expected one", f.name, hits.len())));
    };
    let mut i: u16 = next_of(&f.events[*r].kind).ok_or(Error::Format("story skip: the boots restore has no next"))?;
    let mut cut: usize = 0;
    let mut talks: usize = 0;
    while i != NONE {
        let a: String = action(f, i as usize).unwrap_or_default();
        let skip: bool = a == "EventSystemActor.EventWait" || a.ends_with(".EventTalk");
        if !skip {
            break;
        }
        talks += usize::from(a.ends_with(".EventTalk"));
        cut += 1;
        i = next_of(&f.events[i as usize].kind).ok_or(Error::Format("story skip: a white screen event has no next"))?;
    }
    if talks == 0 {
        return Err(Error::Msg(format!("story skip: {} shows no line after the medallion, the game files differ", f.name)));
    }
    set_next(&mut f.events[*r].kind, i)?;
    Ok(cut)
}

pub fn medal_file(scene: &str) -> String {
    format!("Event/EventFlow/{scene}.bfevfl.zs")
}

/// Darunia's Goron Ruby scene. Its main entry runs its parts in order (sub :C01, :C01-05, :C01-07,
/// :C02 ... :C09); :C01-07 ends with the grant (the check), everything after is Darunia, the Gorons
/// and Navi talking, with no flag or item write, and :C09 closes with a listener reset, a fade out
/// and a one second wait. The grant part is pointed straight at that close.
pub const DARUNIA_FILE: &str = "Event/EventFlow/Dm_DeathMtTrail_0025.bfevfl.zs";
const DARUNIA_GRANT_PART: &str = "C01-07";
const LISTENER: &str = "SoundSystemActor.EventSoundChangeListenerPreset";
const FADE_OUT: &str = "UISystemActor.EventEventFadeOut";

pub fn cut_darunia_talk(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.first_mut().ok_or(Error::Format("story skip: Darunia's scene holds no flow"))?;
    let parts: Vec<usize> = (0..f.events.len()).filter(|&i| matches!(&f.events[i].kind, Kind::Sub { flow, entry, .. } if flow.is_empty() && entry == DARUNIA_GRANT_PART)).collect();
    let [part] = parts.as_slice() else {
        return Err(Error::Msg(format!("story skip: Darunia's scene runs {DARUNIA_GRANT_PART} {} times, expected once", parts.len())));
    };
    // The closing listener reset, the one whose next is the fade out.
    let close: Vec<usize> = (0..f.events.len())
        .filter(|&i| action(f, i).as_deref() == Some(LISTENER))
        .filter(|&i| next_of(&f.events[i].kind).is_some_and(|n| n != NONE && action(f, n as usize).as_deref() == Some(FADE_OUT)))
        .collect();
    let [close] = close.as_slice() else {
        return Err(Error::Msg(format!("story skip: Darunia's scene has {} closing fades, expected one", close.len())));
    };
    if next_of(&f.events[*part].kind) == Some(*close as u16) {
        return Err(Error::Format("story skip: Darunia's talk is already cut"));
    }
    set_next(&mut f.events[*part].kind, *close as u16)
}

// ---------------------------------------------------------------- Darunia's Joy

/// Darunia's Joy. Saria's Song to child Darunia runs Get_StoneOfFire entry
/// Get_StoneOfFire_PlayOcarina: when the melody is the one it asks for (EventQueryIsRecommendedMelody
/// case 0) it fades out, plays the dance timeline Dm_Goron_0025 (no flow, no writes), then reaches
/// the switch (Event39) that picks the bracelet scene Dm_Goron_0030 or Darunia's talk.
/// crate::open_world::darunia already makes that switch read HavePlayedEvent.Dm_Goron_0030, so the
/// gift is given once and does not follow the quest step. The melody case goes straight to that
/// switch, so the song leads to the item and a second song to the talk.
pub const STONE_FILE: &str = "Event/EventFlow/Get_StoneOfFire.bfevfl.zs";
const STONE_FLOW: &str = "Get_StoneOfFire";
const MELODY_QUERY: &str = "EventQueryIsRecommendedMelody";
const TRANSITION: &str = "EventSystemActor.EventTransition";
const DANCE: &str = "Dm_Goron_0025";
const BRACELET: &str = "Dm_Goron_0030";

/// The bracelet scene Dm_Goron_0030 runs its parts in a row of forks (sub :C01 to :C09, all talk,
/// dance and camera), then :C10 (the GetInEvent_Goron grant, the check) and :C11 (the bracelet
/// equip pose, Darunia back to his spot, the Get_StoneOfFire step DodongoCavernOpne and its
/// progress window). The main entry starts at the :C10 fork, and :C11 no longer plays the equip
/// pose, which only fits the Goron's Bracelet.
pub const BRACELET_FILE: &str = "Event/EventFlow/Dm_Goron_0030.bfevfl.zs";
const BRACELET_MAIN: &str = "EntryPoint0";
const BRACELET_GRANT_PART: &str = "C10";
const EQUIP_POSE: &str = "Gauntlet_Goron_Equip";

pub fn patch_darunia_song(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, STONE_FLOW)?;
    let to = |f: &Flow, scene: &str| -> Vec<usize> { (0..f.events.len()).filter(|&i| action(f, i).as_deref() == Some(TRANSITION) && text(&f.events[i], "EventName") == Some(scene)).collect() };
    let [dance] = to(f, DANCE)[..] else {
        return Err(Error::Msg(format!("story skip: {STONE_FLOW} moves to {DANCE} other than once")));
    };
    let [gift] = to(f, BRACELET)[..] else {
        return Err(Error::Msg(format!("story skip: {STONE_FLOW} moves to {BRACELET} other than once")));
    };
    // The switch that picks the bracelet scene: the gift happens once.
    let picks: Vec<usize> = (0..f.events.len()).filter(|&i| matches!(&f.events[i].kind, Kind::Switch { cases, .. } if cases.iter().any(|c| c.ev as usize == gift))).collect();
    let [pick] = picks[..] else {
        return Err(Error::Msg(format!("story skip: {} switches in {STONE_FLOW} lead to {BRACELET}, expected one", picks.len())));
    };
    let asks: Vec<usize> = (0..f.events.len())
        .filter(|&i| match &f.events[i].kind {
            Kind::Switch { actor, query, .. } => f.actors.get(*actor as usize).and_then(|a| a.queries.get(*query as usize)).is_some_and(|q| q == MELODY_QUERY),
            _ => false,
        })
        .collect();
    // The melody check whose right melody case leads (through a fade) to the dance.
    let mut hit: Option<usize> = None;
    for i in asks {
        let Kind::Switch { cases, .. } = &f.events[i].kind else {
            continue;
        };
        if let Some(c) = cases.iter().find(|c| c.val == 0) {
            let first: usize = c.ev as usize;
            if first == dance || next_of(&f.events[first].kind) == Some(dance as u16) {
                hit = Some(i);
            }
        }
    }
    let i: usize = hit.ok_or_else(|| Error::Msg(format!("story skip: no melody check in {STONE_FLOW} leads to {DANCE}")))?;
    let Kind::Switch { cases, .. } = &mut f.events[i].kind else {
        return Err(Error::Format("story skip: the melody check is no switch"));
    };
    for c in cases.iter_mut().filter(|c| c.val == 0) {
        c.ev = idx(pick)?;
    }
    Ok(())
}

pub fn cut_bracelet_scene(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, BRACELET)?;
    let parts: Vec<usize> = (0..f.events.len()).filter(|&i| matches!(&f.events[i].kind, Kind::Sub { flow, entry, .. } if flow.is_empty() && entry == BRACELET_GRANT_PART)).collect();
    let [part] = parts[..] else {
        return Err(Error::Msg(format!("story skip: {BRACELET} runs :{BRACELET_GRANT_PART} other than once")));
    };
    let fork: usize = (0..f.events.len())
        .find(|&i| matches!(&f.events[i].kind, Kind::Fork { forks, .. } if forks.contains(&(part as u16))))
        .ok_or_else(|| Error::Msg(format!("story skip: {BRACELET} runs :{BRACELET_GRANT_PART} outside a fork")))?;
    let e = f.entries.iter_mut().find(|e| e.name == BRACELET_MAIN).ok_or_else(|| Error::Msg(format!("story skip: {BRACELET} has no {BRACELET_MAIN}")))?;
    if e.main == idx(fork)? {
        return Err(Error::Format("story skip: the bracelet scene is already cut"));
    }
    e.main = idx(fork)?;
    let pose: Vec<u16> = (0..f.events.len()).filter(|&i| text(&f.events[i], "ASCommand") == Some(EQUIP_POSE)).map(idx).collect::<Res<Vec<u16>>>()?;
    if pose.is_empty() {
        return Err(Error::Msg(format!("story skip: {BRACELET} has no {EQUIP_POSE} pose")));
    }
    let mut dropped: usize = 0;
    for ev in f.events.iter_mut() {
        if let Kind::Fork { forks, .. } = &mut ev.kind {
            let n: usize = forks.len();
            forks.retain(|x| !pose.contains(x));
            dropped += n - forks.len();
        }
    }
    if dropped == 0 {
        return Err(Error::Msg(format!("story skip: no fork in {BRACELET} starts the {EQUIP_POSE} pose")));
    }
    Ok(())
}

pub fn stage_endings(rf: &mut Romfs, have: Have) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = vec![edit_zs(rf, have, ENDING_FILE, |raw| {
        rebuild(raw, |b| {
            patch_endings(b)?;
            patch_fire_ending(b)?;
            Ok(())
        })
    })?];
    // The location patcher may have staged these scenes already (their grant holds the check).
    outs.push(edit_zs(rf, have, DARUNIA_FILE, |raw| rebuild(raw, cut_darunia_talk))?);
    outs.push(edit_zs(rf, have, STONE_FILE, |raw| rebuild(raw, patch_darunia_song))?);
    // The location patcher stages the bracelet scene already (its grant holds the check).
    outs.push(edit_zs(rf, have, BRACELET_FILE, |raw| rebuild(raw, cut_bracelet_scene))?);
    for s in MEDAL_SCENES {
        // The location patcher may have staged the scene already (its grant holds the check).
        outs.push(edit_zs(rf, have, &medal_file(s), |raw| {
            rebuild(raw, |b| {
                cut_white_screen(b)?;
                Ok(())
            })
        })?);
    }
    Ok(outs)
}

pub fn worked_endings(rf: &mut Romfs, pk: &crate::modpack::Pack) -> Res<Vec<Out>> {
    let root: std::path::PathBuf = pk.work.join("romfs");
    let have = |p: &str| -> Option<Vec<u8>> { fs::read(p.split('/').fold(root.clone(), |a, s| a.join(s))).ok() };
    stage_endings(rf, &have)
}

// ---------------------------------------------------------------- light arrows

/// The Light Arrows scene lives in AllSage.bfevfl: entry Market_Adult_Field_GetArrowLight (the
/// adult Temple of Time area, AllSage Step02) and GetArrowLight_OcarinaBackTimeTemple (the Song
/// of Time warp, Dm_SY_Ocarina) both reach Event175, which forces the Master Sword out, then play
/// the timelines Dm_TimeTemple_0090/0095/0110/0115/0125 (Sheik is Zelda), the flowchart
/// Dm_TimeTemple_0130 (the Light Arrows check, nothing else) and the timeline 0135 (Ganondorf takes
/// Zelda), then Navi talks and the quests advance. The sword request becomes a flag write, the six
/// timelines become their played flags, and after the closing camera/Navi reset the chain fades in
/// and goes straight to the quest writes.
pub const ARROWS_FILE: &str = "Event/EventFlow/AllSage.bfevfl.zs";
pub const ARROWS_FLOW: &str = "AllSage";
pub const ARROWS_PREFIX: &str = "OOTRRArrows";
const ARROWS_SWORD: (&str, &str) = ("Event175", "Player_Event.EventTriggerEquipmentUserRequestCreateStaticEquipment");
const ARROWS_CUT: [(&str, &str); 6] = [
    ("Event12", "Dm_TimeTemple_0090"),
    ("Event13", "Dm_TimeTemple_0095"),
    ("Event14", "Dm_TimeTemple_0110"),
    ("Event16", "Dm_TimeTemple_0115"),
    ("Event63", "Dm_TimeTemple_0125"),
    ("Event42", "Dm_TimeTemple_0135"),
];
const ARROWS_GRANT: (&str, &str) = ("Event17", "Dm_TimeTemple_0130");
/// After 0135: Event116 (camera, Navi and Link reset fork) -> join -> Event179 (unequip) -> Event97 (fade in) -> Event10 (AllSage Complete).
const ARROWS_RESET: &str = "Event116";
const ARROWS_UNEQUIP: (&str, &str) = ("Event179", "Player_Event.EventPlayerUnequip");
const ARROWS_FADE_IN: (&str, &str) = ("Event97", "UISystemActor.EventEventFadeIn");
const ARROWS_DONE: (&str, &str) = ("Event10", "ChallengeSystemActor.EventExecuteProgressChallengeStep");

fn find_action(f: &Flow, (name, want): (&str, &str)) -> Res<usize> {
    let i: usize = find(f, name)?;
    if action(f, i).as_deref() != Some(want) {
        return Err(Error::Msg(format!("story skip: {name} in {} is not {want}, the game files differ", f.name)));
    }
    Ok(i)
}

/// Turns the transition `name` (checked to play `scene`) into a write of the scene's played flag,
/// keeping its next unless `to` is given.
fn transition_to_flag(f: &mut Flow, prefix: &str, name: &str, scene: &str, to: Option<u16>) -> Res<usize> {
    let i: usize = find_action(f, (name, "EventSystemActor.EventTransition"))?;
    if text(&f.events[i], "EventName") != Some(scene) {
        return Err(Error::Msg(format!("story skip: {name} in {} does not play {scene}, the game files differ", f.name)));
    }
    let next: u16 = match to {
        Some(t) => t,
        None => next_of(&f.events[i].kind).ok_or(Error::Format("story skip: a transition has no next"))?,
    };
    let mut k: Kind = set_bool(f, &format!("HavePlayedEvent.{scene}"), true)?;
    set_next(&mut k, next)?;
    f.events[i].kind = k;
    f.events[i].name = format!("{prefix}{i}");
    Ok(i)
}

pub fn patch_light_arrows(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, ARROWS_FLOW)?;
    guard(f, ARROWS_PREFIX)?;
    let grant: usize = find_action(f, (ARROWS_GRANT.0, "EventSystemActor.EventTransition"))?;
    if text(&f.events[grant], "EventName") != Some(ARROWS_GRANT.1) {
        return Err(Error::Format("story skip: AllSage no longer plays the Light Arrows scene"));
    }
    let reset: usize = find(f, ARROWS_RESET)?;
    if !matches!(f.events[reset].kind, Kind::Fork { .. }) {
        return Err(Error::Format("story skip: the Light Arrows reset is not a fork, the game files differ"));
    }
    let unequip: usize = find_action(f, ARROWS_UNEQUIP)?;
    let fade: usize = find_action(f, ARROWS_FADE_IN)?;
    let done: usize = find_action(f, ARROWS_DONE)?;
    // The forced Master Sword: same next, now a harmless flag write (0090 is set again below).
    let sword: usize = find_action(f, ARROWS_SWORD)?;
    let next: u16 = next_of(&f.events[sword].kind).ok_or(Error::Format("story skip: the sword request has no next"))?;
    let mut k: Kind = set_bool(f, "HavePlayedEvent.Dm_TimeTemple_0090", true)?;
    set_next(&mut k, next)?;
    f.events[sword].kind = k;
    f.events[sword].name = format!("{ARROWS_PREFIX}{sword}");
    let reset: u16 = idx(reset)?;
    for (name, scene) in ARROWS_CUT {
        let to: Option<u16> = (scene == "Dm_TimeTemple_0135").then_some(reset);
        transition_to_flag(f, ARROWS_PREFIX, name, scene, to)?;
    }
    set_next(&mut f.events[unequip].kind, idx(fade)?)?;
    set_next(&mut f.events[fade].kind, idx(done)?)?;
    Ok(())
}

pub fn stage_arrows(rf: &mut Romfs, have: Have) -> Res<Out> {
    edit_zs(rf, have, ARROWS_FILE, |raw| rebuild(raw, patch_light_arrows))
}

// ---------------------------------------------------------------- zelda escape

/// DoorOfTime entry DoorOfTime_Escape (the drawbridge area while DoorOfTime is Step03) fades out,
/// sets the storm, plays the timeline Dm_HrlField_0060 (Zelda and Impa ride out, the Ocarina falls
/// in the moat, Ganondorf), then restores the bridge and weather, warps Link, writes Step04 and fades
/// in with Navi's hint. Only the timeline is cut; the Ocarina pickup (Step04) and Dm_MelodyGet_Time
/// keep both checks.
const ESCAPE_EVENT: &str = "Event13";
const ESCAPE_SCENE: &str = "Dm_HrlField_0060";
pub const ESCAPE_PREFIX: &str = "OOTRREscape";

pub fn patch_escape(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, DOOR_FLOW)?;
    guard(f, ESCAPE_PREFIX)?;
    transition_to_flag(f, ESCAPE_PREFIX, ESCAPE_EVENT, ESCAPE_SCENE, None)?;
    Ok(())
}

// ---------------------------------------------------------------- owls in flows

/// Owls played from story flows. Each scene only talks (the field owl's flow writes its quest step,
/// the Threads of Time menu and the amiibo item itself, after the scene), so each transition becomes
/// its played flag and the flow goes on.
pub const OWL_PREFIX: &str = "OOTRROwl";
/// Goto_HyruleCastle entry Goto_HyruleCastle_Kaepora_00, the first field owl (closed Kakariko
/// gate): the owl flies in (Dm_HrlField_0005) and talks (0010). Dm_HrlField_0000 (the field
/// opening and its clock) stays.
const FIELD_OWL: [(&str, &str); 2] = [("Event59", "Dm_HrlField_0005"), ("Event12", "Dm_HrlField_0010")];
/// Get_StoneOfWater entry StoneOfWater_TalkKaepora_ZoraRiver: the owl at the river entrance; the
/// Water Stone step after it stays.
pub const RIVER_FILE: &str = "Event/EventFlow/Get_StoneOfWater.bfevfl.zs";
const RIVER_OWL: [(&str, &str); 1] = [("Event81", "Dm_ZoraRiver_0000")];
/// Dm_SY_BancChangeArrive: the Lost Woods owls (from Kokiri Forest, from Goron City, and after
/// Saria's Song). Their next is the same step logic the button skip lands on.
pub const ARRIVE_FILE: &str = "Event/EventFlow/Dm_SY_BancChangeArrive.bfevfl.zs";
const WOODS_OWL: [(&str, &str); 3] = [("Event214", "Dm_LostWood_0000"), ("Event209", "Dm_LostWood_0000_GoronCity"), ("Event211", "Dm_LostWood_0010")];

fn cut_owls(b: &mut Bf, flow: &str, cuts: &[(&str, &str)]) -> Res<()> {
    let f: &mut Flow = flow_mut(b, flow)?;
    guard(f, OWL_PREFIX)?;
    for (name, scene) in cuts {
        transition_to_flag(f, OWL_PREFIX, name, scene, None)?;
    }
    Ok(())
}

pub fn patch_field_owl(b: &mut Bf) -> Res<()> {
    cut_owls(b, BRIDGE_FLOW, &FIELD_OWL)
}

pub fn patch_river_owl(b: &mut Bf) -> Res<()> {
    cut_owls(b, "Get_StoneOfWater", &RIVER_OWL)
}

pub fn patch_woods_owls(b: &mut Bf) -> Res<()> {
    cut_owls(b, "Dm_SY_BancChangeArrive", &WOODS_OWL)
}

// ---------------------------------------------------------------- sheik songs

/// Sheik's six song scenes (flowcharts, Dm_MelodyGet_*). Each places the actors, talks, plays the
/// song three times (Sheik, Link's input, the duet: subs into Dm_SY_Ocarina), then hands over the
/// song with GetPouchContent (the check) and closes the message, then talks and sends Sheik away.
/// The cut jumps from the placement straight to the grant, and after the message closes runs
/// copies of the departure actions (Sheik offstage, fades, camera reset, the met-Sheik counter,
/// the temple flag of the Prelude) into the scene's own last call.
pub const SHEIK_PREFIX: &str = "OOTRRSheik";
pub struct SheikCut {
    pub scene: &'static str,
    /// The placement event whose next becomes the grant (the first copy of `before`, if any).
    pub start: &'static str,
    pub before: &'static [&'static str],
    pub grant: &'static str,
    /// The message close after the grant.
    pub close: &'static str,
    pub tail: &'static [&'static str],
    /// The original last call the tail ends on, or the end.
    pub end: Option<&'static str>,
    /// Events whose next becomes the end (a talk or the music after the placement).
    pub stop: &'static [&'static str],
}

pub const SHEIK_CUTS: [SheikCut; 6] = [
    // Minuet: Sheik is sent offstage in the placement and never seen.
    SheikCut { scene: "Dm_MelodyGet_Forest", start: "Event307", before: &["Event318"], grant: "Event22", close: "Event17", tail: &["Event60", "Event167"], end: Some("Event330"), stop: &[] },
    // Bolero: Sheik and the harp leave, the camera returns, the counter goes up.
    SheikCut { scene: "Dm_MelodyGet_Fire", start: "Event0", before: &[], grant: "Event192", close: "Event204", tail: &["Event38", "Event43", "Event244", "Event152"], end: Some("Event301"), stop: &[] },
    // Serenade: Sheik leaves behind a fade; the Ice Cavern quest writes stay in the caller.
    SheikCut { scene: "Dm_MelodyGet_Water", start: "Event44", before: &[], grant: "Event29", close: "Event26", tail: &["Event18", "Event66", "Event47", "Event69", "Event19", "Event74", "Event282"], end: Some("Event123"), stop: &[] },
    // Nocturne: ends faded out like vanilla (the caller fades in); the burning village stays.
    SheikCut { scene: "Dm_MelodyGet_Shadow", start: "Event119", before: &[], grant: "Event21", close: "Event18", tail: &["Event76", "Event36"], end: None, stop: &["Event45"] },
    // Requiem: Sheik leaves, camera, counter.
    SheikCut { scene: "Dm_MelodyGet_Spilit", start: "Event253", before: &[], grant: "Event17", close: "Event14", tail: &["Event112", "Event153", "Event63"], end: Some("Event282"), stop: &[] },
    // Prelude: behind a fade, Market_Adult_GetLightPrelude removes the temple Sheik.
    SheikCut { scene: "Dm_MelodyGet_Shine", start: "Event253", before: &[], grant: "Event53", close: "Event220", tail: &["Event133", "Event100", "Event46", "Event103", "Event129", "Event116", "Event189"], end: None, stop: &["Event5"] },
];

pub fn sheik_file(scene: &str) -> String {
    format!("Event/EventFlow/{scene}.bfevfl.zs")
}

fn copy_kind(f: &Flow, name: &str) -> Res<Kind> {
    let k: Kind = f.events[find(f, name)?].kind.clone();
    if next_of(&k).is_none() {
        return Err(Error::Msg(format!("story skip: {name} in {} has no single next, the game files differ", f.name)));
    }
    Ok(k)
}

pub fn patch_sheik(b: &mut Bf, c: &SheikCut) -> Res<()> {
    let f: &mut Flow = flow_mut(b, c.scene)?;
    guard(f, SHEIK_PREFIX)?;
    let grant: usize = find(f, c.grant)?;
    if !matches!(&f.events[grant].kind, Kind::Sub { entry, .. } if entry == "GetPouchContentByActorName") {
        return Err(Error::Msg(format!("story skip: {} in {} is not the song grant, the game files differ", c.grant, c.scene)));
    }
    let close: usize = find_action(f, (c.close, "GameSystemActor.EventCloseMessageDialog"))?;
    if next_of(&f.events[grant].kind) != Some(idx(close)?) {
        return Err(Error::Msg(format!("story skip: the song grant of {} is not followed by its message close", c.scene)));
    }
    let start: usize = find(f, c.start)?;
    let end: u16 = match c.end {
        Some(n) => idx(find(f, n)?)?,
        None => NONE,
    };
    let stops: Vec<usize> = c.stop.iter().map(|n| find(f, n)).collect::<Res<_>>()?;
    let before: Vec<Kind> = c.before.iter().map(|n| copy_kind(f, n)).collect::<Res<_>>()?;
    let tail: Vec<Kind> = c.tail.iter().map(|n| copy_kind(f, n)).collect::<Res<_>>()?;
    let to_grant: u16 = if before.is_empty() { idx(grant)? } else { chain(f, SHEIK_PREFIX, before, idx(grant)?)? };
    set_next(&mut f.events[start].kind, to_grant)?;
    let after: u16 = if tail.is_empty() { end } else { chain(f, SHEIK_PREFIX, tail, end)? };
    set_next(&mut f.events[close].kind, after)?;
    for s in stops {
        set_next(&mut f.events[s].kind, NONE)?;
    }
    // Marks the flow as patched even when nothing was appended.
    f.events[start].name = format!("{SHEIK_PREFIX}{start}");
    Ok(())
}

pub fn stage_sheik(rf: &mut Romfs, have: Have) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = Vec::new();
    for c in &SHEIK_CUTS {
        // The location patcher has staged these (their grant holds the song check).
        outs.push(edit_zs(rf, have, &sheik_file(c.scene), |raw| rebuild(raw, |b| patch_sheik(b, c)))?);
    }
    Ok(outs)
}

// ---------------------------------------------------------------- burning kakariko

/// Dm_SY_BancChangeArrive entry KakarikoVillage_Arrive (adult Kakariko before the Shadow Temple):
/// preloads KakarikoVillage_Adult, sets KakarikoVillage_Burning_Start, forces the Master Sword out
/// (Event37), mutes the world sounds (Event498), plays the timeline Dm_Kakariko_0030 (the village
/// burns), then changes to KakarikoVillage_Adult booting KakarikoVillage_FlashbackScenes, whose only
/// job is the timeline Dm_Kakariko_0040 (Sheik's flashback) and a change to
/// KakarikoVillage_Adult_Alternative booting KakarikoVillage_MelodyGet_Shadow (the Nocturne scene
/// and the Shadow quest writes). The cut: the sword request becomes the 0030 flag, the mute is
/// skipped, the 0030 transition becomes the 0040 flag, and the preload and the banc change go
/// straight to the Alternative village with the Nocturne boot (one load instead of two).
const BURN_SWORD: (&str, &str) = ("Event37", "Player_Event.EventTriggerEquipmentUserRequestCreateStaticEquipment");
const BURN_LOG: (&str, &str) = ("Event484", "UISystemActor.EventTriggerSetIsNotClearMessageLog");
const BURN_SCENE: (&str, &str) = ("Event70", "Dm_Kakariko_0030");
const BURN_PRELOAD: (&str, &str) = ("Event327", "GameSystemActor.EventTriggerBancRequestBeforehandLoad");
const BURN_CHANGE: (&str, &str) = ("Event329", "GameSystemActor.EventReserveBancChange");
const BURN_PATH: &str = "Work/Banc/Scene/KakarikoVillage/KakarikoVillage_Adult_Alternative.bcett.json";
const BURN_BANC: &str = "KakarikoVillage_Adult_Alternative";
const BURN_BOOT: &str = "KakarikoVillage_MelodyGet_Shadow";
pub const BURN_PREFIX: &str = "OOTRRBurn";

pub fn patch_burning(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = flow_mut(b, "Dm_SY_BancChangeArrive")?;
    guard(f, BURN_PREFIX)?;
    if !f.entries.iter().any(|e| e.name == BURN_BOOT) {
        return Err(Error::Format("story skip: the Nocturne boot entry is missing, the game files differ"));
    }
    let sword: usize = find_action(f, BURN_SWORD)?;
    let log: usize = find_action(f, BURN_LOG)?;
    let preload: usize = find_action(f, BURN_PRELOAD)?;
    let change: usize = find_action(f, BURN_CHANGE)?;
    if text(&f.events[change], "BootEventName") != Some("KakarikoVillage_FlashbackScenes") {
        return Err(Error::Format("story skip: the burning village no longer boots the flashback, the game files differ"));
    }
    let next: u16 = next_of(&f.events[sword].kind).ok_or(Error::Format("story skip: the sword request has no next"))?;
    let mut k: Kind = set_bool(f, &format!("HavePlayedEvent.{}", BURN_SCENE.1), true)?;
    set_next(&mut k, next)?;
    f.events[sword].kind = k;
    f.events[sword].name = format!("{BURN_PREFIX}{sword}");
    let scene: usize = transition_to_flag(f, BURN_PREFIX, BURN_SCENE.0, BURN_SCENE.1, None)?;
    // The flag of the flashback in place of the burning scene's transition.
    set_param(&mut f.events[scene].kind, "GameDataName", Val::Str("HavePlayedEvent.Dm_Kakariko_0040".to_string()))?;
    set_next(&mut f.events[log].kind, idx(scene)?)?;
    set_param(&mut f.events[preload].kind, "BancPath", Val::Str(BURN_PATH.to_string()))?;
    set_param(&mut f.events[change].kind, "NextBanc", Val::Str(BURN_BANC.to_string()))?;
    set_param(&mut f.events[change].kind, "BootEventName", Val::Str(BURN_BOOT.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------- staging

fn source(rf: &mut Romfs, have: Have, path: &str) -> Res<Vec<u8>> {
    let z: Vec<u8> = match have(path) {
        Some(b) => b,
        None => rf.read(path)?,
    };
    rf.zs.decompress(&z)
}

/// The item the location patcher left in a grant event (the vanilla one when unshuffled).
pub fn placed(rf: &mut Romfs, have: Have, file: &str, index: usize, event: &str, param: &str) -> Res<String> {
    let raw: Vec<u8> = source(rf, have, file)?;
    param_of(&raw, index, event, param)
}

/// Rewrites the bridge and the draw. `have` returns a file already staged by an earlier step (the door fix,
/// the location patcher), so those edits are kept and Rauru's gift follows the shuffled item.
pub fn stage(rf: &mut Romfs, have: Have) -> Res<Vec<Out>> {
    let medal: String = placed(rf, have, RAURU_FILE, RAURU_INDEX, RAURU_EVENT, RAURU_PARAM)?;
    let l: crate::flags::FlagList = crate::flags_build::load(rf)?;
    let got: String = format!("IsGet.{medal}");
    let got: Option<&str> = l.by_hash(crate::flags::hash_name(&got) as u64).is_some().then_some(got.as_str());
    Ok(vec![
        edit_zs(rf, have, BRIDGE_FILE, |raw| {
            rebuild(raw, |b| {
                patch_bridge(b)?;
                patch_field_owl(b)
            })
        })?,
        edit_zs(rf, have, GIFT_FILE, |raw| rebuild(raw, patch_gift))?,
        edit_zs(rf, have, RIVER_FILE, |raw| rebuild(raw, patch_river_owl))?,
        edit_zs(rf, have, ARRIVE_FILE, |raw| {
            rebuild(raw, |b| {
                patch_woods_owls(b)?;
                patch_burning(b)
            })
        })?,
        edit_zs(rf, have, DOOR_FILE, |raw| {
            rebuild(raw, |b| {
                patch_draw(b, &medal, got)?;
                patch_escape(b)
            })
        })?,
        stage_arrows(rf, have)?,
    ]
    .into_iter()
    .chain(stage_sheik(rf, have)?)
    .collect())
}

pub fn worked(rf: &mut Romfs, pk: &crate::modpack::Pack) -> Res<Vec<Out>> {
    let root: std::path::PathBuf = pk.work.join("romfs");
    let have = |p: &str| -> Option<Vec<u8>> { fs::read(p.split('/').fold(root.clone(), |a, s| a.join(s))).ok() };
    stage(rf, &have)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("D:/Oot/romfs"));
        if p.is_dir() {
            Some(p)
        } else {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", p.display());
            None
        }
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    /// Follows single nexts from `start`, collecting each event's summary.
    fn walk(f: &Flow, start: u16, limit: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut i: u16 = start;
        while i != NONE && out.len() < limit {
            let e: &Event = &f.events[i as usize];
            let what: String = match &e.kind {
                Kind::Action { .. } => {
                    let a: String = action(f, i as usize).unwrap_or_default();
                    let p: String = ["EventName", "GameDataName", "Actor", "ChallengeName", "StepName"].iter().filter_map(|k| text(e, k)).collect::<Vec<_>>().join(",");
                    format!("{a}({p})")
                }
                Kind::Sub { flow, entry, .. } => format!("sub {flow}:{entry}"),
                k => format!("{k:?}"),
            };
            out.push(what);
            i = next_of(&e.kind).unwrap_or(NONE);
        }
        out
    }

    /// Like walk, but a fork continues at its join.
    fn walk_joins(f: &Flow, start: u16, limit: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut i: u16 = start;
        while i != NONE && out.len() < limit {
            let w: Vec<String> = walk(f, i, 1);
            out.extend(w);
            i = match &f.events[i as usize].kind {
                Kind::Fork { join, .. } => *join,
                k => next_of(k).unwrap_or(NONE),
            };
        }
        out
    }

    #[test]
    fn the_new_game_state_covers_the_deku_tree_and_the_child_temple() {
        let o: Ops = deku();
        assert_eq!(o.steps[0], ("Meet_DekuTree".to_string(), "Complete".to_string()));
        assert_eq!(o.steps.last().unwrap(), &("Get_StoneOfForest".to_string(), "MeetBoss".to_string()));
        assert!(o.flags.iter().any(|(n, _)| n == "KokiriForest_Meet_DekuTree_OpenDekuTreeMouth"));
        assert!(o.items.is_empty() && o.time.is_none());
        let s: Ops = start();
        assert_eq!(s.flags.len(), TEMPLE_FLAGS.len() + OWL_FLAGS.len());
        assert!(s.flags.iter().all(|(_, v)| *v));
        assert!(!s.flags.iter().any(|(n, _)| n.contains("Kokiri")));
        assert!(s.steps.is_empty() && s.items.is_empty());
    }

    #[test]
    fn the_real_draw_hands_over_to_the_later_draw() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        assert_eq!(placed(&mut rf, &none, RAURU_FILE, RAURU_INDEX, RAURU_EVENT, RAURU_PARAM).unwrap(), "Item_Medal_Light");
        let mut b: Bf = Bf::parse(&rf.read_zs(DOOR_FILE).unwrap()).unwrap();
        crate::door_fix::patch_flow(&mut b).unwrap();
        patch_draw(&mut b, "Item_Test", Some("IsGet.Item_Test")).unwrap();
        assert!(patch_draw(&mut b, "Item_Test", None).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == DOOR_FLOW).unwrap();
        let Kind::Switch { cases, .. } = &f.events[find(f, DRAW_SWITCH).unwrap()].kind else { panic!() };
        let later: u16 = cases.iter().find(|c| c.val == 0).unwrap().ev;
        assert_eq!(walk(f, later, 2), vec!["EventSystemActor.EventTransition(Market_Child_Field_ChildToAdult)".to_string()]);
        let w: Vec<String> = walk(f, cases.iter().find(|c| c.val == 1).unwrap().ev, 30);
        assert!(w[1].contains("HavePlayedEvent.Dm_TimeTemple_0040"));
        assert_eq!(w[9], "PouchSystemActor.EventTriggerAddOrRemovePouchContent(Weapon_Sword_Master)");
        assert_eq!(w[10], "PouchSystemActor.EventTriggerAddOrRemovePouchContent(Item_Test)");
        assert_eq!(w[11], "GameDataActor.EventTriggerSetGameDataBool(IsGet.Item_Test)");
        assert_eq!(w[12], "ChallengeSystemActor.EventExecuteProgressChallengeStep(SageOfLight,Complete)");
        assert_eq!(w[14], "ChallengeSystemActor.EventExecuteProgressChallengeStep(SageOfForest,Step01)");
        assert!(w[15].starts_with("Switch"), "{}", w[15]);
        assert_eq!(w.len(), 16);
        // The guard: Step06 or later asks for Complete then transitions, else just transitions;
        // the transition is the last event either way.
        let mut i: u16 = cases.iter().find(|c| c.val == 1).unwrap().ev;
        for _ in 0..15 {
            i = next_of(&f.events[i as usize].kind).unwrap();
        }
        let Kind::Switch { cases: g, .. } = &f.events[i as usize].kind else { panic!() };
        let yes: Vec<String> = walk(f, g.iter().find(|c| c.val == 0).unwrap().ev, 5);
        assert_eq!(yes, vec!["ChallengeSystemActor.EventExecuteProgressChallengeStep(DoorOfTime,Complete)".to_string(), "EventSystemActor.EventTransition(Market_Child_Field_ChildToAdult)".to_string()]);
        let no: Vec<String> = walk(f, g.iter().find(|c| c.val != 0).unwrap().ev, 5);
        assert_eq!(no, vec!["EventSystemActor.EventTransition(Market_Child_Field_ChildToAdult)".to_string()]);
        assert!(!w.iter().chain(&yes).any(|s| s.starts_with("sub")));
        assert!(f.events.iter().filter(|e| e.name.starts_with(DOOR_PREFIX)).all(|e| match &e.kind {
            Kind::Action { params, .. } => params.as_ref().and_then(|c| c.get("IsEquipDIrect")) != Some(&Val::Bool(TRUE)),
            _ => true,
        }));
        Bf::parse(&b.write().unwrap()).unwrap();
        assert_eq!(stage(&mut rf, &none).unwrap().len(), 6 + SHEIK_CUTS.len());
    }

    #[test]
    fn the_real_bridge_goes_straight_to_the_gift_scene() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(BRIDGE_FILE).unwrap()).unwrap();
        crate::start_state::keep_field_clock(&mut b).unwrap();
        patch_bridge(&mut b).unwrap();
        assert!(patch_bridge(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == BRIDGE_FLOW).unwrap();
        let w: Vec<String> = walk(f, find(f, DUCKING).unwrap() as u16, 6);
        assert!(w[0].starts_with("SoundSystemActor.EventSoundRequestDucking"));
        assert_eq!(w[1], "GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.Dm_Kokiri_0150)");
        assert!(w[2].starts_with("SoundSystemActor.EventSoundReleaseDucking"));
        assert_eq!(w[3], "EventSystemActor.EventTransition(Dm_Kokiri_0155)");
        assert_eq!(w[4], "GameDataActor.EventTriggerSetGameDataBool(HyruleField_ChangeENV)");
        assert!(w[5].contains("EventReserveBancChange"));
        Bf::parse(&b.write().unwrap()).unwrap();
    }
    #[test]
    fn the_real_dungeon_endings_keep_only_the_reward_scenes() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(ENDING_FILE).unwrap()).unwrap();
        let n: usize = patch_endings(&mut b).unwrap();
        assert!(n >= CUT_SCENES.len());
        assert!(patch_endings(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == ENDING_FLOW).unwrap();
        let played: Vec<String> = (0..f.events.len()).filter(|&i| action(f, i).as_deref() == Some("EventSystemActor.EventTransition")).filter_map(|i| text(&f.events[i], "EventName").map(str::to_string)).collect();
        for s in CUT_SCENES {
            assert!(!played.iter().any(|p| p == s), "{s} still plays");
        }
        for s in ["Dm_Kokiri_0130", "Dm_DeathMtTrail_0025", "Dm_ZoraFountain_0030", "Dm_Awake_Forest_Get", "Dm_Awake_Fire_Get", "Dm_Awake_Water_Get", "Dm_Awake_Shadow_Get", "Dm_Awake_Spirit_Get"] {
            assert!(played.iter().any(|p| p == s), "{s} no longer plays");
        }
        // The Deku Tree chain: flags in place of the four scenes, the emerald scene kept.
        let e = f.entries.iter().find(|e| e.name == "Warp_Kokiri").unwrap();
        let w: Vec<String> = walk(f, e.main, 12);
        let at: usize = w.iter().position(|s| s == "EventSystemActor.EventTransition(Dm_Kokiri_0130)").unwrap();
        let flag = |n: &str| format!("GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.{n})");
        assert_eq!(w[at - 3..at].to_vec(), vec![flag("Dm_Kokiri_0080"), flag("Dm_Kokiri_0095"), flag("Dm_Kokiri_0090")]);
        assert_eq!(w[at + 1], flag("Dm_Kokiri_0140"));
        Bf::parse(&b.write().unwrap()).unwrap();
    }

    #[test]
    fn the_real_medallion_scenes_end_without_the_white_screen_lines() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        for s in MEDAL_SCENES {
            let mut b: Bf = Bf::parse(&rf.read_zs(&medal_file(s)).unwrap()).unwrap();
            let n: usize = cut_white_screen(&mut b).unwrap();
            assert!(n >= 3, "{s}: {n}");
            assert!(cut_white_screen(&mut b).is_err(), "{s}: a second run finds no line");
            let f: &Flow = &b.flows[0];
            let g: usize = (0..f.events.len()).find(|&i| matches!(&f.events[i].kind, Kind::Sub { entry, .. } if entry == "GetPouchContentByActorName")).unwrap();
            let w: Vec<String> = walk(f, g as u16, 10);
            assert!(w[0].starts_with("sub GetPouchContent:GetPouchContentByActorName"), "{s}");
            assert!(w.iter().any(|x| x.starts_with("UISystemActor.EventEventFadeOut")), "{s}");
            assert!(!w.iter().any(|x| x.ends_with(")") && x.contains(".EventTalk(")), "{s}: {w:?}");
            let tail: Vec<&String> = w.iter().skip_while(|x| !x.starts_with("Player_Event.EventTriggerPlayerRestoreBootsChange")).skip(1).collect();
            if s.contains("Forest") || s.contains("Fire") {
                assert_eq!(tail, vec!["GameDataActor.EventTriggerSetGameDataBool(AllSage_AwakeSage_Fire_Or_Forest)"], "{s}");
            } else {
                assert!(tail.is_empty(), "{s}: {tail:?}");
            }
        }
    }

    #[test]
    fn saria_song_to_darunia_goes_straight_to_the_bracelet_scene_and_it_starts_at_the_gift() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(STONE_FILE).unwrap()).unwrap();
        patch_darunia_song(&mut b).unwrap();
        let f: &Flow = flow_mut(&mut b, STONE_FLOW).unwrap();
        let ask: usize = f.events.iter().position(|e| e.name == "Event35").unwrap();
        let Kind::Switch { cases, .. } = &f.events[ask].kind else { panic!() };
        let to: usize = cases.iter().find(|c| c.val == 0).unwrap().ev as usize;
        assert_eq!(f.events[to].name, "Event39");
        let Kind::Switch { cases: pick, .. } = &f.events[to].kind else { panic!() };
        assert!(pick.iter().any(|c| text(&f.events[c.ev as usize], "EventName") == Some(BRACELET)));
        // The wrong melody case still goes to Darunia's other scene.
        let other: usize = cases.iter().find(|c| c.val == 1).unwrap().ev as usize;
        assert_eq!(text(&f.events[other], "EventName"), Some("Dm_Goron_0028"));
        let mut s: Bf = Bf::parse(&rf.read_zs(BRACELET_FILE).unwrap()).unwrap();
        cut_bracelet_scene(&mut s).unwrap();
        assert!(cut_bracelet_scene(&mut s).is_err());
        let f: &Flow = &s.flows[0];
        let e = f.entries.iter().find(|e| e.name == BRACELET_MAIN).unwrap();
        let Kind::Fork { forks, join } = &f.events[e.main as usize].kind else { panic!("main is no fork") };
        assert!(forks.iter().any(|x| matches!(&f.events[*x as usize].kind, Kind::Sub { entry, .. } if entry == BRACELET_GRANT_PART)));
        // After the gift the scene only runs :C11, which keeps the quest step.
        let Kind::Join { next } = &f.events[*join as usize].kind else { panic!() };
        let Kind::Fork { forks: last, .. } = &f.events[*next as usize].kind else { panic!() };
        assert!(last.iter().any(|x| matches!(&f.events[*x as usize].kind, Kind::Sub { entry, .. } if entry == "C11")));
        assert!(f.events.iter().all(|ev| !matches!(&ev.kind, Kind::Fork { forks, .. } if forks.iter().any(|x| text(&f.events[*x as usize], "ASCommand") == Some(EQUIP_POSE)))));
        assert!(f.events.iter().any(|ev| text(ev, "StepName") == Some("DodongoCavernOpne")));
    }

    #[test]
    fn the_real_darunia_scene_ends_after_the_ruby() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(DARUNIA_FILE).unwrap()).unwrap();
        cut_darunia_talk(&mut b).unwrap();
        assert!(cut_darunia_talk(&mut b).is_err());
        let f: &Flow = &b.flows[0];
        let e = f.entries.iter().find(|e| e.name == "EntryPoint0").unwrap();
        let w: Vec<String> = walk(f, e.main, 20);
        assert_eq!(w, vec!["sub :Ready", "sub :C01", "sub :C01-05", "sub :C01-07", "SoundSystemActor.EventSoundChangeListenerPreset()", "UISystemActor.EventEventFadeOut()", "EventSystemActor.EventWait()"]);
        // The grant is still the end of the part that is kept.
        let g = f.entries.iter().find(|e| e.name == DARUNIA_GRANT_PART).unwrap();
        let mut i: u16 = g.main;
        let mut granted = false;
        for _ in 0..40 {
            if i == NONE {
                break;
            }
            granted |= matches!(&f.events[i as usize].kind, Kind::Sub { entry, .. } if entry == "GetPouchContentByActorName");
            i = match &f.events[i as usize].kind {
                Kind::Fork { join, .. } => *join,
                k => next_of(k).unwrap_or(NONE),
            };
        }
        assert!(granted);
        Bf::parse(&b.write().unwrap()).unwrap();
    }

    #[test]
    fn the_real_light_arrows_scene_keeps_only_the_grant() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(ARROWS_FILE).unwrap()).unwrap();
        patch_light_arrows(&mut b).unwrap();
        assert!(patch_light_arrows(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == ARROWS_FLOW).unwrap();
        let flag = |n: &str| format!("GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.{n})");
        for entry in ["Market_Adult_Field_GetArrowLight", "GetArrowLight_OcarinaBackTimeTemple"] {
            let e = f.entries.iter().find(|e| e.name == entry).unwrap();
            let w: Vec<String> = walk_joins(f, e.main, 60);
            assert!(!w.iter().any(|x| x.contains("Equipment")), "{entry}: {w:?}");
            let at: usize = w.iter().position(|x| x == "EventSystemActor.EventTransition(Dm_TimeTemple_0130)").unwrap();
            assert_eq!(w[at + 1], flag("Dm_TimeTemple_0135"));
            assert_eq!(w.iter().filter(|x| x.starts_with("EventSystemActor.EventTransition")).count(), 1, "{entry}: {w:?}");
            for s in ["Dm_TimeTemple_0090", "Dm_TimeTemple_0095", "Dm_TimeTemple_0110", "Dm_TimeTemple_0115", "Dm_TimeTemple_0125"] {
                assert!(w[..at].contains(&flag(s)), "{entry}: {s}");
            }
            let tail: Vec<&String> = w[at + 2..].iter().filter(|x| !x.starts_with("Fork") && !x.starts_with("Join")).collect();
            assert_eq!(tail, vec!["Player_Event.EventPlayerUnequip()", "UISystemActor.EventEventFadeIn()", "ChallengeSystemActor.EventExecuteProgressChallengeStep(AllSage,Complete)", "ChallengeSystemActor.EventExecuteProgressChallengeStep(Defeat_Ganon,Step01)"], "{entry}");
        }
        Bf::parse(&b.write().unwrap()).unwrap();
    }

    #[test]
    fn the_real_zelda_escape_keeps_the_step_and_the_fade() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(DOOR_FILE).unwrap()).unwrap();
        patch_escape(&mut b).unwrap();
        assert!(patch_escape(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == DOOR_FLOW).unwrap();
        let e = f.entries.iter().find(|e| e.name == "DoorOfTime_Escape").unwrap();
        let w: Vec<String> = walk_joins(f, e.main, 80);
        assert!(w.contains(&"GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.Dm_HrlField_0060)".to_string()), "{w:?}");
        assert!(!w.iter().any(|x| x.starts_with("EventSystemActor.EventTransition")), "{w:?}");
        assert!(w.contains(&"ChallengeSystemActor.EventExecuteProgressChallengeStep(DoorOfTime,Step04)".to_string()), "{w:?}");
        Bf::parse(&b.write().unwrap()).unwrap();
    }

    #[test]
    fn the_real_owls_become_flags() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        for (file, patch) in [(BRIDGE_FILE, patch_field_owl as fn(&mut Bf) -> Res<()>), (RIVER_FILE, patch_river_owl), (ARRIVE_FILE, patch_woods_owls)] {
            let mut b: Bf = Bf::parse(&rf.read_zs(file).unwrap()).unwrap();
            patch(&mut b).unwrap();
            assert!(patch(&mut b).is_err(), "{file}");
            let f: &Flow = &b.flows[0];
            for s in ["Dm_HrlField_0005", "Dm_HrlField_0010", "Dm_ZoraRiver_0000", "Dm_LostWood_0000", "Dm_LostWood_0000_GoronCity", "Dm_LostWood_0010"] {
                assert!(!(0..f.events.len()).any(|i| action(f, i).as_deref() == Some("EventSystemActor.EventTransition") && text(&f.events[i], "EventName") == Some(s)), "{file}: {s}");
            }
            Bf::parse(&b.write().unwrap()).unwrap();
        }
        // The field owl's flow still writes its quest step, menu and item after the cut.
        let mut b: Bf = Bf::parse(&rf.read_zs(BRIDGE_FILE).unwrap()).unwrap();
        patch_field_owl(&mut b).unwrap();
        let f: &Flow = &b.flows[0];
        let e = f.entries.iter().find(|e| e.name == "Goto_HyruleCastle_Kaepora_00").unwrap();
        let w: Vec<String> = walk_joins(f, e.main, 40);
        for want in ["EventSystemActor.EventTransition(Dm_HrlField_0000)", "GameDataActor.EventTriggerSetGameDataBool(UI_IsFinishChallengeMenuOpenEvent)", "PouchSystemActor.EventTriggerAddOrRemovePouchContent(Pouch_Amiibo)", "ChallengeSystemActor.EventExecuteProgressChallengeStep(Goto_HyruleCastle,Step02)"] {
            assert!(w.iter().any(|x| x == want), "{want}: {w:?}");
        }
    }

    #[test]
    fn the_real_sheik_scenes_go_from_the_placement_to_the_song() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        for c in &SHEIK_CUTS {
            let mut b: Bf = Bf::parse(&rf.read_zs(&sheik_file(c.scene)).unwrap()).unwrap();
            patch_sheik(&mut b, c).unwrap();
            assert!(patch_sheik(&mut b, c).is_err(), "{}", c.scene);
            let f: &Flow = &b.flows[0];
            let e = f.entries.iter().find(|e| e.name == "EntryPoint0").unwrap();
            let w: Vec<String> = walk_joins(f, e.main, 80);
            let at: usize = w.iter().position(|x| x == "sub GetPouchContent:GetPouchContentByActorName").unwrap_or_else(|| panic!("{}: {w:?}", c.scene));
            // No ocarina lesson, no talk on the way in or out.
            assert!(!w.iter().any(|x| x.contains("Dm_SY_Ocarina") || x.contains(".EventTalk(")), "{}: {w:?}", c.scene);
            assert_eq!(w[at + 1], "GameSystemActor.EventCloseMessageDialog()", "{}", c.scene);
            let rest: &[String] = &w[at + 2..];
            if !["Dm_MelodyGet_Shadow", "Dm_MelodyGet_Forest", "Dm_MelodyGet_Shine"].contains(&c.scene) {
                assert!(rest.iter().any(|x| x.contains("Sheik_Adult.EventGoOffstageForUser")), "{}: {rest:?}", c.scene);
            }
            if c.scene == "Dm_MelodyGet_Shine" {
                assert!(rest.iter().any(|x| x.ends_with("(Market_Adult_GetLightPrelude)")), "{rest:?}");
            }
            if c.scene != "Dm_MelodyGet_Shadow" {
                assert!(rest.iter().any(|x| x.ends_with("(AllSage_MeetSheikCount)")), "{}: {rest:?}", c.scene);
            }
            Bf::parse(&b.write().unwrap()).unwrap();
        }
    }

    #[test]
    fn the_real_burning_village_loads_once_into_the_nocturne() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(ARRIVE_FILE).unwrap()).unwrap();
        patch_woods_owls(&mut b).unwrap();
        patch_burning(&mut b).unwrap();
        assert!(patch_burning(&mut b).is_err());
        let f: &Flow = &b.flows[0];
        // From the preload, past the horse check at the start of KakarikoVillage_Arrive.
        let w: Vec<String> = walk_joins(f, find(f, BURN_PRELOAD.0).unwrap() as u16, 40);
        assert!(!w.iter().any(|x| x.contains("RequestCreateStaticEquipment") || x.starts_with("EventSystemActor.EventTransition") || x.contains("Ducking")), "{w:?}");
        for want in ["GameDataActor.EventTriggerSetGameDataBool(KakarikoVillage_Burning_Start)", "GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.Dm_Kakariko_0030)", "GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.Dm_Kakariko_0040)"] {
            assert!(w.iter().any(|x| x == want), "{want}: {w:?}");
        }
        assert_eq!(w.last().unwrap(), "Player_Event.EventWaitCreationFinishedEquipment()");
        let change: usize = find(f, BURN_CHANGE.0).unwrap();
        assert_eq!(text(&f.events[change], "NextBanc"), Some(BURN_BANC));
        assert_eq!(text(&f.events[change], "BootEventName"), Some(BURN_BOOT));
        assert_eq!(text(&f.events[find(f, BURN_PRELOAD.0).unwrap()], "BancPath"), Some(BURN_PATH));
        Bf::parse(&b.write().unwrap()).unwrap();
    }

    #[test]
    fn the_real_gift_scene_is_a_plain_item_get() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(GIFT_FILE).unwrap()).unwrap();
        patch_gift(&mut b).unwrap();
        assert!(patch_gift(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == GIFT_FLOW).unwrap();
        let e = f.entries.iter().find(|e| e.name == GIFT_ENTRY).unwrap();
        let w: Vec<String> = walk(f, e.main, 5);
        assert_eq!(w, vec!["sub GetPouchContent:DisplayGetByActorName".to_string(), "PouchSystemActor.EventTriggerAddOrRemovePouchContent(Item_Ocarina_Fairy)".to_string()]);
        assert_eq!(e.subs, vec![find(f, GIFT_SHOW).unwrap() as u16]);
        // With a progressive item placed, both events are tier checks first; every window
        // copy then leads to the grant check, and every grant copy ends the flow.
        let mut b: Bf = Bf::parse(&rf.read_zs(GIFT_FILE).unwrap()).unwrap();
        let fl: &mut Flow = flow_mut(&mut b, GIFT_FLOW).unwrap();
        let (g, s): (usize, usize) = (find(fl, GIFT_GRANT).unwrap(), find(fl, GIFT_SHOW).unwrap());
        crate::progressive::wrap_grant(fl, g, GIFT_GRANT, "Actor", "Ocarina").unwrap();
        crate::progressive::wrap_grant(fl, s, GIFT_SHOW, "ActorName", "Ocarina").unwrap();
        patch_gift(&mut b).unwrap();
        let f: &Flow = b.flows.iter().find(|f| f.name == GIFT_FLOW).unwrap();
        assert_eq!(f.entries.iter().find(|e| e.name == GIFT_ENTRY).unwrap().main as usize, s);
        assert_eq!(ends(f, s).unwrap() as usize, g);
        assert_eq!(ends(f, g).unwrap(), NONE);
        let out: Vec<u8> = b.write().unwrap();
        assert_eq!(Bf::parse(&out).unwrap(), b);
    }

    #[test]
    fn the_real_fire_temple_goes_straight_to_the_chamber() {
        let Some(root) = dump() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let mut b: Bf = Bf::parse(&rf.read_zs(ENDING_FILE).unwrap()).unwrap();
        patch_endings(&mut b).unwrap();
        patch_fire_ending(&mut b).unwrap();
        assert!(patch_fire_ending(&mut b).is_err());
        let f: &Flow = b.flows.iter().find(|f| f.name == ENDING_FLOW).unwrap();
        let w: Vec<String> = walk(f, find(f, FIRE_EXIT.0).unwrap() as u16, 10);
        assert_eq!(w, vec!["EventSystemActor.EventTransition(Dm_FireTemple_0030)", "GameDataActor.EventTriggerSetGameDataBool(HavePlayedEvent.Dm_Kakariko_0025)", "GameSystemActor.EventTriggerBancRequestBeforehandLoad()", "GameSystemActor.EventReserveBancChange()"]);
        let last: &Event = f.events.iter().find(|e| e.name.starts_with(FIRE_PREFIX) && text(e, "NextBanc").is_some()).unwrap();
        assert_eq!(text(last, "NextBanc"), Some("SageRoom"));
        assert_eq!(text(last, "BootEventName"), Some("FireTemple_SageOfFire_SageRoom"));
        Bf::parse(&b.write().unwrap()).unwrap();
    }
}
