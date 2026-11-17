//! Lets an NPC hold the Ocarina up.
//!
//! Every NPC gift that has Link hold the item up (each Dm_SY_GetInEvent entry reaches
//! :Item_Position) shows the item with Player_Event.EventIgnitionShowItem, IsAsPlay true and
//! ASCommand Rotate_GetDemo, which plays that animation on the actor it was handed. The two
//! Ocarina item actors are equipment accessories with no animation at all, so the game crashed.
//! Giving those actors the item animation components (ASInfo, ASRef Item_Common, AnimationRef)
//! still crashed in the game, right after the patched pack loaded.
//!
//! The game's own hold up gifts also hand out pickup actors: the Death Mountain Crater scrub's
//! sale gives Obj_Heart_Piece. A pickup actor is built on Obj_Item_Base with the shared Item_Common
//! animation, and its PouchContentParam names the item it adds (PouchActor). So at a hold up gift
//! the Ocarina is handed out as its pickup actor from the pickup catalog (Obj_Ocarina_Fairy,
//! Obj_Ocarina_TimeLand): the chain's ownership questions still ask about Item_Ocarina_Fairy and
//! Item_Ocarina_Time, and each branch hands out the matching pickup actor. The seed build writes
//! both pickup packs, since the chain picks the tier in the game. The Ocarina actors stay vanilla.

use crate::settings::seed::SeedFile;
use crate::slice::data::Data;
use crate::slice::key_spots::{pick, Pick};

/// The item actors with no animation that a hold up cannot show.
pub const NO_ANIM: [&str; 2] = ["Item_Ocarina_Fairy", "Item_Ocarina_Time"];

/// The player's equipped Ocarina. Adding the pickup actor gives the Ocarina but leaves this empty,
/// so the D-pad cannot play it; the starting items write it the same way (start_items::give).
pub const EQUIP: &str = "PlayerStatus.EquipOcarina";

/// The game data the gift writes after handing out `tier`'s pickup actor, if any.
pub fn equip_key(tier: &str) -> Option<&'static str> {
    NO_ANIM.contains(&tier).then_some(EQUIP)
}

pub const NOTE: &str = "An Ocarina is handed out by an NPC who holds the item up, so it is handed out as its pickup actor (the Ocarina actors have no item animation), and both Ocarina pickup packs are written into the mod. Experimental, untested in the game.";

/// The pickup actor a hold up hands out in place of `actor`, if `actor` cannot be shown.
pub fn shown(actor: &str) -> Option<String> {
    if !NO_ANIM.contains(&actor) {
        return None;
    }
    pick(actor).map(|p| p.pack)
}

/// Whether `actor` is one of the pickup actors handed out in place of an Ocarina.
pub fn is_form(actor: &str) -> bool {
    NO_ANIM.iter().any(|a| shown(a).as_deref() == Some(actor))
}

fn holds_up(d: &Data, loc: &str) -> bool {
    d.flow.by_name(loc).is_some_and(crate::slice::flow_rules::holds_up)
}

/// The actor `loc` hands out for `actor`: its pickup actor at a hold up gift.
pub fn shown_at(d: &Data, loc: &str, actor: &str) -> Option<String> {
    if !holds_up(d, loc) {
        return None;
    }
    shown(actor)
}

/// The actors each tier's branch of the progressive chain at `loc` hands out, or nothing when the
/// tiers themselves can be shown.
pub fn forms(d: &Data, loc: &str, family: &str) -> Vec<String> {
    if !holds_up(d, loc) {
        return Vec::new();
    }
    let Ok(t) = crate::progressive::tiers(family) else {
        return Vec::new();
    };
    if !t.iter().any(|a| NO_ANIM.contains(a)) {
        return Vec::new();
    }
    t.iter().map(|a| shown(a).unwrap_or_else(|| a.to_string())).collect()
}

fn ocarina(d: &Data, item: &str) -> bool {
    d.item(item).is_some_and(|i| i.actor.iter().chain(i.tiers.iter()).any(|a| NO_ANIM.contains(&a.as_str())))
}

/// The hold up locations that hand out an Ocarina.
pub fn spots(d: &Data, seed: &SeedFile) -> Vec<String> {
    seed.placements.iter().filter(|(loc, item)| ocarina(d, item) && holds_up(d, loc) && d.vanilla_of(loc) != Some(item.as_str())).map(|(l, _)| l.clone()).collect()
}

/// Whether the seed puts an Ocarina (any tier) at an NPC gift that holds the item up.
pub fn needed(d: &Data, seed: &SeedFile) -> bool {
    !spots(d, seed).is_empty()
}

/// The pickup packs the seed build writes for those locations: every Ocarina tier's, at each.
pub fn wanted(d: &Data, seed: &SeedFile) -> Vec<(String, Pick)> {
    let mut out: Vec<(String, Pick)> = Vec::new();
    for loc in spots(d, seed) {
        for a in NO_ANIM {
            if let Some(p) = pick(a) {
                out.push((loc.clone(), p));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hold_up_gift_hands_out_the_ocarina_pickup_actors() {
        let d: Data = Data::load().unwrap();
        assert_eq!(shown("Item_Ocarina_Fairy").as_deref(), Some("Obj_Ocarina_Fairy"));
        assert_eq!(shown("Item_Ocarina_Time").as_deref(), Some("Obj_Ocarina_TimeLand"));
        assert_eq!(shown("Item_Hookshot"), None);
        assert!(is_form("Obj_Ocarina_TimeLand") && !is_form("Obj_Heart_Piece"));
        assert_eq!(forms(&d, "Kak Man on Roof", "Ocarina"), vec!["Obj_Ocarina_Fairy".to_string(), "Obj_Ocarina_TimeLand".to_string()]);
        assert!(forms(&d, "Kak Man on Roof", "Hookshot").is_empty());
        // Saria's gift shows a get window, no hold up.
        assert!(forms(&d, "LW Gift from Saria", "Ocarina").is_empty());
        assert_eq!(crate::slice::flow_pool::dest_form(&d, "Ocarina", "Kak Anju as Child", "Item_Ocarina_Fairy".to_string()), "Obj_Ocarina_Fairy");
    }

    #[test]
    fn a_seed_with_the_ocarina_at_the_man_on_the_roof_builds_both_pickup_packs_and_hands_them_out() {
        let root: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("D:/Oot/romfs"));
        if !root.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", root.display());
            return;
        }
        let d: Data = Data::load().unwrap();
        let mut g = crate::slice::generate::generate_data(&d, &crate::settings::Settings::default(), "ocarina held", &crate::slice::GenOpts::default()).unwrap();
        // Trade one Ocarina copy with the man on the roof's item; only the build is tested here.
        let roof: &str = "Kak Man on Roof";
        let at: String = g.seed_file.placements.iter().find(|(_, i)| i.as_str() == "Ocarina").map(|(l, _)| l.clone()).unwrap();
        let there: String = g.seed_file.placements[roof].clone();
        g.seed_file.placements.insert(at, there);
        g.seed_file.placements.insert(roof.to_string(), "Ocarina".to_string());
        assert!(needed(&d, &g.seed_file));
        let load: std::path::PathBuf = std::env::temp_dir().join(format!("ootrr_ocarina_hold_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&load);
        let b = crate::slice::edit::build(&g.seed_file, &root, &load, "ocarina").unwrap();
        let romfs: std::path::PathBuf = b.root.join("romfs");
        for a in NO_ANIM {
            let p: std::path::PathBuf = format!("Pack/Actor/{}.pack.zs", shown(a).unwrap()).split('/').fold(romfs.clone(), |x, s| x.join(s));
            assert!(p.is_file(), "{a}'s pickup pack not written");
            let item: std::path::PathBuf = format!("Pack/Actor/{a}.pack.zs").split('/').fold(romfs.clone(), |x, s| x.join(s));
            assert!(!item.is_file(), "{a}'s own pack stays vanilla");
        }
        // The man on the roof's gift hands out the pickup actors.
        let l = d.flow.by_name(roof).unwrap();
        let file: String = crate::flow_locations::path_of(&l.edits[0].file);
        let mut rf: crate::romfs::Romfs = crate::romfs::Romfs::open(&root).unwrap();
        let z: Vec<u8> = std::fs::read(file.split('/').fold(romfs.clone(), |x, s| x.join(s))).unwrap();
        let raw: Vec<u8> = rf.zs.decompress(&z).unwrap();
        let bf = crate::bfevfl::Bf::parse(&raw).unwrap();
        // Hand out events only: the chain's ownership questions still ask about the item actors.
        let named = |n: &str| {
            bf.flows.iter().flat_map(|f| f.events.iter()).filter(|e| !matches!(e.kind, crate::bfevfl::Kind::Switch { .. })).any(|e| e.kind.params().and_then(|c| c.get(&l.edits[0].param)) == Some(&crate::bfevfl::Val::Str(n.to_string())))
        };
        assert!(named("Obj_Ocarina_Fairy") && named("Obj_Ocarina_TimeLand"));
        // Each pickup branch then equips its Ocarina tier, as the starting items do.
        for t in NO_ANIM {
            let equips = bf.flows.iter().flat_map(|f| f.events.iter()).any(|e| {
                e.kind.params().is_some_and(|c| c.get("GameDataName") == Some(&crate::bfevfl::Val::Str(EQUIP.to_string())) && c.get("Value") == Some(&crate::bfevfl::Val::Str(t.to_string())))
            });
            assert!(equips, "{t} is not equipped after its pickup actor");
        }
        assert!(!named("Item_Ocarina_Fairy"));
        let _ = std::fs::remove_dir_all(&load);
    }
}

#[cfg(test)]
mod pickup_tests {
    use crate::settings::{Settings, Value};
    use crate::slice::data::Data;

    #[test]
    fn a_generated_seed_with_an_ocarina_at_a_pickup_spot_builds_its_pickup_pack() {
        let root: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("D:/Oot/romfs"));
        if !root.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", root.display());
            return;
        }
        let d: Data = Data::load().unwrap();
        let mut s: Settings = Settings::default();
        s.set("shuffle_freestanding_items", Value::s("all")).unwrap();
        s.set("shuffle_pots", Value::s("all")).unwrap();
        let spot = |loc: &str| d.pots.has(loc) || d.cat.by_name(loc).is_some_and(crate::slice::pickups::is_world);
        let mut found = None;
        for n in 0..30 {
            let g = crate::slice::generate::generate_data(&d, &s, &format!("ocarina spot {n}"), &crate::slice::GenOpts::default()).unwrap();
            if let Some((l, _)) = g.seed_file.placements.iter().find(|(l, i)| i.as_str() == "Ocarina" && spot(l)) {
                found = Some((l.clone(), g));
                break;
            }
        }
        let Some((loc, g)) = found else {
            panic!("no seed put an Ocarina at a freestanding spot or a pot in 30 tries");
        };
        let load: std::path::PathBuf = std::env::temp_dir().join(format!("ootrr_ocarina_spot_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&load);
        let b = crate::slice::edit::build(&g.seed_file, &root, &load, "ocarina spot").unwrap_or_else(|e| panic!("{loc}: {e}"));
        let actor = b.root.join("romfs").join("Pack").join("Actor");
        assert!(actor.join("Obj_Ocarina_Fairy.pack.zs").is_file() || actor.join("Obj_Ocarina_TimeLand.pack.zs").is_file(), "{loc}");
        let _ = std::fs::remove_dir_all(&load);
    }
}
