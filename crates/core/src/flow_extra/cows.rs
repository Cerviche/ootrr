use crate::error::{Error as E, Res};
use crate::flow_extra::data::{Extra, Spec};
use crate::flow_extra::op::How;
use crate::flow_locations::{Change, Loc};

pub const SETTING: &str = "shuffle_cows";
pub const ROLE: &str = "once";
pub const PARAM: &str = "ActorName";
pub const VANILLA: &str = "Milk";

pub const LOCS: [&str; 2] = ["KF Links House Cow", "GV Cow"];

pub const NOTE: &str = "Link's House cow in Kokiri Forest and the Gerudo Valley cow each give one extra shuffled item the first time their milk scene starts, with or without a bottle. The milk itself stays vanilla and stays repeatable. The other cows are not shuffled.";

pub fn is(name: &str) -> bool {
    LOCS.contains(&name)
}

fn spec(name: &str) -> Res<Spec> {
    Extra::load()?.specs.into_iter().find(|s| s.name == name).ok_or_else(|| E::Msg(format!("{name} has no once grant edit in data/flow_extra.json")))
}

pub fn flag_of(name: &str) -> Option<String> {
    let s: Spec = spec(name).ok()?;
    s.ops.iter().find_map(|o| match &o.how {
        How::Once(w) => Some(w.flag.clone()),
        _ => None,
    })
}

pub fn changes(l: &Loc, item: &str) -> Res<Vec<Change>> {
    let s: Spec = spec(&l.name)?;
    let mut out: Vec<Change> = Vec::new();
    for o in &s.ops {
        if matches!(o.how, How::Once(_)) {
            out.push(Change {
                location: l.name.clone(),
                file: o.file.clone(),
                index: o.index,
                event: o.event.clone(),
                param: PARAM.to_string(),
                role: ROLE.to_string(),
                old: l.item.clone().unwrap_or_default(),
                new: item.to_string(),
            });
        }
    }
    if out.is_empty() {
        return Err(E::Msg(format!("{} has no once grant edit in data/flow_extra.json", l.name)));
    }
    Ok(out)
}

pub fn plain(c: &Change) -> bool {
    c.role != ROLE
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet as BS, HashMap as HM};
    use std::path::{Path as P, PathBuf as PB};

    use super::*;
    use crate::bfevfl::Flow;
    use crate::flow_extra::sim::{broken, called, edited, granted, run, sim, Scene};
    use crate::flow_locations::Data as FlowData;
    use crate::flow_sim::run::{End, Fin, Sim};
    use crate::romfs::Romfs;
    use crate::settings::{Settings, Value};
    use crate::slice::data::Data;
    use crate::slice::generate::{generate_data, GenOpts, Generated};
    use crate::slice::support::plan as plan_of;

    const FLAGS: &str = include_str!("../../../../data/flags.json");
    const DUMP: &str = "C:/o/nup dump/updates/131072/romfs";
    const NEW: &str = "Item_Longshot";
    const BOTTLE: &str = "Dm_SY_GetInEvent:GetInEventInBottle";
    const CHILD: &str = "Sequence_IsChildhood";

    fn root() -> Option<PB> {
        let p: PB = std::env::var_os("OOTRR_ROMFS").map(PB::from).unwrap_or_else(|| PB::from(DUMP));
        if p.join("Event/EventFlow").is_dir() {
            Some(p)
        } else {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", p.display());
            None
        }
    }

    fn scene(entry: &str, set: &[(&str, &str)]) -> Scene {
        Scene { flow: "Animal_Cow".to_string(), entry: entry.to_string(), set: set.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), own: Vec::new() }
    }

    fn both(r: &P, loc: &str, sc: &Scene) -> (Vec<Fin>, Vec<Fin>) {
        let mut rf: Romfs = Romfs::open(r).unwrap();
        let fl: HM<String, Flow> = edited(&mut rf, loc, NEW).unwrap();
        assert_eq!(fl.len(), 1, "{loc}");
        let mut a: Sim = sim(r, HM::new()).unwrap();
        let mut b: Sim = sim(r, fl).unwrap();
        (run(&mut a, sc).unwrap(), run(&mut b, sc).unwrap())
    }

    fn shape(f: &Fin, flag: &str) -> String {
        let gr: Vec<String> = f.m.gr.iter().filter(|(k, _)| k.as_str() != NEW).map(|(k, v)| format!("{k}={v}")).collect();
        let wr: Vec<String> = f.m.writes.iter().filter(|(k, _)| k.as_str() != flag).map(|(k, v)| format!("{k}={v}")).collect();
        format!("{:?} | {gr:?} | {:?} | {wr:?}", f.end, f.m.reqs)
    }

    fn shapes(fins: &[Fin], flag: &str) -> BS<String> {
        fins.iter().map(|f| shape(f, flag)).collect()
    }

    fn gave(f: &Fin, flag: &str) -> bool {
        f.m.writes.get(flag).map(|s| s.as_str()) == Some("true")
    }

    fn check_once(r: &P, loc: &str, entry: &str, flag: &str, set: &[(&str, &str)]) {
        let (a, b): (Vec<Fin>, Vec<Fin>) = both(r, loc, &scene(entry, set));
        assert!(a.iter().all(|f| granted(f, NEW) == 0 && !f.m.writes.contains_key(flag)), "{loc} vanilla");
        assert!(a.iter().any(|f| called(f, BOTTLE)), "{loc} vanilla milk");
        assert_eq!(shapes(&a, flag), shapes(&b, flag), "{loc} the edited paths differ from vanilla beyond the one grant");
        assert!(b.iter().any(|f| granted(f, NEW) == 1), "{loc}");
        for f in &b {
            assert_eq!(gave(f, flag), granted(f, NEW) == 1, "{loc} the grant and the flag write go together");
            assert!(granted(f, NEW) <= 1, "{loc}");
        }
        assert_eq!(broken(&b), broken(&a), "{loc}");
        assert!(b.iter().filter(|f| f.end == End::Done).any(|f| called(f, BOTTLE)), "{loc} the milk path survives");
    }

    fn check_quiet(r: &P, loc: &str, entry: &str, flag: &str, set: &[(&str, &str)]) {
        let (a, b): (Vec<Fin>, Vec<Fin>) = both(r, loc, &scene(entry, set));
        assert!(b.iter().all(|f| granted(f, NEW) == 0 && !f.m.writes.contains_key(flag)), "{loc} {set:?}");
        assert_eq!(shapes(&a, flag), shapes(&b, flag), "{loc} {set:?}");
        assert_eq!(a.len(), b.len(), "{loc} {set:?}");
    }

    #[test]
    fn the_house_cow_hands_out_the_item_once_and_the_milk_path_runs_as_vanilla() {
        let Some(r) = root() else { return };
        let flag: String = flag_of("KF Links House Cow").unwrap();
        check_once(&r, "KF Links House Cow", "PushToBottleMilk_LinkHouse", &flag, &[]);
        check_quiet(&r, "KF Links House Cow", "PushToBottleMilk_LinkHouse", &flag, &[(flag.as_str(), "true")]);
    }

    #[test]
    fn the_valley_cow_asks_for_a_child_and_the_fortress_adult_keeps_vanilla() {
        let Some(r) = root() else { return };
        let flag: String = flag_of("GV Cow").unwrap();
        check_once(&r, "GV Cow", "PushToBottleMilk_LinkHouse0", &flag, &[(CHILD, "true")]);
        check_quiet(&r, "GV Cow", "PushToBottleMilk_LinkHouse0", &flag, &[(CHILD, "true"), (flag.as_str(), "true")]);
        check_quiet(&r, "GV Cow", "PushToBottleMilk_LinkHouse0", &flag, &[(CHILD, "false")]);
    }

    fn with(on: bool) -> Settings {
        let mut s: Settings = Settings::default();
        s.set("shuffle_rewards", Value::s("supported")).unwrap();
        s.set(SETTING, Value::Bool(on)).unwrap();
        s
    }

    #[test]
    fn the_toggle_is_off_by_default_needs_supported_rewards_and_keeps_the_string() {
        assert!(!plan_of(&Settings::default()).unwrap().cows);
        assert!(!plan_of(&with(false)).unwrap().cows);
        assert!(plan_of(&with(true)).unwrap().cows);
        let mut s: Settings = Settings::default();
        s.set("shuffle_rewards", Value::s("off")).unwrap();
        s.set(SETTING, Value::Bool(true)).unwrap();
        assert!(!plan_of(&s).unwrap().cows);
        let mut d: Settings = Settings::default();
        d.set(SETTING, Value::Bool(false)).unwrap();
        assert_eq!(Settings::default().encode().unwrap(), d.encode().unwrap());
    }

    #[test]
    fn the_cows_are_flow_ok_but_closed_until_the_toggle_is_on() {
        let d: Data = Data::load().unwrap();
        let off = plan_of(&Settings::default()).unwrap();
        let on = plan_of(&with(true)).unwrap();
        for n in LOCS {
            let l: &Loc = d.flow.by_name(n).unwrap();
            assert!(d.flow_ok(n), "{n}");
            assert!(!crate::flow_extra::open(l, &off) && crate::flow_extra::open(l, &on), "{n}");
            assert_eq!(d.vanilla_of(n), Some("Rupees (5)"), "{n}");
            assert!(crate::slice::edit::is_sale(&d, n), "{n}");
        }
        assert!(!crate::flow_extra::bound(&d, "Rupees (5)") && !crate::flow_extra::bound(&d, "Milk"));
        assert!(!d.flow_ok("HF Cow Grotto Cow"));
    }

    #[test]
    fn toggle_off_places_neither_cow_and_toggle_on_adds_both_with_items_the_flows_hand_out() {
        let d: Data = Data::load().unwrap();
        let off: Generated = generate_data(&d, &with(false), "cows", &GenOpts::default()).unwrap();
        for l in LOCS {
            assert!(!off.seed_file.placements.contains_key(l), "{l}");
        }
        for n in 0..3 {
            let g: Generated = generate_data(&d, &with(true), &format!("cows {n}"), &GenOpts::default()).unwrap();
            assert_eq!(g.seed_file.placements.len(), off.seed_file.placements.len() + LOCS.len());
            for l in LOCS {
                let it: &str = &g.seed_file.placements[l];
                assert!(d.item(it).is_some_and(|i| d.flow_accepts(i) || it == "Rupees (5)"), "{l} holds {it}");
                assert_ne!(it, "Milk", "{l}");
            }
            assert!(g.seed_file.placements.values().all(|i| i != "Milk"));
        }
    }

    #[test]
    fn a_built_mod_carries_the_once_gate_for_both_cows_and_leaves_the_others_alone() {
        let Some(r) = root() else { return };
        let d: Data = Data::load().unwrap();
        let o: GenOpts = GenOpts { plando: vec![crate::slice::plando::parse("KF Links House Cow=Hover Boots").unwrap()], ..GenOpts::default() };
        let g: Generated = generate_data(&d, &with(true), "cows build", &o).unwrap();
        assert_eq!(g.seed_file.placements.get("KF Links House Cow").map(|s| s.as_str()), Some("Hover Boots"));
        let load: PB = PB::from(env!("CARGO_MANIFEST_DIR")).join("../../target/cows_mods");
        let _ = std::fs::remove_dir_all(&load);
        let b: crate::slice::edit::Built = crate::slice::edit::build_for(&g.seed_file, &r, &load, "ootrr-cows", crate::modpack::Host::Eden).unwrap();
        let mut rf: Romfs = Romfs::open(&r).unwrap();
        let z: Vec<u8> = std::fs::read(b.root.join("romfs").join(crate::flow_locations::path_of("Animal_Cow.bfevfl"))).unwrap();
        let out: crate::bfevfl::Bf = crate::bfevfl::Bf::parse(&rf.zs.decompress(&z).unwrap()).unwrap();
        let van: crate::bfevfl::Bf = crate::bfevfl::Bf::parse(&rf.read_zs(&crate::flow_locations::path_of("Animal_Cow.bfevfl")).unwrap()).unwrap();
        let (f, v): (&Flow, &Flow) = (&out.flows[0], &van.flows[0]);
        let main = |x: &Flow, n: &str| x.entries.iter().find(|e| e.name == n).map(|e| e.main).unwrap();
        assert_ne!(main(f, "PushToBottleMilk_LinkHouse"), main(v, "PushToBottleMilk_LinkHouse"));
        assert_ne!(main(f, "PushToBottleMilk_LinkHouse0"), main(v, "PushToBottleMilk_LinkHouse0"));
        assert_eq!(main(f, "PushToBottleMilk"), main(v, "PushToBottleMilk"));
        assert_eq!(f.events.len(), v.events.len() + 7);
        let gave: bool = f.events.iter().any(|e| matches!(&e.kind, crate::bfevfl::Kind::Sub { entry, params, .. } if entry == "GetInEvent_NotTurn_NotNpc" && params.as_ref().and_then(|c| c.get("ActorName")).is_some_and(|x| *x != crate::bfevfl::Val::Str("Item_Bottle_Milk".to_string()))));
        assert!(gave);
        let _ = std::fs::remove_dir_all(&load);
    }

    #[test]
    fn the_two_cows_edit_one_file_together_and_add_seven_events() {
        let Some(r) = root() else { return };
        let mut rf: Romfs = Romfs::open(&r).unwrap();
        let d: FlowData = FlowData::load().unwrap();
        let ch: Vec<Change> = LOCS.iter().flat_map(|l| changes(d.by_name(l).unwrap(), NEW).unwrap()).collect();
        let os = crate::flow_extra::ops_for(&ch).unwrap();
        assert_eq!(os.len(), 1);
        assert_eq!(os["Animal_Cow.bfevfl"].len(), 2);
        let raw: Vec<u8> = rf.read_zs(&crate::flow_locations::path_of("Animal_Cow.bfevfl")).unwrap();
        let b: crate::bfevfl::Bf = crate::flow_extra::edit_flow(&raw, &[], &os["Animal_Cow.bfevfl"]).unwrap();
        assert_eq!(crate::bfevfl::Bf::parse(&b.write().unwrap()).unwrap(), b);
        let vanilla: crate::bfevfl::Bf = crate::bfevfl::Bf::parse(&raw).unwrap();
        assert_eq!(b.flows[0].events.len(), vanilla.flows[0].events.len() + 7);
    }

    #[test]
    fn both_cows_are_gated_flow_locations_with_one_once_edit_each() {
        let d: FlowData = FlowData::load().unwrap();
        for n in LOCS {
            let l: &Loc = d.by_name(n).unwrap();
            assert!(l.gated && !l.supported, "{n}");
            assert_eq!(l.cat, "cow", "{n}");
            assert_eq!(l.item.as_deref(), Some("Item_Bottle_Milk"), "{n}");
            let ch: Vec<Change> = changes(l, "Item_Longshot").unwrap();
            assert_eq!(ch.len(), 1, "{n}");
            assert!(ch.iter().all(|c| c.role == ROLE && c.new == "Item_Longshot" && c.old == "Item_Bottle_Milk" && !plain(c)), "{n}");
            assert_eq!(ch[0].file, "Animal_Cow.bfevfl");
        }
        assert!(changes(d.by_name("LW Trade Cojiro").unwrap(), "Item_Longshot").is_err());
    }

    #[test]
    fn the_once_flags_are_distinct_unused_persistent_bools() {
        let a: String = flag_of("KF Links House Cow").unwrap();
        let b: String = flag_of("GV Cow").unwrap();
        assert_ne!(a, b);
        for f in [&a, &b] {
            let row: &str = FLAGS.lines().find(|l| l.contains(&format!("\"name\":\"{f}\""))).unwrap();
            assert!(row.contains("\"type\":\"Bool\"") && row.contains("\"save\":0") && row.contains("\"reset\":8"), "{f}");
            assert!(f.starts_with("HavePlayedEvent."), "{f}");
        }
        assert!(flag_of("LW Trade Cojiro").is_none());
    }

    #[test]
    fn the_gerudo_valley_edit_asks_for_a_child_and_the_house_edit_does_not() {
        for (n, child, entry) in [("KF Links House Cow", false, "PushToBottleMilk_LinkHouse"), ("GV Cow", true, "PushToBottleMilk_LinkHouse0")] {
            let s: Spec = spec(n).unwrap();
            assert_eq!(s.ops.len(), 1, "{n}");
            let How::Once(w) = &s.ops[0].how else { panic!("once") };
            assert_eq!((w.child, w.entry.as_str()), (child, entry), "{n}");
            assert_eq!(w.sub, "Dm_SY_GetInEvent:GetInEvent_NotTurn_NotNpc");
        }
    }
}
