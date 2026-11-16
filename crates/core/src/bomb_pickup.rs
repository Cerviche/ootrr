use crate::byml::{Byml, Node};
use crate::error::{Error, Res};
use crate::pickup_pack::kit::AT_MAX;
use crate::pot_slice::targets::Give;
use crate::romfs::Romfs;
use crate::sarc;
use crate::staging::{rewrite, Out};

pub const ACTOR: &str = "Obj_Bomb";
pub const PACK: &str = "Pack/Actor/Obj_Bomb.pack.zs";
pub const PARAM: &str = "Component/PouchContentParam/Obj_Bomb.game__component__PouchContentParam.bgyml";

pub fn needed(ts: &[Give]) -> bool {
    ts.iter().any(|t| t.actor == ACTOR)
}

pub fn quiet(raw: &[u8]) -> Res<(Vec<u8>, bool)> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let mut hit: bool = false;
    for e in es.iter_mut().filter(|e| e.name == PARAM) {
        let mut y: Byml = Byml::parse(&e.data)?;
        if y.root.get(AT_MAX) == Some(&Node::Bool(true)) {
            y.root.set(AT_MAX, Node::Bool(false));
            e.data = y.write()?;
            hit = true;
        }
    }
    Ok((sarc::write(&es)?, hit))
}

pub fn stage(rf: &mut Romfs) -> Res<Out> {
    let mut hit: bool = false;
    let o: Out = rewrite(rf, PACK, |raw| {
        let (b, h): (Vec<u8>, bool) = quiet(raw)?;
        hit = h;
        Ok(b)
    })?;
    if !hit {
        return Err(Error::Msg(format!("bomb pickup: {PARAM} no longer plays the get event at a full pouch, the game files may have changed")));
    }
    Ok(o)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf as PB;

    use super::*;
    use crate::pot_slice::data::Kind;

    fn param(at_max: bool) -> Vec<u8> {
        let y: Byml = Byml {
            root: Node::Map(vec![
                (AT_MAX.to_string(), Node::Bool(at_max)),
                ("PouchActor".to_string(), Node::Str("Work/Actor/Item_Bomb.engine__actor__ActorParam.gyml".to_string())),
            ]),
            version: 7,
            big: false,
        };
        y.write().unwrap()
    }

    fn give(actor: &str) -> Give {
        Give { name: "Pot".to_string(), kind: Kind::Pot, hash: 1, files: vec!["f".to_string()], was: String::new(), item: "Bombs (5)".to_string(), actor: actor.to_string() }
    }

    #[test]
    fn only_the_obj_bomb_pouch_file_stops_playing_the_get_event_at_a_full_pouch() {
        let other: String = "Component/PouchContentParam/Item_Bomb.game__component__PouchContentParam.bgyml".to_string();
        let es: Vec<sarc::Entry> = vec![sarc::Entry { name: PARAM.to_string(), data: param(true) }, sarc::Entry { name: other.clone(), data: param(true) }];
        let (out, hit): (Vec<u8>, bool) = quiet(&sarc::write(&es).unwrap()).unwrap();
        assert!(hit);
        let back: Vec<sarc::Entry> = sarc::parse(&out).unwrap();
        let get = |n: &str| -> Byml { Byml::parse(&sarc::get(&back, n).unwrap().data).unwrap() };
        assert_eq!(get(PARAM).root.get(AT_MAX), Some(&Node::Bool(false)));
        assert!(get(PARAM).root.get("PouchActor").is_some());
        assert_eq!(get(&other).root.get(AT_MAX), Some(&Node::Bool(true)));
        let (_, again): (Vec<u8>, bool) = quiet(&out).unwrap();
        assert!(!again);
    }

    #[test]
    fn the_edit_is_needed_only_when_a_breakable_drops_a_bomb() {
        assert!(needed(&[give("Obj_Rupee_Red"), give(ACTOR)]));
        assert!(!needed(&[give("Obj_Rupee_Red"), give("Obj_DekuNut")]));
        assert!(!needed(&[]));
    }

    #[test]
    fn every_bomb_item_on_a_pot_or_a_rock_drops_the_actor_the_edit_covers() {
        use std::collections::BTreeMap as BM;
        use crate::settings::seed::SeedFile;
        use crate::settings::Settings;
        use crate::slice::data::{Data, State};
        let d: Data = Data::load().unwrap();
        let mut pots: Vec<String> = d.pots.state.iter().filter(|(n, s)| **s == State::Eligible && d.pots.kind_of(n) == Some(Kind::Pot)).map(|(n, _)| n.clone()).collect();
        pots.sort();
        for item in ["Bomb (1)", "Bombs (5)", "Bombs (10)", "Bombs (20)"] {
            let m: BM<String, String> = [(pots[0].clone(), item.to_string())].into_iter().collect();
            let s: SeedFile = SeedFile::new("s", &Settings::default(), "g", m).unwrap();
            let ts: Vec<Give> = crate::pot_slice::targets::targets(&d, &s).unwrap();
            assert!(!ts.is_empty() && ts.iter().all(|t| t.actor == ACTOR), "{item}");
            assert!(needed(&ts), "{item}");
        }
        let m: BM<String, String> = [(pots[0].clone(), "Rupees (20)".to_string())].into_iter().collect();
        let s: SeedFile = SeedFile::new("s", &Settings::default(), "g", m).unwrap();
        assert!(!needed(&crate::pot_slice::targets::targets(&d, &s).unwrap()));
    }

    #[test]
    fn the_shipped_bomb_pickup_plays_its_get_event_at_a_full_pouch_until_the_edit() {
        let p: PB = std::env::var("OOTRR_ROMFS").map(PB::from).unwrap_or_else(|_| PB::from("C:/o/nup dump/updates/131072/romfs"));
        if !p.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            return;
        }
        let mut rf: Romfs = Romfs::open(&p).unwrap();
        let raw: Vec<u8> = rf.read_zs(PACK).unwrap();
        let es: Vec<sarc::Entry> = sarc::parse(&raw).unwrap();
        let before: &sarc::Entry = sarc::get(&es, PARAM).unwrap();
        let y: Byml = Byml::parse(&before.data).unwrap();
        assert_eq!(y.root.get(AT_MAX), Some(&Node::Bool(true)));
        assert_eq!(y.root.get("PouchActor").and_then(|v| v.as_str()), Some("Work/Actor/Item_Bomb.engine__actor__ActorParam.gyml"));
        assert!(y.root.get("IsCountCheckOnFirstGet").is_none());
        let o: Out = stage(&mut rf).unwrap();
        let after: Vec<sarc::Entry> = sarc::parse(&rf.zs.decompress(&o.z).unwrap()).unwrap();
        assert_eq!(after.len(), es.len());
        let a: &sarc::Entry = sarc::get(&after, PARAM).unwrap();
        assert!(a.data.len() <= before.data.len());
        assert_eq!(Byml::parse(&a.data).unwrap().root.get(AT_MAX), Some(&Node::Bool(false)));
        for e in &es {
            if e.name != PARAM {
                assert_eq!(sarc::get(&after, &e.name).map(|x| &x.data), Some(&e.data), "{}", e.name);
            }
        }
    }
}
