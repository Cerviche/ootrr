//! Dungeon items (small keys, boss keys, maps, compasses) from gift scenes: cutscenes, Great
//! Fairies, the dungeon reward scenes and Saria's gift (crate::slice::flow_rules::gift_scene).
//!
//! A gift scene hands out its item through the game's generic pouch get: the get window, then the
//! pouch add. The pouch add does not know which dungeon a key belongs to, so for a dungeon item the
//! grant keeps only the get window (GetPouchContent:DisplayGetByActorName, the window Saria's gift
//! shows) and then makes the writes a credited chest makes (crate::dungeon_credit for small keys,
//! crate::dungeon_grant for the rest): the dungeon's key count and flags, then the line that names
//! the dungeon. The location patcher has already pointed the grant's ActorName at the item's actor.

use std::collections::BTreeSet as BS;

use crate::bfevfl::{Bf, Cont, Flow, Kind, Val};
use crate::dungeon_credit::{action_slot, add_int, flag, idx, push, set_bool, talk, HOLDER, SET, TALK, TALKER};
use crate::dungeon_grant::{struct_of, Kind as Grant};
use crate::error::{Error as E, Res};
use crate::romfs::Romfs;
use crate::settings::seed::SeedFile;
use crate::flow_locations::StagedFile;
use crate::slice::data::Data;

const POUCH: &str = "GetPouchContent";
const WINDOW: &str = "DisplayGetByActorName";
const ADD: &str = "PouchSystemActor.EventTriggerAddOrRemovePouchContent";
const ADD_INT: &str = "EventTriggerAddOrSubGameDataInt";
const PARAM: &str = "ActorName";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Small,
    Other(Grant),
}

impl Item {
    pub fn of(kind: &str) -> Option<Item> {
        match kind {
            "SmallKey" => Some(Item::Small),
            k => Grant::of(k).map(Item::Other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gift {
    pub location: String,
    pub file: String,
    pub index: usize,
    pub event: String,
    pub item: Item,
    pub dungeon: &'static str,
}

/// True for a flow location whose grant is the generic pouch get.
pub fn spot(d: &Data, name: &str) -> bool {
    d.flow.by_name(name).is_some_and(|l| l.name == crate::slice::flow_rules::SARIA || crate::slice::flow_rules::gift_scene(l))
}

/// The dungeon items the seed puts in gift scenes, one per grant event.
pub fn gifts(d: &Data, seed: &SeedFile) -> Res<Vec<Gift>> {
    let mut out: Vec<Gift> = Vec::new();
    for (loc, item) in &seed.placements {
        if !spot(d, loc) {
            continue;
        }
        let Some(i) = d.item(item) else { continue };
        let Some(k) = Item::of(&i.kind) else { continue };
        let dungeon: &'static str = i
            .dungeon
            .as_deref()
            .and_then(struct_of)
            .ok_or_else(|| E::Msg(format!("{item} at {loc} belongs to no dungeon this build can credit")))?;
        let l = d.flow.by_name(loc).ok_or_else(|| E::Msg(format!("{loc} is not a flow location")))?;
        for e in l.edits.iter().filter(|e| e.role == "grant") {
            for file in std::iter::once(&e.file).chain(e.also.iter()) {
                out.push(Gift { location: loc.clone(), file: file.clone(), index: e.index, event: e.event.clone(), item: k, dungeon });
            }
        }
    }
    Ok(out)
}

/// The boss key, map and compass lines the gifts need (crate::dungeon_grant::stage_text).
pub fn marked(gs: &[Gift]) -> BS<(Grant, &'static str)> {
    gs.iter()
        .filter_map(|g| match g.item {
            Item::Other(k) => Some((k, g.dungeon)),
            Item::Small => None,
        })
        .collect()
}

/// The dungeons whose small key line the gifts need (crate::hints::stage keys).
pub fn small_dungeons(gs: &[Gift]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = gs.iter().filter(|g| g.item == Item::Small).map(|g| g.dungeon).collect();
    out.sort();
    out.dedup();
    out
}

fn actor_of(c: &Cont) -> Option<String> {
    c.items.iter().find(|(k, _)| k == PARAM || k == "Actor").and_then(|(_, v)| match v {
        Val::Str(s) | Val::Actor(s, _) => Some(s.clone()),
        _ => None,
    })
}

fn action_name(f: &Flow, i: usize) -> Option<String> {
    match &f.events.get(i)?.kind {
        Kind::Action { actor, act, .. } => {
            let a = f.actors.get(*actor as usize)?;
            Some(format!("{}.{}", a.name, a.actions.get(*act as usize)?))
        }
        _ => None,
    }
}

/// The credit writes and the dungeon line, ending on `end`. Returns the first event.
fn credit(f: &mut Flow, item: Item, d: &'static str, end: u16) -> Res<u16> {
    let (a, set): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let (t, say): (u16, u16) = action_slot(f, TALKER, TALK)?;
    let at: u16 = idx(f.events.len())?;
    match item {
        Item::Small => {
            let (_, add): (u16, u16) = action_slot(f, HOLDER, ADD_INT)?;
            push(f, Kind::Action { next: at + 1, actor: a, act: add, params: Some(add_int(flag(d, "SmallKeyNum"))) })?;
            push(f, Kind::Action { next: at + 2, actor: a, act: add, params: Some(add_int(flag(d, "SmallKeyTotalNum"))) })?;
            push(f, Kind::Action { next: at + 3, actor: a, act: set, params: Some(set_bool(flag(d, "IsGetSmallKeyOnce"))) })?;
            push(f, Kind::Action { next: end, actor: t, act: say, params: Some(talk(crate::dungeon_text::message_id(d))) })?;
        }
        Item::Other(k) => {
            push(f, Kind::Action { next: at + 1, actor: a, act: set, params: Some(set_bool(flag(d, k.member()))) })?;
            push(f, Kind::Action { next: end, actor: t, act: say, params: Some(talk(crate::dungeon_grant::message_id(k, d))) })?;
        }
    }
    Ok(at)
}

/// Turns one grant into the get window plus the credit. A pouch get sub call (GetPouchContent or a
/// Dm_SY_GetInEvent variant) becomes the window; a bare pouch add (Saria's, whose window is its own
/// event) becomes the credit.
pub fn edit_flow(f: &mut Flow, index: usize, event: &str, item: Item, d: &'static str) -> Res<()> {
    let e = f.events.get(index).ok_or_else(|| E::Msg(format!("gift keys: event index {index} is out of range")))?;
    if e.name != event {
        return Err(E::Msg(format!("gift keys: event {index} is named {} and not {event}, the game files differ", e.name)));
    }
    if action_name(f, index).as_deref() == Some(ADD) {
        let Kind::Action { next, .. } = f.events[index].kind else { unreachable!() };
        let first: u16 = credit(f, item, d, next)?;
        // The add event itself becomes a jump to the credit: copy the first credit write in place.
        let k: Kind = f.events[first as usize].kind.clone();
        f.events[index].kind = k;
        return Ok(());
    }
    let (next, actor): (u16, String) = match &f.events[index].kind {
        Kind::Sub { next, params, flow, .. } if flow == POUCH || flow == "Dm_SY_GetInEvent" => {
            let a: String = params.as_ref().and_then(actor_of).ok_or(E::Format("gift keys: the grant names no actor"))?;
            (*next, a)
        }
        _ => return Err(E::Msg(format!("gift keys: {event} in {} is not a pouch get, the game files differ", f.name))),
    };
    let first: u16 = credit(f, item, d, next)?;
    f.events[index].kind = Kind::Sub {
        next: first,
        params: Some(Cont { items: vec![(PARAM.to_string(), Val::Str(actor))] }),
        flow: POUCH.to_string(),
        entry: WINDOW.to_string(),
    };
    Ok(())
}

fn edit_bytes(raw: &[u8], gs: &[&Gift]) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    let f: &mut Flow = b.flows.first_mut().ok_or(E::Format("gift keys: the scene file has no flow"))?;
    for g in gs {
        edit_flow(f, g.index, &g.event, g.item, g.dungeon)?;
    }
    if let Some(m) = crate::dungeon_credit::sound(f).first() {
        return Err(E::Msg(format!("gift keys: the scene {} is broken after the edit, {m}", f.name)));
    }
    let out: Vec<u8> = b.write()?;
    if Bf::parse(&out)? != b {
        return Err(E::Format("gift keys: the edited scene does not parse back"));
    }
    Ok(out)
}

/// Edits the scenes in `files` (the location patcher's staged flows), or the game's if not staged.
pub fn apply(rf: &mut Romfs, files: &mut Vec<StagedFile>, gs: &[Gift]) -> Res<()> {
    let names: BS<&str> = gs.iter().map(|g| g.file.as_str()).collect();
    for file in names {
        let these: Vec<&Gift> = gs.iter().filter(|g| g.file == file).collect();
        let path: String = crate::flow_locations::path_of(file);
        let i: usize = match files.iter().position(|f| f.path == path) {
            Some(i) => i,
            None => {
                let z: Vec<u8> = rf.read(&path)?;
                let n: usize = rf.zs.decompress(&z)?.len();
                files.push(StagedFile { path: path.clone(), z, old: n, new: n });
                files.len() - 1
            }
        };
        let id: Option<u32> = crate::flow_locations::dict_id(&files[i].z);
        let raw: Vec<u8> = rf.zs.decompress(&files[i].z)?;
        let out: Vec<u8> = edit_bytes(&raw, &these)?;
        files[i].new = out.len();
        files[i].z = crate::zs::compress_raw(&out, id)?;
    }
    Ok(())
}
