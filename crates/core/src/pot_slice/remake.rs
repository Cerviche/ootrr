use std::collections::HashSet as HS;
use std::path::Path as P;

use crate::catalog::{Catalog, Loc};
use crate::error::{Error as E, Res};
use crate::json::{self, Json};
use crate::logic_json::Json as Lj;
use crate::pot_slice::data::Kind;
use crate::pot_slice::rocks::{apply_with, Spot};
use crate::pot_slice::scan::{finish, judge_all, PotScan};
use crate::romfs::Romfs;
use crate::slice::data::Data;

pub const CATALOG_JSON: &str = include_str!("../../../../data/pots_remake.json");
pub const SCAN_JSON: &str = include_str!("../../../../data/pots_remake_scan.json");
pub const SETTING: &str = "shuffle_remake_pots";
pub const KEY: &str = "remake_pot_spots";
pub const ON: &str = "on";
pub const MARK: &str = "remake";

pub fn locs() -> Res<Vec<Loc>> {
    let c: Catalog = Catalog::parse(CATALOG_JSON)?;
    for l in &c.locs {
        if !matches!(Kind::of(&l.kind), Some(Kind::Pot | Kind::Crate)) {
            return Err(E::Msg(format!("remake pot catalog: {} has the kind {}, which is not pot or crate", l.name, l.kind)));
        }
        if l.refs.len() != 1 || l.refs.iter().any(|r| r.files.len() != 1) {
            return Err(E::Msg(format!("remake pot catalog: {} must own exactly one actor in one file", l.name)));
        }
        if !l.ev.iter().any(|e| e == MARK) {
            return Err(E::Msg(format!("remake pot catalog: {} is not marked {MARK}", l.name)));
        }
    }
    Ok(c.locs)
}

pub fn names() -> Res<HS<String>> {
    Ok(locs()?.into_iter().map(|l| l.name).collect())
}

fn text<'a>(j: &'a Json, k: &str, at: &str) -> Res<&'a str> {
    j.get(k).and_then(|v| v.as_str()).ok_or_else(|| E::Msg(format!("remake pot catalog: {at} has no {k}")))
}

pub fn spots_of(t: &str) -> Res<Vec<Spot>> {
    let j: Json = json::parse(t)?;
    let rows: &[Json] = j.get("locations").and_then(|v| v.as_arr()).ok_or(E::Format("remake pot catalog: no locations"))?;
    let mut out: Vec<Spot> = Vec::with_capacity(rows.len());
    for r in rows {
        let name: &str = text(r, "name", "a row")?;
        out.push(Spot { name: name.to_string(), region: text(r, "region", name)?.to_string(), rule: text(r, "rule", name)?.to_string() });
    }
    Ok(out)
}

pub fn spots() -> Res<Vec<Spot>> {
    spots_of(CATALOG_JSON)
}

pub fn wanted(over: &[(&str, &str)]) -> bool {
    over.iter().any(|(k, v)| *k == KEY && *v == ON)
}

pub fn apply(files: &mut [Lj]) -> Res<usize> {
    apply_with(files, &spots()?)
}

pub fn scan(root: &P) -> Res<PotScan> {
    let d: Data = Data::load()?;
    let ls: Vec<Loc> = locs()?;
    let mut rf: Romfs = Romfs::open(root)?;
    let rows: Vec<(String, crate::slice::data::State)> = judge_all(&d, &mut rf, &ls)?;
    finish(&rf, &ls, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap as BM;

    use crate::pot_slice::data::PotData;
    use crate::search::{explore, Search};
    use crate::settings::{Settings, Value};
    use crate::slice::data::State;
    use crate::slice::fill::vanilla_place;
    use crate::slice::generate::{allowed, generate_data, GenOpts, Generated};
    use crate::slice::logic::{world, world_with};
    use crate::slice::support::{plan, Mode, Plan};
    use crate::world::{LocId, Placement, World};

    fn with(pairs: &[(&str, &str)]) -> Settings {
        let mut s: Settings = Settings::default();
        for (k, v) in pairs {
            s.set(k, Value::s(v)).unwrap();
        }
        s
    }

    fn on_world(d: &Data) -> World {
        world_with(d, &[(KEY, ON)]).unwrap()
    }

    #[test]
    fn the_catalog_counts_are_pinned_and_no_hash_is_held_twice() {
        let d: Data = Data::load().unwrap();
        let ls: Vec<Loc> = locs().unwrap();
        assert_eq!(ls.len(), 19);
        assert!(ls.iter().all(|l| l.kind == "pot" && l.conf == crate::catalog::Conf::Forced));
        let mut by: BM<&str, usize> = BM::new();
        for l in &ls {
            *by.entry(l.area.as_str()).or_insert(0) += 1;
        }
        assert_eq!(by.get("Gerudo Valley"), Some(&4));
        assert_eq!(by.get("Fire Temple"), Some(&4));
        assert_eq!(by.get("Shadow Temple"), Some(&3));
        let mine: HS<u64> = ls.iter().map(|l| l.refs[0].hash).collect();
        assert_eq!(mine.len(), ls.len());
        for l in d.cat.locs.iter().chain(d.pots.locs.iter().filter(|l| !d.pots.is_remake(&l.name))) {
            for r in &l.refs {
                assert!(!mine.contains(&r.hash), "{} already holds hash {}", l.name, r.hash);
            }
        }
        for l in &ls {
            assert!(d.cat.by_name(&l.name).is_none(), "{}", l.name);
            assert!(crate::pot_slice::pickup::of(&l.vanilla).is_some_and(crate::pot_slice::pickup::refill), "{}", l.name);
        }
    }

    #[test]
    fn every_remake_pot_is_scanned_eligible() {
        let d: Data = Data::load().unwrap();
        let p: &PotData = &d.pots;
        for l in locs().unwrap() {
            assert!(p.is_remake(&l.name));
            assert_eq!(p.state_of(&l.name), Some(State::Eligible), "{}", l.name);
            assert_eq!(d.vanilla_of(&l.name), Some(l.vanilla.as_str()));
        }
        assert!(p.remake_game.starts_with("oot-remake:"));
        assert!(!p.remake_files.is_empty());
    }

    #[test]
    fn the_world_has_no_remake_pot_until_the_key_is_set() {
        let d: Data = Data::load().unwrap();
        let off: World = world(&d).unwrap();
        let on: World = on_world(&d);
        let sp: Vec<Spot> = spots().unwrap();
        assert!(sp.iter().all(|s| off.loc(&s.name).is_none()));
        assert_eq!(on.real.len(), off.real.len() + sp.len());
        for s in &sp {
            let l: LocId = on.loc(&s.name).unwrap_or_else(|| panic!("{}", s.name));
            assert_eq!(on.regions[on.locs[l].region].name, s.region, "{}", s.name);
        }
        assert!(!wanted(&plan(&Settings::default()).unwrap().logic()));
        assert!(wanted(&plan(&with(&[(SETTING, "all")])).unwrap().logic()));
        assert!(!wanted(&plan(&with(&[("shuffle_pots", "all")])).unwrap().logic()));
    }

    #[test]
    fn the_vanilla_world_reaches_every_remake_pot_and_the_age_rules_bind() {
        let d: Data = Data::load().unwrap();
        let w: World = on_world(&d);
        let v: Placement = vanilla_place(&w, &d).unwrap();
        let s: Search = explore(&w, &v, &[]);
        assert!(s.won());
        for sp in spots().unwrap() {
            assert!(s.loc_access(w.loc(&sp.name).unwrap()), "{}", sp.name);
        }
        let sp: Vec<Spot> = spots().unwrap();
        assert!(sp.iter().all(|s| !s.name.contains(" Child ")));
        assert!(sp.iter().filter(|s| s.name.contains(" Adult ")).all(|s| s.rule == "is_adult"));
    }

    #[test]
    fn remake_pots_take_refills_only() {
        let d: Data = Data::load().unwrap();
        let w: World = on_world(&d);
        let p: Plan = plan(&with(&[(SETTING, "all"), ("shuffle_pots", "all")])).unwrap();
        let ok = |item: &str, loc: &str| allowed(&d, &w, &p, w.item(item).unwrap(), w.loc(loc).unwrap());
        for loc in ["Fire Temple Near Boss Remake Pot 1", "GV Storms Grotto Adult Remake Pot 1"] {
            assert!(ok("Rupees (20)", loc), "{loc}");
            assert!(ok("Recovery Heart", loc), "{loc}");
            assert!(!ok("Progressive Hookshot", loc), "{loc}");
            assert!(!ok("Megaton Hammer", loc), "{loc}");
            assert!(!ok("Fire Arrows", loc), "{loc}");
            assert!(!ok("Gold Skulltula Token", loc), "{loc}");
            assert!(!ok("Small Key (Fire Temple)", loc), "{loc}");
        }
        assert!(ok("Megaton Hammer", "Dodongos Cavern Right Side Pot 1"));
    }

    #[test]
    fn modes_follow_the_new_setting_and_not_the_pot_setting() {
        let mut bad: Vec<String> = Vec::new();
        let d: crate::pot_slice::Drops = crate::pot_slice::plan::read(&with(&[(SETTING, "dungeons")]), &mut bad);
        assert!(bad.is_empty());
        assert_eq!((d.remake, d.pots), (Mode::Dungeons, Mode::Off));
        assert_eq!(crate::pot_slice::plan::read(&Settings::default(), &mut bad).remake, Mode::Off);
    }

    fn beatable(d: &Data, s: &Settings, g: &Generated) -> bool {
        let p: Plan = plan(s).unwrap();
        let w: World = world_with(d, &p.logic()).unwrap();
        let mut pl: Placement = vanilla_place(&w, d).unwrap();
        for (loc, item) in &g.seed_file.placements {
            pl.put(w.loc(loc).unwrap(), w.item(item).unwrap());
        }
        explore(&w, &pl, &[]).won()
    }

    #[test]
    fn remake_pot_seeds_are_beatable_and_keep_to_the_rules() {
        let d: Data = Data::load().unwrap();
        let mut moved: usize = 0;
        for (remake, pots) in [("all", "off"), ("dungeons", "all"), ("overworld", "off")] {
            let s: Settings = with(&[(SETTING, remake), ("shuffle_pots", pots)]);
            let p: Plan = plan(&s).unwrap();
            let w: World = world_with(&d, &p.logic()).unwrap();
            for n in 0..2 {
                let g: Generated = generate_data(&d, &s, &format!("remake pots {remake} {pots} {n}"), &GenOpts::default()).unwrap();
                assert!(beatable(&d, &s, &g), "{remake} {pots} {n}");
                let mut here: usize = 0;
                for (loc, item) in &g.seed_file.placements {
                    if !d.pots.is_remake(loc) {
                        continue;
                    }
                    here += 1;
                    assert!(!crate::pot_slice::pickup::holds(item), "{loc} holds {item}");
                    let dungeon: bool = w.regions[w.locs[w.loc(loc).unwrap()].region].dungeon.is_some();
                    assert!(remake == "all" || (remake == "dungeons") == dungeon, "{loc} under {remake}");
                    if d.vanilla_of(loc) != Some(item.as_str()) {
                        moved += 1;
                    }
                }
                let want: usize = match remake {
                    "all" => 19,
                    "dungeons" => 15,
                    _ => 4,
                };
                assert_eq!(here, want, "{remake} {pots} {n}");
            }
        }
        assert!(moved > 5, "{moved}");
    }

    #[test]
    fn the_default_settings_and_the_old_pot_settings_never_touch_a_remake_pot() {
        let d: Data = Data::load().unwrap();
        for s in [Settings::default(), with(&[("shuffle_pots", "all"), ("shuffle_crates", "all")])] {
            let g: Generated = generate_data(&d, &s, "no remake pots", &GenOpts::default()).unwrap();
            assert!(g.seed_file.placements.keys().all(|k| !d.pots.is_remake(k)));
        }
    }

    #[test]
    fn a_remake_pot_never_gets_a_pot_kit() {
        use crate::settings::seed::SeedFile;
        let d: Data = Data::load().unwrap();
        let one = |item: &str| {
            let m: BM<String, String> = [("Fire Temple Near Boss Remake Pot 1".to_string(), item.to_string())].into_iter().collect();
            crate::pot_slice::targets::targets(&d, &SeedFile::new("s", &Settings::default(), "g", m).unwrap())
        };
        let t = one("Rupees (50)").unwrap();
        assert_eq!(t.len(), 1);
        assert!(!t[0].kit && t[0].actor == "Obj_Rupee_Purple");
        assert!(one("Megaton Hammer").is_err());
    }

    fn real() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("C:/o/nup dump/updates/131072/romfs"));
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(p)
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    #[test]
    fn the_shipped_scan_matches_the_dump() {
        let Some(root) = real() else {
            return;
        };
        let s: PotScan = scan(&root).unwrap();
        let (game, files, rows): (String, Vec<String>, Vec<(String, State)>) = crate::pot_slice::data::parse_scan(SCAN_JSON).unwrap();
        assert_eq!((s.game, s.files, s.rows), (game, files, rows));
    }
}
