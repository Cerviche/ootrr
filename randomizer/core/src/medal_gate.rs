use std::path::PathBuf as PB;

use crate::ainb::Ainb;
use crate::byml::{Byml, Node};
use crate::door_open::{multi_entry, SHEIK_FILE};
use crate::error::{Error, Res};
use crate::romfs::Romfs;
use crate::staging::{self, Out};
use crate::start_defaults::Edit;

pub const SETTING: &str = "shuffle_dungeon_rewards";
pub const LOGIC_KEY: &str = "medallion_gates";
pub const KAKARIKO_EXP: &str = "SageOfShadow_Kakariko_Burning";
pub const PRELUDE_EXP: &str = "AllSage_IsAfter_Step01_02_Exp";
pub const LACS_EXP: &str = "AllSage_IsAfter_Step02_Exp";
pub const FOREST_EXP: &str = "SageOfForest_IsCompleted_Exp";
pub const LACS_PLAYED: &str = "HavePlayedEvent.Dm_TimeTemple_0130";
pub const PRELUDE_MODULE: &str = "Logic/Market_Adult_Prefab_edc2.logic.module.ainb";
pub const LACS_MODULE: &str = "Logic/Market_Adult_Prefab_c668.logic.module.ainb";
pub const LACS_AREA: &str = "Area_a3ca";
pub const LACS_AREA_HASH: u64 = 11085071794120779663;
pub const ALL_SAGE: &str = "Work/Challenge/AllSage.game__challenge__Challenge.gyml";
pub const LACS_STEP: &str = "Step02";

const READ_EXP: &str = "HoldGameDataGetBoolExpSimple";
const EXP_IN: &str = "GameDataNameBoolExp";
const AREA: &str = "ActorLogicAreaActor";
const STEP_CHECK: &str = "HoldChallengeCurrentStepCheckOrder";
const HIT: &str = "HoldActorObserverBySensorHitInfo";
const AND: &str = "OperateAND";
const CREATE: &str = "Logic_Create";
const MULTI: &str = "BoolMulti";
const STR: usize = 3;
const BOOL: usize = 1;
const AND_OP: i32 = 3;
const NOT_OP: i32 = 5;

pub fn medal(n: &str) -> String {
    format!("IsGet.Item_Medal_{n}")
}

fn op(o: i32) -> (i32, String) {
    (o, String::new())
}

pub fn exps() -> Vec<Edit> {
    vec![
        Edit::Exp(KAKARIKO_EXP.to_string(), vec![(0, medal("Forest")), (0, medal("Fire")), (0, medal("Water")), op(AND_OP), op(AND_OP)]),
        Edit::Exp(PRELUDE_EXP.to_string(), vec![(0, medal("Forest"))]),
        Edit::Exp(LACS_EXP.to_string(), vec![(0, medal("Shadow")), (0, medal("Spirit")), op(AND_OP), (0, LACS_PLAYED.to_string()), op(NOT_OP), op(AND_OP)]),
    ]
}

fn one(hits: Vec<usize>, what: &str) -> Res<usize> {
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("medal gate: expected one {what}, found {}, the module is not the unpatched one", hits.len()))),
    }
}

pub fn patch_prelude(a: &mut Ainb) -> Res<()> {
    let i: usize = one(a.find_els(READ_EXP).into_iter().filter(|&i| a.in_str(i, EXP_IN) == Some(FOREST_EXP)).collect(), "Forest Sage read")?;
    let g: usize = a.find_in(i, STR, EXP_IN).ok_or(Error::Format("medal gate: the Forest Sage read has no flag input"))?;
    let o: u32 = a.pool.add(PRELUDE_EXP);
    a.set_in_val(STR, g, &o.to_le_bytes())
}

pub fn lacs_step(a: &Ainb) -> Res<usize> {
    one(
        a.find_els(STEP_CHECK).into_iter().filter(|&i| a.in_str(i, "ChallengeName") == Some(ALL_SAGE) && a.in_str(i, "ChallengeStepName") == Some(LACS_STEP)).collect(),
        "AllSage Step02 check",
    )
}

fn multi_range(a: &Ainb, i: usize) -> Option<(usize, usize)> {
    let m = a.input(BOOL, a.find_in(i, BOOL, MULTI)?)?;
    if m.src > -100 || m.plug < 1 {
        return None;
    }
    let start: usize = (-100 - i32::from(m.src)) as usize;
    Some((start, m.plug as usize))
}

pub fn reads_of(a: &Ainb, src: usize) -> usize {
    let mut n: usize = 0;
    for i in 0..a.els.len() {
        for k in 0..6 {
            n += a.input_ids(i, k).into_iter().filter(|&g| a.input(k, g).is_some_and(|x| x.src >= 0 && x.src as usize == src)).count();
        }
        n += a.params[i].plugs.iter().filter(|p| p.a as usize == src).count();
        if a.el_name(i) == AND
            && let Some((s, c)) = multi_range(a, i)
        {
            n += (s..s + c).filter(|&j| multi_entry(a, j).is_some_and(|(e, _)| e as usize == src)).count();
        }
    }
    n
}

pub fn patch_lacs(a: &mut Ainb) -> Res<usize> {
    let area: usize = one(a.find_els(AREA).into_iter().filter(|&i| a.prop_str(i, "InstanceName") == Some(LACS_AREA)).collect(), "Light Arrows area")?;
    let step: usize = lacs_step(a)?;
    let hit: usize = one(a.find_els(HIT).into_iter().collect(), "player sensor")?;
    let g: usize = a.find_in(area, BOOL, CREATE).ok_or(Error::Format("medal gate: the area has no create input"))?;
    let x = a.input(BOOL, g).ok_or(Error::Format("medal gate: input out of range"))?;
    if x.src < 0 || x.src as usize != step {
        return Err(Error::Format("medal gate: the area is not created from the AllSage Step02 check"));
    }
    a.force_bool(area, CREATE, true)?;
    let mut moved: usize = 0;
    for i in a.find_els(AND) {
        let Some((s, c)) = multi_range(a, i) else { continue };
        let entries: Vec<(u16, u16)> = (s..s + c).filter_map(|j| multi_entry(a, j)).collect();
        if !entries.iter().any(|(e, _)| *e as usize == step) {
            continue;
        }
        let from: usize = (s..s + c)
            .find(|&j| multi_entry(a, j).is_some_and(|(e, _)| e as usize == hit))
            .ok_or(Error::Format("medal gate: the gate that reads the step does not read the player sensor"))?;
        let rec: [u8; 4] = a.multi[8 * from..8 * from + 4].try_into().map_err(|_| Error::Format("medal gate: bad multi entry"))?;
        for j in s..s + c {
            if multi_entry(a, j).is_some_and(|(e, _)| e as usize == step) {
                a.multi[8 * j..8 * j + 4].copy_from_slice(&rec);
                moved += 1;
            }
        }
    }
    if reads_of(a, step) != 0 {
        return Err(Error::Format("medal gate: something still reads the AllSage Step02 check"));
    }
    Ok(moved)
}

pub fn patch_banc(y: &mut Byml) -> Res<()> {
    let actors: &mut Vec<Node> = y.root.get_mut("Actors").and_then(|n| n.as_arr_mut()).ok_or(Error::Format("medal gate: the adult Market map has no actors"))?;
    let a: &mut Node = actors
        .iter_mut()
        .find(|a| matches!(a.get("Hash"), Some(Node::U64(h)) if *h == LACS_AREA_HASH))
        .ok_or(Error::Format("medal gate: the adult Market map has no Light Arrows area"))?;
    if a.get("Gyaml").and_then(|g| g.as_str()) != Some("Area") || a.get("Presence").is_some() {
        return Err(Error::Format("medal gate: the Light Arrows area is not the unpatched one"));
    }
    let p: Node = Node::Map(vec![
        ("FlagName".to_string(), Node::Str(LACS_EXP.to_string())),
        ("IsCheckLazy".to_string(), Node::Bool(false)),
        ("IsNegation".to_string(), Node::Bool(false)),
    ]);
    a.set("Presence", p);
    Ok(())
}

pub fn banc_bytes(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    patch_banc(&mut y)?;
    let out: Vec<u8> = y.write()?;
    if Byml::parse(&out)?.root != y.root {
        return Err(Error::Format("medal gate: the patched map does not read back"));
    }
    Ok(out)
}

fn module(rf: &mut Romfs, path: &str, f: impl FnOnce(&mut Ainb) -> Res<()>) -> Res<Out> {
    let raw: Vec<u8> = rf.read(path)?;
    let mut a: Ainb = Ainb::parse(&raw)?;
    f(&mut a)?;
    let out: Vec<u8> = a.write()?;
    let back: Ainb = Ainb::parse(&out)?;
    if back.write()? != out || !crate::ainb::audit(&back).is_empty() {
        return Err(Error::Msg(format!("medal gate: the patched module {path} does not read back")));
    }
    Ok(Out { path: path.to_string(), old: raw.len(), new: out.len(), z: out })
}

pub fn stage(rf: &mut Romfs, more: &mut Vec<Out>, from: Option<&PB>) -> Res<()> {
    more.push(module(rf, PRELUDE_MODULE, patch_prelude)?);
    more.push(module(rf, LACS_MODULE, |a| patch_lacs(a).map(|_| ()))?);
    match more.iter().position(|o| o.path == SHEIK_FILE) {
        Some(i) => {
            let raw: Vec<u8> = rf.zs.decompress(&more[i].z)?;
            let out: Vec<u8> = banc_bytes(&raw)?;
            let old: usize = more[i].old;
            more[i] = staging::banc_out(rf, SHEIK_FILE, old, &out)?;
        }
        None => {
            let raw: Vec<u8> = staging::banc_raw(rf, SHEIK_FILE, from)?;
            let out: Vec<u8> = banc_bytes(&raw)?;
            more.push(staging::banc_out(rf, SHEIK_FILE, raw.len(), &out)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_sim::db::Db;
    use crate::flow_sim::expr::truth;
    use crate::flow_sim::state::{State, V};
    use crate::logic_items::ItemId;
    use crate::search::{explore, Search};
    use crate::settings::{Settings, Value};
    use crate::slice::data::Data;
    use crate::slice::logic::world_with;
    use crate::world::{Placement, World};

    fn real() -> Option<Romfs> {
        let p: PB = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Romfs::open(&p).ok()
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    fn edited_list(rf: &mut Romfs) -> Byml {
        let mut y: Byml = Byml::parse(&rf.read_zs(crate::chest_slot::LIST).unwrap()).unwrap();
        assert_eq!(crate::start_defaults::apply(&mut y, &exps()).unwrap(), 3);
        y
    }

    fn with(db: &mut Db, pairs: &[(&str, &str)]) -> State {
        let mut st: State = State::new();
        for (k, v) in pairs {
            let h: u32 = db.reg(k);
            let x: V = db.parse(h, v).unwrap();
            st.v.insert(h, x);
        }
        st
    }

    fn holds(db: &mut Db, exp: &str, pairs: &[(&str, &str)]) -> bool {
        let st: State = with(db, pairs);
        let h: u32 = db.reg(exp);
        truth(db, &st, h, 0).unwrap()
    }

    const SAGES: [(&str, &str); 3] = [("Step_SageOfForest", "Complete"), ("Step_SageOfFire", "Complete"), ("Step_SageOfWater", "Complete")];

    #[test]
    fn the_three_formulas_follow_the_medallion_items_and_ignore_the_sage_quests() {
        let Some(mut rf) = real() else {
            return;
        };
        let mut van: Db = Db::from_list(&Byml::parse(&rf.read_zs(crate::chest_slot::LIST).unwrap()).unwrap()).unwrap();
        assert!(holds(&mut van, KAKARIKO_EXP, &SAGES));
        let medals: Vec<String> = ["Forest", "Fire", "Water"].iter().map(|m| medal(m)).collect();
        let owned: Vec<(&str, &str)> = medals.iter().map(|m| (m.as_str(), "true")).collect();
        assert!(!holds(&mut van, KAKARIKO_EXP, &owned));
        let mut db: Db = Db::from_list(&edited_list(&mut rf)).unwrap();
        assert!(!holds(&mut db, KAKARIKO_EXP, &SAGES));
        assert!(holds(&mut db, KAKARIKO_EXP, &owned));
        for gone in 0..3 {
            let some: Vec<(&str, &str)> = owned.iter().enumerate().filter(|(i, _)| *i != gone).map(|(_, p)| *p).chain(SAGES).collect();
            assert!(!holds(&mut db, KAKARIKO_EXP, &some), "{}", medals[gone]);
        }
        let forest: String = medal("Forest");
        assert!(holds(&mut db, PRELUDE_EXP, &[(forest.as_str(), "true")]));
        assert!(!holds(&mut db, PRELUDE_EXP, &[("Step_SageOfForest", "Complete")]));
        let (sh, sp): (String, String) = (medal("Shadow"), medal("Spirit"));
        let both: [(&str, &str); 2] = [(sh.as_str(), "true"), (sp.as_str(), "true")];
        assert!(holds(&mut db, LACS_EXP, &both));
        assert!(!holds(&mut db, LACS_EXP, &[(sh.as_str(), "true"), ("HavePlayedEvent.Dm_Awake_Spirit", "true"), ("HavePlayedEvent.Dm_Awake_Shadow", "true")]));
        assert!(!holds(&mut db, LACS_EXP, &[(sp.as_str(), "true"), ("Step_AllSage", "Step02")]));
        assert!(!holds(&mut db, LACS_EXP, &[both[0], both[1], (LACS_PLAYED, "true")]));
    }

    #[test]
    fn the_saria_talk_that_reads_the_kakariko_fire_follows_the_medallions_in_the_flow_sim() {
        let Some(mut rf) = real() else {
            return;
        };
        let mut db: Db = Db::from_list(&edited_list(&mut rf)).unwrap();
        db.load_deps(&mut rf).unwrap();
        let medals: Vec<String> = ["Forest", "Fire", "Water"].iter().map(|m| medal(m)).collect();
        let mut run = |pairs: &[(&str, &str)]| -> Vec<String> {
            let st: State = with(&mut db, pairs);
            let src = crate::flow_sim::load::Disk::new(Romfs::open(&rf.root).unwrap(), None, Vec::new());
            let mut sim = crate::flow_sim::run::Sim::new(db.clone(), Box::new(src), crate::flow_sim::run::Opts::default());
            let m = sim.start("Dm_SY_Ocarina_SariaSong", "SariaSong_Child_Story", st, Default::default()).unwrap();
            let mut out: Vec<String> = sim
                .explore(m)
                .into_iter()
                .flat_map(|f| crate::flow_sim::run::lines(&f.m.log).into_iter().map(|(_, t)| t).filter(|t| t.contains(KAKARIKO_EXP)).collect::<Vec<String>>())
                .collect();
            out.sort();
            out.dedup();
            out
        };
        let owned: Vec<(&str, &str)> = medals.iter().map(|m| (m.as_str(), "true")).collect();
        let lit: Vec<String> = run(&owned);
        let dark: Vec<String> = run(&SAGES);
        assert_eq!(lit.len(), 1);
        assert!(lit[0].contains("= true -> event 33"), "{lit:?}");
        assert_eq!(dark.len(), 1);
        assert!(dark[0].contains("= false -> event 11"), "{dark:?}");
        assert_eq!(dark, run(&[]));
        let two: Vec<(&str, &str)> = owned[..2].iter().copied().chain(SAGES).collect();
        assert_eq!(run(&two), dark);
    }

    #[test]
    fn the_prelude_module_reads_the_forest_medallion_formula() {
        let Some(rf) = real() else {
            return;
        };
        let raw: Vec<u8> = rf.read(PRELUDE_MODULE).unwrap();
        let mut a: Ainb = Ainb::parse(&raw).unwrap();
        let n: usize = a.els.len();
        patch_prelude(&mut a).unwrap();
        let reads: Vec<usize> = a.find_els(READ_EXP);
        assert_eq!(reads.iter().filter(|&&i| a.in_str(i, EXP_IN) == Some(PRELUDE_EXP)).count(), 1);
        assert!(reads.iter().all(|&i| a.in_str(i, EXP_IN) != Some(FOREST_EXP)));
        assert!(patch_prelude(&mut a).is_err());
        let out: Vec<u8> = a.write().unwrap();
        let b: Ainb = Ainb::parse(&out).unwrap();
        assert_eq!(b.els.len(), n);
        assert!(crate::ainb::audit(&b).is_empty());
        assert_eq!(b.write().unwrap(), out);
    }

    #[test]
    fn the_light_arrows_module_no_longer_waits_for_the_last_sage() {
        let Some(rf) = real() else {
            return;
        };
        let raw: Vec<u8> = rf.read(LACS_MODULE).unwrap();
        let mut a: Ainb = Ainb::parse(&raw).unwrap();
        let step: usize = lacs_step(&a).unwrap();
        assert!(reads_of(&a, step) > 0);
        assert_eq!(patch_lacs(&mut a).unwrap(), 1);
        assert_eq!(reads_of(&a, step), 0);
        let area: usize = a.find_els(AREA)[0];
        let x = a.input(BOOL, a.find_in(area, BOOL, CREATE).unwrap()).unwrap();
        assert_eq!((x.src, x.val), (-1, 1u32.to_le_bytes().to_vec()));
        let hit: usize = a.find_els(HIT)[0];
        let and: usize = a.find_els(AND)[0];
        let (s, c) = multi_range(&a, and).unwrap();
        assert!((s..s + c).all(|j| multi_entry(&a, j).map(|(e, _)| e as usize) == Some(hit)));
        let req: usize = a.find_els("ActorLogicEventRequestActor")[0];
        let t = a.input(BOOL, a.find_in(req, BOOL, "EventRequestTrigger").unwrap()).unwrap();
        assert_eq!(t.src as usize, and);
        assert!(patch_lacs(&mut a).is_err());
        let out: Vec<u8> = a.write().unwrap();
        let b: Ainb = Ainb::parse(&out).unwrap();
        assert!(crate::ainb::audit(&b).is_empty());
        assert_eq!(b.write().unwrap(), out);
        assert!(out.len() <= raw.len());
    }

    #[test]
    fn the_light_arrows_area_appears_only_with_the_two_medallions() {
        let Some(mut rf) = real() else {
            return;
        };
        let raw: Vec<u8> = rf.read_zs(SHEIK_FILE).unwrap();
        let out: Vec<u8> = banc_bytes(&raw).unwrap();
        let y: Byml = Byml::parse(&out).unwrap();
        let van: Byml = Byml::parse(&raw).unwrap();
        let acts = |b: &Byml| b.root.get("Actors").unwrap().as_arr().unwrap().to_vec();
        let (a, v): (Vec<Node>, Vec<Node>) = (acts(&y), acts(&van));
        assert_eq!(a.len(), v.len());
        let changed: Vec<&Node> = a.iter().zip(v.iter()).filter(|(x, z)| x != z).map(|(x, _)| x).collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].get("Hash"), Some(&Node::U64(LACS_AREA_HASH)));
        let p: &Node = changed[0].get("Presence").unwrap();
        assert_eq!(p.get("FlagName"), Some(&Node::Str(LACS_EXP.to_string())));
        assert_eq!(p.get("IsNegation"), Some(&Node::Bool(false)));
        assert!(banc_bytes(&out).is_err());
        let groups: &[Node] = van.root.get("AiGroups").unwrap().as_arr().unwrap();
        let g: &Node = groups.iter().find(|g| g.get("Logic").and_then(|l| l.as_str()).is_some_and(|l| l.contains("Market_Adult_Prefab_c668"))).unwrap();
        let refs: &[Node] = g.get("References").unwrap().as_arr().unwrap();
        assert!(refs.iter().any(|r| r.get("Id") == Some(&Node::Str(LACS_AREA.to_string())) && r.get("Reference") == Some(&Node::U64(LACS_AREA_HASH))));
    }

    #[test]
    fn the_stage_writes_two_modules_and_merges_the_map_edit_into_an_earlier_one() {
        let Some(mut rf) = real() else {
            return;
        };
        let mut more: Vec<Out> = crate::door_open::stage_pedestal(&mut rf, None).unwrap();
        let n: usize = more.len();
        stage(&mut rf, &mut more, None).unwrap();
        assert_eq!(more.len(), n + 2);
        assert_eq!(more.iter().filter(|o| o.path == SHEIK_FILE).count(), 1);
        let map: Byml = Byml::parse(&rf.zs.decompress(&more.iter().find(|o| o.path == SHEIK_FILE).unwrap().z).unwrap()).unwrap();
        let acts: &[Node] = map.root.get("Actors").unwrap().as_arr().unwrap();
        let sheik: &Node = acts.iter().find(|a| a.get("Hash") == Some(&Node::U64(crate::door_open::SHEIK_HASH))).unwrap();
        assert_ne!(sheik.get("Translate").unwrap().as_arr().unwrap()[2], Node::F32(-34.0));
        assert!(acts.iter().find(|a| a.get("Hash") == Some(&Node::U64(LACS_AREA_HASH))).unwrap().get("Presence").is_some());
        let mut alone: Vec<Out> = Vec::new();
        stage(&mut rf, &mut alone, None).unwrap();
        assert_eq!(alone.len(), 3);
    }

    fn rules(gates: bool) -> World {
        let d: Data = Data::load().unwrap();
        let over: Vec<(&str, &str)> = if gates { vec![(LOGIC_KEY, "true")] } else { Vec::new() };
        world_with(&d, &over).unwrap()
    }

    fn without(w: &World, gone: &[&str]) -> Placement {
        let d: Data = Data::load().unwrap();
        let mut p: Placement = crate::slice::fill::vanilla_place(w, &d).unwrap();
        let g: Vec<ItemId> = gone.iter().map(|n| w.item(n).unwrap()).collect();
        let f: ItemId = w.item("Rupees (5)").unwrap();
        for l in &w.real {
            if p.get(*l).is_some_and(|i| g.contains(&i)) {
                p.put(*l, f);
            }
        }
        p
    }

    fn reach(w: &World, p: &Placement, start: &[&str], loc: &str) -> bool {
        let mut extra: Vec<ItemId> = vec![w.item("Progressive Strength Upgrade").unwrap(); 3];
        extra.extend(start.iter().map(|n| w.item(n).unwrap()));
        let s: Search = explore(w, p, &extra);
        s.loc_access(w.loc(loc).unwrap())
    }

    const LACS: &str = "ToT Light Arrows Cutscene";
    const PRELUDE: &str = "Sheik at Temple";
    const NOCTURNE: &str = "Sheik in Kakariko";

    #[test]
    fn with_the_gates_the_logic_needs_the_medallion_items_and_no_boss() {
        let on: World = rules(true);
        let off: World = rules(false);
        for (w, gone, start, loc, want) in [
            (&on, &["Shadow Medallion"][..], &[][..], LACS, false),
            (&on, &["Spirit Medallion"][..], &[][..], LACS, false),
            (&on, &["Forest Medallion"][..], &[][..], PRELUDE, false),
            (&on, &["Forest Medallion"][..], &[][..], NOCTURNE, false),
            (&on, &["Water Medallion"][..], &[][..], NOCTURNE, false),
            (&on, &[][..], &[][..], LACS, true),
            (&on, &[][..], &[][..], PRELUDE, true),
            (&on, &[][..], &[][..], NOCTURNE, true),
            (&off, &["Shadow Medallion", "Spirit Medallion"][..], &[][..], LACS, true),
            (&off, &["Forest Medallion"][..], &[][..], PRELUDE, true),
        ] {
            assert_eq!(reach(w, &without(w, gone), start, loc), want, "{loc} without {gone:?}");
        }
        let no_bosses: [&str; 3] = ["Mirror Shield", "Dins Fire", "Lens of Truth"];
        assert!(!reach(&off, &without(&off, &no_bosses), &[], LACS));
        assert!(reach(&on, &without(&on, &no_bosses), &["Shadow Medallion", "Spirit Medallion"], LACS));
        assert!(!reach(&off, &without(&off, &["Forest Medallion", "Bow"]), &[], PRELUDE));
        assert!(reach(&on, &without(&on, &["Forest Medallion", "Bow"]), &["Forest Medallion"], PRELUDE));
    }

    #[test]
    fn only_stones_and_medallions_turn_the_gates_on() {
        let mut s: Settings = Settings::default();
        s.set("shuffle_rewards", Value::s("supported")).unwrap();
        let p = crate::slice::support::plan(&s).unwrap();
        assert!(!p.logic().contains(&(LOGIC_KEY, "true")));
        assert!(!p.defaults().contains(&(SETTING, "true")));
        s.set(SETTING, Value::Bool(true)).unwrap();
        let p = crate::slice::support::plan(&s).unwrap();
        assert!(p.logic().contains(&(LOGIC_KEY, "true")));
        assert!(p.defaults().contains(&(SETTING, "true")));
        assert_eq!(crate::start_defaults::presets::chosen(&p.defaults()).unwrap().iter().filter(|e| exps().contains(e)).count(), 3);
    }
}
