//! Ganon's Boss Key from the Light Arrows cutscene (Ganon's Boss Key: "LACS - Dungeon Rewards").
//!
//! The cutscene plays once the player holds `lacs_rewards` Stones and Medallions together
//! (crate::medal_gate builds that formula) and gives Ganon's Castle Boss Key in place of the
//! Light Arrows, which join the item pool.
//!
//! The game's events only hand out pouch items, and the generic pouch add does not know which
//! dungeon a key belongs to (see SARIA_REFUSES in crate::slice::flow_rules). So the scene's grant,
//! `GetPouchContent:GetPouchContentByActorName`, becomes the get window Saria's gift uses,
//! `GetPouchContent:DisplayGetByActorName` showing the boss key, followed by the flag write a
//! credited boss key chest makes (crate::dungeon_grant): DungeonInfo.InsideGanonCastle.IsGetBossKey.
//! The fanfare and the get effect of the scene name the boss key too.

use std::collections::BTreeSet as BS;

use crate::bfevfl::{Bf, Cont, Flow, Kind, Val};
use crate::dungeon_credit::{action_slot, flag, idx, push, set_bool, HOLDER, SET};
use crate::dungeon_grant::Kind as Grant;
use crate::error::{Error as E, Res};
use crate::logic_items::ItemId;
use crate::romfs::Romfs;
use crate::slice::support::{Keep, Plan};
use crate::staging::Out;
use crate::world::{LocId, Placement, World};

/// The Ganon's Boss Key choice that puts the key on the Light Arrows cutscene.
pub const VALUE: &str = "lacs_dungeons";
pub const LOC: &str = "ToT Light Arrows Cutscene";
pub const ITEM: &str = "Boss Key (Ganons Castle)";
pub const FILE: &str = "Dm_TimeTemple_0130.bfevfl";
pub const GANON: &str = "InsideGanonCastle";
pub const KEY_ACTOR: &str = "Item_Dungeon_BossKey";
pub const NOTE: &str = "Ganon's Castle Boss Key is given by Zelda in the Light Arrows Cutscene, which plays once you hold the set number of Stones and Medallions together. The Light Arrows are shuffled into the item pool. The scene shows the boss key's get window and sets Ganon's Castle boss key flag in place of its usual item. Experimental and untested in the game.";
pub const NEEDS: &str = "Ganon's Boss Key on the Light Arrows Cutscene needs Story and NPC rewards shuffled, so the cutscene can hold the key, and Chests shuffled in dungeons, so the key's own chest can take another item";

const POUCH: &str = "GetPouchContent";
const GRANT: (usize, &str, &str) = (8, "Event85", "GetPouchContentByActorName");
const SHOW: &str = "DisplayGetByActorName";
const DISPLAYS: [(usize, &str); 2] = [(14, "Event137"), (25, "Event17")];
const OLD: &str = "Item_Arrow_Light";
const PARAM: &str = "ActorName";

pub fn path() -> String {
    crate::flow_locations::path_of(FILE)
}

fn actor_name(c: &mut Cont) -> Res<&mut String> {
    c.items
        .iter_mut()
        .find(|(k, _)| k == PARAM)
        .and_then(|(_, v)| match v {
            Val::Str(s) | Val::Actor(s, _) => Some(s),
            _ => None,
        })
        .ok_or(E::Format("lacs key: an event of the Light Arrows scene has no ActorName"))
}

fn named(f: &Flow, i: usize, name: &str) -> Res<()> {
    match f.events.get(i) {
        Some(e) if e.name == name => Ok(()),
        _ => Err(E::Msg(format!("lacs key: event {i} of the Light Arrows scene is not {name}, the game files differ"))),
    }
}

/// Edits the scene's flow: the grant becomes the key's get window followed by the flag write,
/// and the two display events name the key.
pub fn edit_flow(f: &mut Flow) -> Res<()> {
    let (g, name, entry) = GRANT;
    named(f, g, name)?;
    let end: u16 = match &f.events[g].kind {
        Kind::Sub { next, flow, entry: e, .. } if flow == POUCH && e == entry => *next,
        _ => return Err(E::Format("lacs key: the Light Arrows grant is not GetPouchContent:GetPouchContentByActorName, the game files differ")),
    };
    let (a, set): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let at: u16 = idx(f.events.len())?;
    push(f, Kind::Action { next: end, actor: a, act: set, params: Some(set_bool(flag(GANON, Grant::Boss.member()))) })?;
    let ev = &mut f.events[g];
    if let Kind::Sub { next, params, entry: e, .. } = &mut ev.kind {
        let c: &mut Cont = params.as_mut().ok_or(E::Format("lacs key: the Light Arrows grant has no parameters"))?;
        let s: &mut String = actor_name(c)?;
        if s != OLD {
            return Err(E::Msg(format!("lacs key: the Light Arrows grant hands out {s} and not {OLD}, the game files differ")));
        }
        *s = KEY_ACTOR.to_string();
        *e = SHOW.to_string();
        *next = at;
    }
    for (i, name) in DISPLAYS {
        named(f, i, name)?;
        let c: &mut Cont = f.events[i].kind.params_mut().ok_or(E::Format("lacs key: a Light Arrows display event has no parameters"))?;
        let s: &mut String = actor_name(c)?;
        if s != OLD {
            return Err(E::Msg(format!("lacs key: {name} of the Light Arrows scene shows {s} and not {OLD}, the game files differ")));
        }
        *s = KEY_ACTOR.to_string();
    }
    if let Some(m) = crate::dungeon_credit::sound(f).first() {
        return Err(E::Msg(format!("lacs key: the Light Arrows scene is broken after the edit, {m}")));
    }
    Ok(())
}

pub fn edit_bytes(raw: &[u8]) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    let f: &mut Flow = b.flows.first_mut().ok_or(E::Format("lacs key: the Light Arrows scene file has no flow"))?;
    edit_flow(f)?;
    let out: Vec<u8> = b.write()?;
    if Bf::parse(&out)? != b {
        return Err(E::Format("lacs key: the edited Light Arrows scene does not parse back"));
    }
    Ok(out)
}

/// Stages the scene edit, and the "Ganon's Castle Boss Key" text when no chest already added it.
pub fn stage(rf: &mut Romfs, more: &mut Vec<Out>, marked: &BS<(Grant, &'static str)>) -> Res<()> {
    crate::dungeon_grant::merge(rf, more, &path(), edit_bytes)?;
    let want: BS<(Grant, &'static str)> = [(Grant::Boss, GANON)].into_iter().filter(|x| !marked.contains(x)).collect();
    crate::dungeon_grant::stage_text(rf, more, &want)
}

/// The fill lock that puts Ganon's Boss Key on the Light Arrows cutscene. The cutscene and the
/// key's own chest must both be shuffled, so the key is in the pool and the Light Arrows take
/// its place there.
pub fn forced(w: &World, p: &Plan, free: &[LocId], base: &Placement, plan: &[(LocId, ItemId)]) -> Res<Vec<(LocId, ItemId)>> {
    if p.keys.ganon != Keep::Lacs {
        return Ok(Vec::new());
    }
    let (Some(l), Some(k)) = (w.loc(LOC), w.item(ITEM)) else {
        return Err(E::Msg(format!("{LOC} or {ITEM} is not in the logic world")));
    };
    if let Some((_, i)) = plan.iter().find(|x| x.0 == l) {
        return if *i == k { Ok(Vec::new()) } else { Err(E::Msg(format!("plando: {LOC} holds Ganon's Boss Key with these settings, so it cannot hold {}", w.items.names[*i]))) };
    }
    if !free.contains(&l) || !free.iter().any(|x| base.get(*x) == Some(k)) {
        return Err(E::Msg(NEEDS.to_string()));
    }
    Ok(vec![(l, k)])
}

/// True when the seed puts Ganon's Boss Key on the Light Arrows cutscene.
pub fn placed(seed: &crate::settings::seed::SeedFile) -> bool {
    seed.placements.get(LOC).is_some_and(|i| i == ITEM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfevfl::flow::NONE;

    fn real() -> Option<Romfs> {
        let p = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Romfs::open(&p).ok()
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    #[test]
    fn the_light_arrows_scene_shows_the_key_and_sets_ganons_key_flag() {
        let Some(mut rf) = real() else { return };
        let raw: Vec<u8> = rf.read_zs(&path()).unwrap();
        let out: Vec<u8> = edit_bytes(&raw).unwrap();
        let b: Bf = Bf::parse(&out).unwrap();
        let f: &Flow = &b.flows[0];
        let Kind::Sub { next, entry, params, .. } = &f.events[GRANT.0].kind else { panic!("the grant is not a sub call") };
        assert_eq!(entry, SHOW);
        assert!(params.as_ref().unwrap().items.iter().any(|(k, v)| k == PARAM && matches!(v, Val::Str(s) if s == KEY_ACTOR)));
        let Kind::Action { params, next: end, .. } = &f.events[*next as usize].kind else { panic!("no flag write after the window") };
        assert_eq!(*end, NONE);
        assert!(params.as_ref().unwrap().items.iter().any(|(k, v)| k == "GameDataName" && matches!(v, Val::Str(s) if s == "DungeonInfo.InsideGanonCastle.IsGetBossKey")));
        let mut b2: Bf = Bf::parse(&out).unwrap();
        assert!(edit_flow(&mut b2.flows[0]).is_err(), "a second edit is refused");
    }
}
