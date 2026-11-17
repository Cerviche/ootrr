//! Window-only copies of the game's give routines, for dungeon items.
//!
//! Every reward event hands its item out through a give routine (Dm_SY_GetInEvent:GetInEvent_*,
//! NpcCommon:NpcCommon_HandOver_*, a local TakenReward_* entry and so on) that shows the item,
//! plays Link's item get, and ends in the game's generic pouch get (GetPouchContent:
//! GetPouchContentByActorName*, or a bare PouchSystemActor add after its own window). The pouch add
//! does not know which dungeon a key belongs to, so a dungeon item runs a copy of the routine whose
//! pouch get is only the get window (GetPouchContent:DisplayGetByActorName) and whose bare adds are
//! a zero-frame wait. Everything else (the item model, the animation, the camera, the clean up)
//! stays. crate::gift_keys writes the dungeon credit after the copy returns.
//!
//! A copy takes every event its entry reaches. A call into another routine that reaches a pouch get
//! (in the same file or another one) calls that routine's copy instead.

use std::collections::{BTreeMap as BM, BTreeSet as BS};

use crate::bfevfl::{Bf, Cont, Entry, Event, Flow, Kind, Val};
use crate::dungeon_credit::action_slot;
use crate::error::{Error as E, Res};
use crate::flow_locations::StagedFile;
use crate::romfs::Romfs;

pub const PREFIX: &str = "OOTRR_W_";
const POUCH: &str = "GetPouchContent";
const WINDOW: &str = "DisplayGetByActorName";
const ADD: (&str, &str) = ("PouchSystemActor", "EventTriggerAddOrRemovePouchContent");
const WAIT: (&str, &str) = ("EventSystemActor", "EventWait");
const NONE: u16 = 0xFFFF;

/// The argument a show copy reads the model it shows from.
pub const SHOW: &str = "ShowActor";
/// Calls whose item argument is the real item even in a show copy: the heart check and the
/// action guide (they act on the item got, not on the model).
const REAL_CALLS: [&str; 2] = ["Dm_SY_Get", "ActionGuide"];

/// What a copy of a give routine changes. `window`: the pouch get is only the get window (a
/// dungeon item, credited after). `show`: the model shown and held up is the ShowActor argument
/// instead of the item (an item the game has no held model for).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mode {
    pub window: bool,
    pub show: bool,
}

pub const WINDOW_ONLY: Mode = Mode { window: true, show: false };
pub const SHOW_ONLY: Mode = Mode { window: false, show: true };
pub const WINDOW_SHOW: Mode = Mode { window: true, show: true };

pub fn name_of(entry: &str, m: Mode) -> String {
    let tag: &str = match (m.window, m.show) {
        (true, false) => "W",
        (false, true) => "S",
        (true, true) => "WS",
        (false, false) => "",
    };
    format!("OOTRR_{tag}_{entry}")
}

/// Item actors the game's own give routines hold up: every item a vanilla event passes to a
/// Dm_SY_GetInEvent or NpcCommon give (taken from the game's flows), plus the dungeon items, whose
/// boss key was seen held up in game. Others, such as the Fairy Ocarina (seen crashing the item
/// get), the songs, the Stones and the Medallions, which the game only hands out in cutscenes, are
/// shown as STAND_IN.
pub const HOLDABLE: [&str; 61] = [
    "Armor_Child_Mask_Bunny", "Armor_Child_Mask_Keaton", "Armor_Child_Mask_Skull", "Armor_Child_Mask_Spooky", "Item_AgonyStone",
    "Item_Arrow_Light", "Item_BombBag_Large", "Item_BombBag_Medium", "Item_BombchuBundle_Large", "Item_BombchuBundle_Medium",
    "Item_Bottle", "Item_Bottle_BluePotion", "Item_Bottle_Milk", "Item_Bottle_RedPotion", "Item_BrokenSword", "Item_BulletBag_Large",
    "Item_BulletBag_Medium", "Item_ClaimCheck", "Item_CojiroCucco", "Item_DinFire", "Item_EyeDrop", "Item_EyeballFrog",
    "Item_FairyBless_DefenseDouble", "Item_FairyBless_MP", "Item_FairyBless_MPDouble", "Item_FaroreWind", "Item_FishingRod",
    "Item_Gauntlet_Goron", "Item_MagicBean", "Item_Mask_Truth", "Item_NayruLove", "Item_NutBag_Large", "Item_NutBag_Medium",
    "Item_OddMushroom", "Item_OddPowder", "Item_OrnamentalBottle_Milk", "Item_PoacherSaw", "Item_PocketEgg", "Item_Prescription",
    "Item_Quiver_Large", "Item_Quiver_Medium", "Item_Scale_Gold", "Item_Scale_Silver", "Item_StickBag_Large", "Item_StickBag_Medium",
    "Item_Sword_Biggoron", "Item_Token_Gerudo", "Item_Wallet_Large", "Item_Wallet_Medium", "Item_WeirdEgg", "Item_ZeldaLetter",
    "Obj_Armor_Goron", "Obj_Armor_Zora", "Obj_Heart_Piece", "Obj_MiniGame_SmallKey", "Obj_Rupee_Blue", "Obj_Rupee_Gold", "Obj_Rupee_Purple",
    "Obj_Rupee_Red", "Obj_Shield_Deku", "TBox_Rupee_Green",
];

/// The model shown in place of an item the game has no held model for: the Stone of Agony, a
/// plain quest item the game does hold up.
pub const STAND_IN: &str = "Item_AgonyStone";

pub fn holdable(actor: &str) -> bool {
    actor.starts_with("Item_Dungeon_") || HOLDABLE.contains(&actor)
}

/// The model to show for `actor` when the game has none to hold up.
pub fn show_for(actor: &str) -> Option<&'static str> {
    (!actor.is_empty() && !holdable(actor)).then_some(STAND_IN)
}

pub fn name(entry: &str) -> String {
    name_of(entry, WINDOW_ONLY)
}

pub fn path(flow: &str) -> String {
    crate::flow_locations::path_of(&format!("{flow}.bfevfl"))
}

/// The flows read so far, from the staged files when there, else from the game.
pub struct Cache {
    flows: BM<String, Bf>,
    missing: BS<String>,
    dirty: BS<String>,
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}

impl Cache {
    pub fn new() -> Cache {
        Cache { flows: BM::new(), missing: BS::new(), dirty: BS::new() }
    }

    fn load(&mut self, rf: &mut Romfs, files: &[StagedFile], flow: &str) -> Res<bool> {
        if self.flows.contains_key(flow) {
            return Ok(true);
        }
        if self.missing.contains(flow) {
            return Ok(false);
        }
        let p: String = path(flow);
        let raw: Vec<u8> = match files.iter().find(|f| f.path == p) {
            Some(s) => rf.zs.decompress(&s.z)?,
            None => match rf.read(&p) {
                Ok(z) => rf.zs.decompress(&z)?,
                Err(_) => {
                    self.missing.insert(flow.to_string());
                    return Ok(false);
                }
            },
        };
        self.flows.insert(flow.to_string(), Bf::parse(&raw)?);
        Ok(true)
    }

    /// The flow named `flow`, loaded, for editing.
    pub fn flow_mut(&mut self, rf: &mut Romfs, files: &[StagedFile], flow: &str) -> Res<&mut Flow> {
        if !self.load(rf, files, flow)? {
            return Err(E::Msg(format!("item window: there is no flow file for {flow}")));
        }
        self.dirty.insert(flow.to_string());
        let b: &mut Bf = self.flows.get_mut(flow).ok_or(E::Format("item window: a loaded flow vanished"))?;
        b.flows.first_mut().ok_or_else(|| E::Msg(format!("item window: {flow} holds no flow")))
    }

    fn flow(&self, flow: &str) -> Option<&Flow> {
        self.flows.get(flow).and_then(|b| b.flows.first())
    }

    /// Writes every edited flow back into `files` (adding the ones not staged yet).
    pub fn store(&self, rf: &mut Romfs, files: &mut Vec<StagedFile>) -> Res<()> {
        for name in &self.dirty {
            let b: &Bf = &self.flows[name];
            if let Some(m) = b.flows.first().and_then(|f| crate::dungeon_credit::sound(f).into_iter().next()) {
                return Err(E::Msg(format!("item window: {name} is broken after the edit, {m}")));
            }
            let out: Vec<u8> = b.write()?;
            if Bf::parse(&out)? != *b {
                return Err(E::Msg(format!("item window: the edited {name} does not parse back")));
            }
            let p: String = path(name);
            match files.iter().position(|f| f.path == p) {
                Some(i) => {
                    let id: Option<u32> = crate::flow_locations::dict_id(&files[i].z);
                    files[i].new = out.len();
                    files[i].z = crate::zs::compress_raw(&out, id)?;
                }
                None => {
                    let z: Vec<u8> = rf.read(&p)?;
                    let id: Option<u32> = crate::flow_locations::dict_id(&z);
                    let old: usize = rf.zs.decompress(&z)?.len();
                    files.push(StagedFile { path: p, z: crate::zs::compress_raw(&out, id)?, old, new: out.len() });
                }
            }
        }
        Ok(())
    }
}

fn targets(k: &Kind) -> Vec<u16> {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => vec![*next],
        Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter().map(|c| c.ev).collect(),
        Kind::Fork { join, forks } => std::iter::once(*join).chain(forks.iter().copied()).collect(),
        Kind::Label { .. } => Vec::new(),
    }
}

fn remap(k: &mut Kind, m: &dyn Fn(u16) -> u16) {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => *next = m(*next),
        Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter_mut().for_each(|c| c.ev = m(c.ev)),
        Kind::Fork { join, forks } => {
            *join = m(*join);
            forks.iter_mut().for_each(|x| *x = m(*x));
        }
        Kind::Label { .. } => {}
    }
}

fn reach(f: &Flow, main: u16) -> Vec<usize> {
    let mut seen: BS<usize> = BS::new();
    let mut out: Vec<usize> = Vec::new();
    let mut todo: Vec<u16> = vec![main];
    while let Some(i) = todo.pop() {
        if i == NONE || (i as usize) >= f.events.len() || !seen.insert(i as usize) {
            continue;
        }
        out.push(i as usize);
        todo.extend(targets(&f.events[i as usize].kind));
    }
    out.sort();
    out
}

fn is_pouch_get(k: &Kind) -> bool {
    matches!(k, Kind::Sub { flow, entry, .. } if flow == POUCH && entry.starts_with("GetPouchContent"))
}

fn is_add(f: &Flow, k: &Kind) -> bool {
    match k {
        Kind::Action { actor, act, .. } => f.actors.get(*actor as usize).is_some_and(|a| a.name == ADD.0 && a.sub.is_empty() && a.actions.get(*act as usize).is_some_and(|x| x == ADD.1)),
        _ => false,
    }
}

/// The routine a sub call runs: (flow, entry), the flow being `here` for a local call.
fn callee(k: &Kind, here: &str) -> Option<(String, String)> {
    match k {
        Kind::Sub { flow, entry, .. } | Kind::SubSwitch { flow, entry, .. } => Some((if flow.is_empty() { here.to_string() } else { flow.clone() }, entry.clone())),
        _ => None,
    }
}

fn entry_main(f: &Flow, entry: &str) -> Option<u16> {
    f.entries.iter().find(|e| e.name == entry).map(|e| e.main)
}

/// True when the routine reaches a pouch get, directly or through the routines it calls.
pub fn gives(c: &mut Cache, rf: &mut Romfs, files: &[StagedFile], flow: &str, entry: &str, seen: &mut BS<(String, String)>) -> Res<bool> {
    if flow == POUCH || !seen.insert((flow.to_string(), entry.to_string())) || !c.load(rf, files, flow)? {
        return Ok(false);
    }
    let (direct, calls): (bool, Vec<(String, String)>) = {
        let f: &Flow = c.flow(flow).ok_or(E::Format("item window: a loaded flow vanished"))?;
        let Some(main) = entry_main(f, entry) else { return Ok(false) };
        let evs: Vec<usize> = reach(f, main);
        let direct: bool = evs.iter().any(|&i| is_pouch_get(&f.events[i].kind) || is_add(f, &f.events[i].kind));
        (direct, evs.iter().filter_map(|&i| callee(&f.events[i].kind, flow)).collect())
    };
    if direct {
        return Ok(true);
    }
    for (f2, e2) in calls {
        if gives(c, rf, files, &f2, &e2, seen)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Makes (once) the window-only copy of `flow`:`entry` and returns its entry name.
pub fn window(c: &mut Cache, rf: &mut Romfs, files: &[StagedFile], flow: &str, entry: &str) -> Res<String> {
    variant(c, rf, files, flow, entry, WINDOW_ONLY, "ActorName")
}

fn action_actor<'a>(f: &'a Flow, k: &Kind) -> Option<&'a str> {
    match k {
        Kind::Action { actor, .. } | Kind::Switch { actor, .. } => f.actors.get(*actor as usize).map(|a| a.name.as_str()),
        _ => None,
    }
}

/// Makes (once) the `m` copy of `flow`:`entry`, whose item argument is `item`, and returns its
/// entry name.
pub fn variant(c: &mut Cache, rf: &mut Romfs, files: &[StagedFile], flow: &str, entry: &str, m: Mode, item: &str) -> Res<String> {
    let w: String = name_of(entry, m);
    if !c.load(rf, files, flow)? {
        return Err(E::Msg(format!("item window: there is no flow file for {flow}")));
    }
    let (src, evs, calls): (Entry, Vec<usize>, Vec<(usize, String, String, String)>) = {
        let f: &Flow = c.flow(flow).ok_or(E::Format("item window: a loaded flow vanished"))?;
        if f.entries.iter().any(|e| e.name == w) {
            return Ok(w);
        }
        let src: Entry = f.entries.iter().find(|e| e.name == entry).cloned().ok_or_else(|| E::Msg(format!("item window: {flow} has no entry {entry}")))?;
        let evs: Vec<usize> = reach(f, src.main);
        let calls: Vec<(usize, String, String, String)> = evs
            .iter()
            .filter(|&&i| !is_pouch_get(&f.events[i].kind))
            .filter_map(|&i| {
                callee(&f.events[i].kind, flow).map(|(a, b)| {
                    // The callee's item argument: the one this call passes our item argument in.
                    let arg: String = f.events[i]
                        .kind
                        .params()
                        .and_then(|p| p.items.iter().find(|(_, v)| *v == Val::Arg(item.to_string())).map(|(k, _)| k.clone()))
                        .unwrap_or_else(|| "ActorName".to_string());
                    (i, a, b, arg)
                })
            })
            .collect();
        (src, evs, calls)
    };
    // The routines this one calls that give the item get their own copies first.
    let mut redirect: BM<usize, String> = BM::new();
    for (i, f2, e2, arg) in calls {
        if f2 != POUCH && gives(c, rf, files, &f2, &e2, &mut BS::new())? {
            redirect.insert(i, variant(c, rf, files, &f2, &e2, m, &arg)?);
        }
    }
    let f: &mut Flow = c.flow_mut(rf, files, flow)?;
    if f.entries.iter().any(|e| e.name == w) {
        return Ok(w);
    }
    let base: usize = f.events.len();
    let at: BM<usize, u16> = evs.iter().enumerate().map(|(k, &i)| (i, (base + k) as u16)).collect();
    if base + evs.len() >= NONE as usize {
        return Err(E::Msg(format!("item window: {flow} would hold too many events")));
    }
    let at_of = |x: u16| -> u16 { at.get(&(x as usize)).copied().unwrap_or(x) };
    let (wa, wait): (u16, u16) = action_slot(f, WAIT.0, WAIT.1)?;
    for &i in &evs {
        let orig: Kind = f.events[i].kind.clone();
        let mut k: Kind = orig.clone();
        remap(&mut k, &at_of);
        let pouch: bool = is_pouch_get(&orig);
        let add: bool = is_add(f, &orig);
        // Events that act on the item got, not on the model: the pouch (window, add) and the
        // heart check and action guide calls.
        let real: bool = pouch
            || action_actor(f, &orig) == Some("PouchSystemActor")
            || matches!(&orig, Kind::Sub { flow: fl, .. } | Kind::SubSwitch { flow: fl, .. } if REAL_CALLS.contains(&fl.as_str()));
        if m.window && pouch {
            let Kind::Sub { next, params, .. } = &k else { unreachable!() };
            let actor: Val = params
                .as_ref()
                .and_then(|p| p.items.iter().find(|(n, _)| n == "ActorName").map(|(_, v)| v.clone()))
                .ok_or_else(|| E::Msg(format!("item window: the pouch get {} in {flow} names no actor", f.events[i].name)))?;
            k = Kind::Sub { next: *next, params: Some(Cont { items: vec![("ActorName".to_string(), actor)] }), flow: POUCH.to_string(), entry: WINDOW.to_string() };
        } else if m.window && add {
            let Kind::Action { next, .. } = &k else { unreachable!() };
            k = Kind::Action { next: *next, actor: wa, act: wait, params: Some(Cont { items: vec![("IsWaitEnd".to_string(), Val::Bool(1)), ("Frames".to_string(), Val::Int(0))] }) };
        } else if let Some(to) = redirect.get(&i) {
            if let Kind::Sub { entry, params, .. } | Kind::SubSwitch { entry, params, .. } = &mut k {
                *entry = to.clone();
                if m.show {
                    params.get_or_insert_with(|| Cont { items: Vec::new() }).items.push((SHOW.to_string(), Val::Arg(SHOW.to_string())));
                }
            }
        } else if m.show && !real
            && let Some(p) = k.params_mut()
        {
            for (_, v) in p.items.iter_mut() {
                if *v == Val::Arg(item.to_string()) {
                    *v = Val::Arg(SHOW.to_string());
                }
            }
        }
        f.events.push(Event { name: format!("OOTRRW{}", f.events.len()), kind: k });
    }
    let subs: Vec<u16> = src.subs.iter().map(|&x| at_of(x)).collect();
    f.entries.push(Entry { name: w.clone(), subs, vars: src.vars.clone(), main: at_of(src.main), pad: src.pad });
    Ok(w)
}
