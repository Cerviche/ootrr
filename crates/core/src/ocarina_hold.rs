//! Lets an NPC hold the Ocarina up.
//!
//! Every NPC gift that has Link hold the item up (each Dm_SY_GetInEvent entry reaches
//! :Item_Position) shows the item with Player_Event.EventIgnitionShowItem, IsAsPlay true and
//! ASCommand Rotate_GetDemo, which plays that animation on the item actor itself. Item actors
//! carry it through three components their packs share: ASInfoRef
//! (Component/ASInfo/Item_GetDemo, 98 item packs), ASRef ?AS/Item_Common.root.asb (the shared
//! item animation set that holds Rotate_GetDemo) and AnimationRef (an AnimationParam that loads
//! Item_Common_Animation). The two Ocarina actors are built as equipment accessories and have none
//! of the three, so the hold up crashed the game; chests play Rotate_GetDemo on a separate guest
//! performer and never touch the actor's own animation.
//!
//! The Ocarina packs get the three components. Both files they point at are copied byte for byte
//! from the game's own packs (ASInfo from Item_Hookshot, the AnimationParam of Item_Bottle, which
//! loads Item_Common_Animation and the bottle's own animations), so the resource size table
//! already knows them; only the actor param and the pack itself grow.

use crate::byml::{Byml, Node};
use crate::error::{Error, Res};
use crate::romfs::Romfs;
use crate::sarc;
use crate::settings::seed::SeedFile;
use crate::slice::data::Data;
use crate::staging::{rewrite, Out};

pub const ACTORS: [&str; 2] = ["Item_Ocarina_Fairy", "Item_Ocarina_Time"];
const AS_INFO: (&str, &str) = ("Item_Hookshot", "Component/ASInfo/Item_GetDemo.engine__component__ASInfo.bgyml");
const ANIMATION: (&str, &str) = ("Item_Bottle", "Component/AnimationParam/Item_Bottle.engine__component__AnimationParam.bgyml");
const AS: &str = "?AS/Item_Common.root.asb";
const COMPONENTS: &str = "Components";

pub const NOTE: &str = "An Ocarina is handed out by an NPC who holds the item up, so both Ocarina actors get the item get animation every other item has (the shared Item_Common animation set). Experimental, untested in the game.";

pub fn pack_of(actor: &str) -> String {
    format!("Pack/Actor/{actor}.pack.zs")
}

fn param_of(actor: &str) -> String {
    format!("Actor/{actor}.engine__actor__ActorParam.bgyml")
}

/// The three component references, as the actor param holds them.
fn refs() -> [(&'static str, String); 3] {
    [("ASInfoRef", format!("?{}", AS_INFO.1)), ("ASRef", AS.to_string()), ("AnimationRef", format!("?{}", ANIMATION.1))]
}

/// Whether the seed puts an Ocarina (any tier) at an NPC gift that holds the item up.
pub fn needed(d: &Data, seed: &SeedFile) -> bool {
    seed.placements.iter().any(|(loc, item)| {
        d.item(item).is_some_and(|i| i.actor.iter().chain(i.tiers.iter()).any(|a| ACTORS.contains(&a.as_str())))
            && d.flow.by_name(loc).is_some_and(crate::slice::flow_rules::holds_up)
            && d.vanilla_of(loc) != Some(item.as_str())
    })
}

/// Adds the item animation components to one Ocarina actor pack.
pub fn patch(raw: &[u8], actor: &str, as_info: &[u8], animation: &[u8]) -> Res<Vec<u8>> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let name: String = param_of(actor);
    let e: &sarc::Entry = sarc::get(&es, &name).ok_or_else(|| Error::Msg(format!("ocarina: {actor}'s pack has no {name}")))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    let comps: &mut Node = y.root.get_mut(COMPONENTS).ok_or_else(|| Error::Msg(format!("ocarina: {name} has no {COMPONENTS}")))?;
    for (k, v) in refs() {
        if comps.get(k).is_some() {
            return Err(Error::Msg(format!("ocarina: {name} already has {k}")));
        }
        comps.set(k, Node::Str(v));
    }
    sarc::set(&mut es, &name, y.write()?);
    for (file, data) in [(AS_INFO.1, as_info), (ANIMATION.1, animation)] {
        if sarc::set(&mut es, file, data.to_vec()) {
            return Err(Error::Msg(format!("ocarina: {actor}'s pack already holds {file}")));
        }
    }
    sarc::write(&es)
}

fn donor(rf: &mut Romfs, (pack, file): (&str, &str)) -> Res<Vec<u8>> {
    let es: Vec<sarc::Entry> = sarc::parse(&rf.read_zs(&pack_of(pack))?)?;
    sarc::get(&es, file).map(|e| e.data.clone()).ok_or_else(|| Error::Msg(format!("ocarina: {pack}'s pack has no {file}")))
}

pub fn stage(rf: &mut Romfs) -> Res<Vec<Out>> {
    let as_info: Vec<u8> = donor(rf, AS_INFO)?;
    let animation: Vec<u8> = donor(rf, ANIMATION)?;
    ACTORS.iter().map(|a| rewrite(rf, &pack_of(a), |raw| patch(raw, a, &as_info, &animation))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump() -> Option<Romfs> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("D:/Oot/romfs"));
        if !p.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", p.display());
            return None;
        }
        Some(Romfs::open(&p).unwrap())
    }

    #[test]
    fn a_seed_with_the_ocarina_at_the_man_on_the_roof_builds_the_patched_ocarina_packs() {
        let Some(rf) = dump() else {
            return;
        };
        drop(rf);
        let root: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("D:/Oot/romfs"));
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
        for a in ACTORS {
            let p: std::path::PathBuf = pack_of(a).split('/').fold(b.root.join("romfs"), |x, s| x.join(s));
            assert!(p.is_file(), "{a} pack not written");
        }
        let _ = std::fs::remove_dir_all(&load);
    }

    #[test]
    fn the_real_ocarinas_get_the_item_animation_components_of_the_hookshot() {
        let Some(mut rf) = dump() else {
            return;
        };
        let hook: Byml = {
            let es = sarc::parse(&rf.read_zs(&pack_of("Item_Hookshot")).unwrap()).unwrap();
            Byml::parse(&sarc::get(&es, &param_of("Item_Hookshot")).unwrap().data).unwrap()
        };
        let outs: Vec<Out> = stage(&mut rf).unwrap();
        assert_eq!(outs.len(), 2);
        for (o, a) in outs.iter().zip(ACTORS) {
            assert!(o.new > o.old);
            let raw: Vec<u8> = rf.zs.decompress(&o.z).unwrap();
            let es = sarc::parse(&raw).unwrap();
            let y: Byml = Byml::parse(&sarc::get(&es, &param_of(a)).unwrap().data).unwrap();
            let comps = y.root.get(COMPONENTS).unwrap();
            assert_eq!(comps.get("ASRef"), hook.root.get(COMPONENTS).unwrap().get("ASRef"), "{a}");
            assert_eq!(comps.get("ASInfoRef"), hook.root.get(COMPONENTS).unwrap().get("ASInfoRef"), "{a}");
            assert!(sarc::get(&es, AS_INFO.1).is_some() && sarc::get(&es, ANIMATION.1).is_some(), "{a}");
            // The rest of the actor stays as it was: the pouch, model and accessory components.
            for k in ["PouchContentRef", "ModelInfoRef", "AccessoryRef"] {
                assert!(comps.get(k).is_some(), "{a} {k}");
            }
            assert!(patch(&raw, a, b"x", b"y").is_err(), "{a} is already patched");
        }
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
