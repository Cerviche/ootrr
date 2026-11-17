use crate::bfevfl::Bf;
use crate::error::{Error, Res};
use crate::qol::{edit_zs, Have};
use crate::romfs::Romfs;
use crate::staging::Out;

use super::guard::{self, Run};

pub const FIRE_FILE: &str = "Event/EventFlow/Get_StoneOfFire.bfevfl.zs";
pub const FIRE_FLOW: &str = "Get_StoneOfFire";
pub const DANCE_FILE: &str = "Event/EventFlow/Dm_Goron_0030.bfevfl.zs";
pub const DANCE_FLOW: &str = "Dm_Goron_0030";
pub const FIRE: &str = "Get_StoneOfFire";
pub const SWITCH: &str = "Event39";
pub const AFTER_SARIA_EXP: &str = "Get_StoneOfFire_IsAfter_SariaMelodyPlay_Exp";
pub const PLAYED: &str = "HavePlayedEvent.Dm_Goron_0030";
pub const BOOL_QUERY: &str = "EventQueryIsGameDataBool";
pub const STEP: &str = "DodongoCavernOpne";
pub const PREFIX: &str = "OOTRRDanceGuard";

fn flow_edit(raw: &[u8], f: impl FnOnce(&mut Bf) -> Res<()>) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    f(&mut b)?;
    let out: Vec<u8> = b.write()?;
    if Bf::parse(&out)? != b {
        return Err(Error::Format("darunia: the edited flow does not parse back"));
    }
    Ok(out)
}

pub fn requery_bytes(raw: &[u8]) -> Res<Vec<u8>> {
    flow_edit(raw, |b| guard::requery(b, FIRE_FLOW, SWITCH, AFTER_SARIA_EXP, BOOL_QUERY, PLAYED))
}

pub fn guard_bytes(raw: &[u8]) -> Res<Vec<u8>> {
    flow_edit(raw, |b| guard::guard_steps(b, DANCE_FLOW, FIRE, &[STEP], AFTER_SARIA_EXP, Run::WhenFalse, PREFIX).map(|_| ()))
}

pub fn stage(rf: &mut Romfs, have: Have) -> Res<Vec<Out>> {
    Ok(vec![edit_zs(rf, have, FIRE_FILE, requery_bytes)?, edit_zs(rf, have, DANCE_FILE, guard_bytes)?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfevfl::{Flow, Kind};
    use std::fs;
    use std::path::PathBuf as PB;

    fn real() -> Option<PB> {
        let p: PB = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(p)
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    fn flow_of<'a>(b: &'a Bf, n: &str) -> &'a Flow {
        b.flows.iter().find(|f| f.name == n).unwrap()
    }

    fn cases(f: &Flow, n: &str) -> Vec<(u32, String)> {
        let Kind::Switch { cases, .. } = &f.events.iter().find(|e| e.name == n).unwrap().kind else { panic!() };
        cases.iter().map(|c| (c.val, f.events[c.ev as usize].name.clone())).collect()
    }

    #[test]
    fn the_dance_follows_its_own_flag_and_only_writes_the_step_before_the_song_is_learned() {
        let Some(root) = real() else {
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let outs: Vec<Out> = stage(&mut rf, &|_: &str| None).unwrap();
        assert_eq!(outs.iter().map(|o| o.path.as_str()).collect::<Vec<&str>>(), vec![FIRE_FILE, DANCE_FILE]);
        let fire_old: Bf = Bf::parse(&rf.read_zs(FIRE_FILE).unwrap()).unwrap();
        let fire_new: Bf = Bf::parse(&rf.zs.decompress(&outs[0].z).unwrap()).unwrap();
        let (a, b): (&Flow, &Flow) = (flow_of(&fire_old, FIRE_FLOW), flow_of(&fire_new, FIRE_FLOW));
        assert_eq!(a.events.len(), b.events.len());
        assert_eq!(cases(a, SWITCH), cases(b, SWITCH));
        let e = b.events.iter().find(|e| e.name == SWITCH).unwrap();
        assert_eq!(guard::text(&e.kind, "GameDataName"), Some(PLAYED));
        let Kind::Switch { actor, query, .. } = &e.kind else { panic!() };
        assert_eq!(b.actors[*actor as usize].queries[*query as usize], BOOL_QUERY);
        assert_eq!(a.events.iter().zip(b.events.iter()).filter(|(x, y)| x != y).count(), 1);
        let dance_old: Bf = Bf::parse(&rf.read_zs(DANCE_FILE).unwrap()).unwrap();
        let dance_new: Bf = Bf::parse(&rf.zs.decompress(&outs[1].z).unwrap()).unwrap();
        let (c, d): (&Flow, &Flow) = (flow_of(&dance_old, DANCE_FLOW), flow_of(&dance_new, DANCE_FLOW));
        assert_eq!(d.events.len(), c.events.len() + 1);
        assert_eq!(d.actors.len(), c.actors.len() + 1);
        assert_eq!(cases(d, "OOTRRDanceGuard0"), vec![(0, "Event128".to_string()), (1, "Event121".to_string())]);
        let steps = |f: &Flow| -> Vec<String> { f.events.iter().filter_map(|e| guard::step_of(f, e, FIRE).map(|s| s.to_string())).collect() };
        assert_eq!(steps(c), steps(d));
        let g: &crate::bfevfl::Event = d.events.last().unwrap();
        assert_eq!(guard::text(&g.kind, "GameDataName"), Some(AFTER_SARIA_EXP));
        let out: PB = PB::from(env!("CARGO_MANIFEST_DIR")).join("../../target/darunia_flow/Event/EventFlow");
        fs::create_dir_all(&out).unwrap();
        for o in &outs {
            fs::write(out.join(o.path.rsplit('/').next().unwrap()), &o.z).unwrap();
        }
        assert!(stage(&mut rf, &|p: &str| outs.iter().find(|o| o.path == p).map(|o| o.z.clone())).is_err());
    }
}
