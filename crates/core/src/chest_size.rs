use std::collections::{BTreeMap as BM, BTreeSet as BS};

use crate::byml::{Byml, Node};
use crate::error::Res;
use crate::placement::{banc_files, hash_of};
use crate::romfs::Romfs;
use crate::settings::seed::SeedFile;
use crate::settings::Settings;
use crate::slice::data::{Data, Ev, Item};

pub const SETTING: &str = "chests_match_contents";

/// The plain wooden chests, small model first. Other chests (the shield chest, invisible
/// chests, the minigame chests, A_03 and A_04) keep their model.
pub const PAIRS: [(&str, &str); 2] = [("Obj_TBox_A_01", "Obj_TBox_A_02"), ("Obj_TBox_A_01_NoBake", "Obj_TBox_A_02_NoBake")];

pub const NOTE: &str = "Chest size matches contents: chests holding major items and boss keys are big, chests holding anything else are small. Chests that kept their vanilla item, run a story scene or are not plain wooden chests keep their size. Experimental, untested in the game.";

pub const AREAS: [&str; 2] = ["DropKeepAwayArea_TBox_A_01", "DropKeepAwayArea_TBox_A_02"];

pub const ROOM: f32 = 1.5;

pub const TILT: f32 = 0.06;

pub const MARKS: [&str; 46] = [
    "anchor", "area", "tag", "startpos", "mapicon", "locator", "eventrequest", "eventproxy", "logic", "spawner", "navi", "sensor", "trigger", "point", "camera", "sound",
    "bgm", "se_", "light", "effect", "emitter", "fog", "env", "spot", "guide", "hint", "link", "rail", "path", "region", "water", "kill", "respawn", "check", "wind",
    "dropkeepaway", "mgr", "manager", "director", "controller", "counter", "timer", "flag", "switch_hidden", "dummy", "debug",
];

pub const WALLS: [&str; 6] = ["land_", "dgn", "field", "fldobj_wall", "fldobj_airwall", "pushairwall"];

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub hash: u64,
    pub gyaml: String,
    pub at: [f32; 3],
    pub rot: [f32; 3],
    pub story: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Moves {
    pub chests: BS<u64>,
    pub areas: BM<(String, [u32; 3]), &'static str>,
    pub tight: BS<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Want {
    pub big: bool,
    pub strip: bool,
}

pub fn mode(s: &Settings, bad: &mut Vec<String>) -> bool {
    s.flag(SETTING).unwrap_or_else(|e| {
        bad.push(e.to_string());
        false
    })
}

/// OoTR's major items: progression items and boss keys. Hearts, Bombchus (unless they are
/// in logic), maps, compasses, small keys, tokens and everything that is not progression
/// count as minor, as in OoTR's Chest Appearance Matches Contents.
pub fn major(i: &Item, chus: bool) -> bool {
    match i.kind.as_str() {
        "BossKey" | "GanonBossKey" | "Song" | "DungeonReward" => true,
        "Item" => i.adv && i.name != "Heart Container" && !i.name.starts_with("Piece of Heart") && (chus || !i.name.starts_with("Bombchus")),
        _ => false,
    }
}

/// The size each placed chest should have, by actor hash. Chests whose story event stays,
/// and the special chests that always run the big chest flow, are left out.
pub fn wanted(d: &Data, seed: &SeedFile, chus: bool, maps: bool) -> BM<u64, Want> {
    let mut out: BM<u64, Want> = BM::new();
    for (loc, item) in &seed.placements {
        let (Some(l), Some(it)) = (d.cat.by_name(loc), d.item(item)) else {
            continue;
        };
        if l.kind == "bossheart" {
            continue;
        }
        let big: bool = major(it, chus);
        if !big && !shrinks(it, maps) {
            continue;
        }
        let moved: bool = d.vanilla_of(loc) != Some(item.as_str());
        for r in &l.refs {
            if crate::special::of(r.hash).is_some() {
                continue;
            }
            let ev: Option<Ev> = d.event_mode(r.hash);
            if ev == Some(Ev::Keep) {
                continue;
            }
            out.insert(r.hash, Want { big, strip: moved && ev == Some(Ev::Strip) });
        }
    }
    out
}

pub fn shrinks(i: &Item, maps: bool) -> bool {
    maps || !matches!(i.kind.as_str(), "Map" | "Compass")
}

pub fn area(big: bool) -> &'static str {
    AREAS[usize::from(big)]
}

pub fn stem(file: &str) -> String {
    for s in ["_Static.bcett.byml.zs", "_Dynamic.bcett.byml.zs"] {
        if let Some(x) = file.strip_suffix(s) {
            return x.to_string();
        }
    }
    file.to_string()
}

fn triple(n: Option<&Node>) -> Option<[f32; 3]> {
    match n?.as_arr()? {
        [Node::F32(x), Node::F32(y), Node::F32(z)] => Some([*x, *y, *z]),
        _ => None,
    }
}

fn bits(p: &[f32; 3]) -> [u32; 3] {
    [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]
}

pub fn seen(y: &Byml) -> Vec<Seen> {
    let Some(list) = y.root.get("Actors").and_then(|a| a.as_arr()) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|a| {
            Some(Seen {
                hash: hash_of(a).unwrap_or(0),
                gyaml: a.get("Gyaml").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                at: triple(a.get("Translate"))?,
                rot: triple(a.get("Rotate")).unwrap_or([0.0; 3]),
                story: a.get("Dynamic").and_then(|d| d.get("GetEventName")).is_some(),
            })
        })
        .collect()
}

pub fn tilted(rot: &[f32; 3]) -> bool {
    [rot[0], rot[2]].iter().any(|x| {
        let m: f32 = x.abs() % std::f32::consts::PI;
        m >= TILT && std::f32::consts::PI - m >= TILT
    })
}

pub fn blocks(chest: &Seen, o: &Seen) -> bool {
    if o.hash == chest.hash {
        return false;
    }
    let d: f32 = chest.at.iter().zip(o.at.iter()).map(|(a, b)| (a - b) * (a - b)).sum::<f32>().sqrt();
    if d >= ROOM {
        return false;
    }
    let g: String = o.gyaml.to_ascii_lowercase();
    if g.starts_with("obj_tbox") {
        return !g.contains("pedestal") && d > 0.05;
    }
    !(MARKS.iter().any(|m| g.contains(m)) || g.ends_with("pos") || WALLS.iter().any(|w| g.starts_with(w)))
}

pub fn room(chest: &Seen, near: &[Seen]) -> bool {
    !tilted(&chest.rot) && !near.iter().any(|o| blocks(chest, o))
}

pub fn moves(scenes: &BM<String, Vec<Seen>>, w: &BM<u64, Want>) -> Moves {
    let mut m: Moves = Moves::default();
    let mut spots: Vec<(u64, String, [u32; 3], bool)> = Vec::new();
    for (st, list) in scenes {
        for s in list {
            let Some(want) = w.get(&s.hash) else {
                continue;
            };
            let Some(t) = target(&s.gyaml, want.big) else {
                continue;
            };
            if s.gyaml == t || (s.story && !want.strip) {
                continue;
            }
            if want.big && !room(s, list) {
                m.tight.insert(s.hash);
                continue;
            }
            spots.push((s.hash, st.clone(), bits(&s.at), want.big));
        }
    }
    for (h, st, at, big) in spots {
        if m.tight.contains(&h) {
            continue;
        }
        m.chests.insert(h);
        m.areas.insert((st, at), area(big));
    }
    m
}

pub fn scan(rf: &mut Romfs, w: &BM<u64, Want>) -> Res<BM<String, Vec<Seen>>> {
    let mut all: BM<String, Vec<Seen>> = BM::new();
    let mut keep: BS<String> = BS::new();
    for f in banc_files(rf)? {
        let y: Byml = Byml::parse(&rf.read_zs(&f)?)?;
        let st: String = stem(&f);
        let v: Vec<Seen> = seen(&y);
        if v.iter().any(|s| w.contains_key(&s.hash)) {
            keep.insert(st.clone());
        }
        all.entry(st).or_default().extend(v);
    }
    all.retain(|k, _| keep.contains(k));
    Ok(all)
}

pub fn target(gyaml: &str, big: bool) -> Option<&'static str> {
    PAIRS.iter().find(|(s, b)| gyaml == *s || gyaml == *b).map(|(s, b)| if big { *b } else { *s })
}

pub fn apply(y: &mut Byml, st: &str, w: &BM<u64, Want>, m: &Moves) -> (usize, usize) {
    let Some(list) = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()) else {
        return (0, 0);
    };
    let mut n: usize = 0;
    let mut k: usize = 0;
    for a in list.iter_mut() {
        let g: &str = a.get("Gyaml").and_then(|v| v.as_str()).unwrap_or("");
        if AREAS.contains(&g) {
            let Some(t) = triple(a.get("Translate")).and_then(|p| m.areas.get(&(st.to_string(), bits(&p)))) else {
                continue;
            };
            if g != *t {
                a.set("Gyaml", Node::Str(t.to_string()));
                k += 1;
            }
            continue;
        }
        let Some(want) = hash_of(a).filter(|h| m.chests.contains(h)).and_then(|h| w.get(&h)) else {
            continue;
        };
        let Some(t) = target(g, want.big) else {
            continue;
        };
        if g == t {
            continue;
        }
        let story: bool = a.get("Dynamic").and_then(|d| d.get("GetEventName")).is_some();
        if story && !want.strip {
            continue;
        }
        a.set("Gyaml", Node::Str(t.to_string()));
        n += 1;
    }
    (n, k)
}

pub fn resize(raw: &[u8], file: &str, w: &BM<u64, Want>, m: &Moves) -> Res<Option<(Vec<u8>, usize)>> {
    if w.is_empty() || m.chests.is_empty() {
        return Ok(None);
    }
    let mut y: Byml = Byml::parse(raw)?;
    let (n, k): (usize, usize) = apply(&mut y, &stem(file), w, m);
    if n + k == 0 {
        return Ok(None);
    }
    Ok(Some((y.write()?, n)))
}

pub fn report(n: usize, tight: usize) -> String {
    format!("{n} chests changed size to match their contents. {tight} chests stayed small because a big chest would not fit at their spot.")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, kind: &str, adv: bool) -> Item {
        Item {
            name: name.to_string(),
            kind: kind.to_string(),
            adv,
            actor: None,
            conf: String::new(),
            dungeon: None,
            tiers: Vec::new(),
            world: Vec::new(),
        }
    }

    #[test]
    fn majors_follow_ootr() {
        assert!(major(&item("Progressive Hookshot", "Item", true), false));
        assert!(major(&item("Boss Key (Forest Temple)", "BossKey", false), false));
        assert!(major(&item("Zeldas Lullaby", "Song", true), false));
        assert!(!major(&item("Piece of Heart", "Item", true), false));
        assert!(!major(&item("Heart Container", "Item", true), false));
        assert!(!major(&item("Bombchus (10)", "Item", true), false));
        assert!(major(&item("Bombchus (10)", "Item", true), true));
        assert!(!major(&item("Rupees (20)", "Item", false), false));
        assert!(!major(&item("Small Key (Forest Temple)", "SmallKey", true), false));
        assert!(!major(&item("Map (Forest Temple)", "Map", false), false));
    }

    fn chest(hash: u64, gyaml: &str, story: bool) -> Node {
        let mut dy: Vec<(String, Node)> = vec![("Drop__DropActor".to_string(), Node::Str("X".to_string()))];
        if story {
            dy.push(("GetEventName".to_string(), Node::Str("Ev".to_string())));
        }
        Node::Map(vec![
            ("Dynamic".to_string(), Node::Map(dy)),
            ("Gyaml".to_string(), Node::Str(gyaml.to_string())),
            ("Hash".to_string(), Node::U64(hash)),
        ])
    }

    fn gyamls(y: &Byml) -> Vec<String> {
        y.root.get("Actors").unwrap().as_arr().unwrap().iter().map(|a| a.get("Gyaml").unwrap().as_str().unwrap().to_string()).collect()
    }

    #[test]
    fn plain_chests_take_the_wanted_size_and_others_stay() {
        let mut y: Byml = Byml {
            root: Node::Map(vec![(
                "Actors".to_string(),
                Node::Arr(vec![
                    chest(1, "Obj_TBox_A_01", false),
                    chest(2, "Obj_TBox_A_02_NoBake", false),
                    chest(3, "Obj_TBox_A_06", false),
                    chest(4, "Obj_TBox_A_01", true),
                    chest(5, "Obj_TBox_A_01", true),
                    chest(6, "Obj_TBox_A_02", false),
                    chest(7, "Obj_TBox_A_01", false),
                ]),
            )]),
            version: 7,
            big: false,
        };
        let big = Want { big: true, strip: false };
        let w: BM<u64, Want> = BM::from([
            (1, big),
            (2, Want { big: false, strip: false }),
            (3, big),
            (4, big),
            (5, Want { big: true, strip: true }),
            (6, big),
        ]);
        let all: Moves = Moves { chests: w.keys().copied().collect(), ..Moves::default() };
        assert_eq!(apply(&mut y, "s", &w, &all), (3, 0));
        assert_eq!(
            gyamls(&y),
            vec!["Obj_TBox_A_02", "Obj_TBox_A_01_NoBake", "Obj_TBox_A_06", "Obj_TBox_A_01", "Obj_TBox_A_02", "Obj_TBox_A_02", "Obj_TBox_A_01"]
        );
        let raw: Vec<u8> = y.write().unwrap();
        assert!(resize(&raw, "s", &w, &all).unwrap().is_none());
        assert!(resize(&raw, "s", &BM::new(), &all).unwrap().is_none());
        let one: BM<u64, Want> = BM::from([(1, Want { big: false, strip: false })]);
        let (back, n): (Vec<u8>, usize) = resize(&raw, "s", &one, &Moves { chests: BS::from([1]), ..Moves::default() }).unwrap().unwrap();
        assert_eq!(n, 1);
        assert_eq!(gyamls(&Byml::parse(&back).unwrap())[0], "Obj_TBox_A_01");
        assert!(resize(&raw, "s", &one, &Moves::default()).unwrap().is_none());
    }

    #[test]
    fn only_the_plain_pairs_change_and_each_size_has_its_area() {
        assert_eq!(target("Obj_TBox_A_01", true), Some("Obj_TBox_A_02"));
        assert_eq!(target("Obj_TBox_A_02_NoBake", false), Some("Obj_TBox_A_01_NoBake"));
        assert_eq!(target("Obj_TBox_A_01_NoBake", true), Some("Obj_TBox_A_02_NoBake"));
        for g in ["Obj_TBox_A_03", "Obj_TBox_A_03_NoBake", "Obj_TBox_A_04", "Obj_TBox_A_06", "Obj_TBox_A_06_NoBake", "Obj_TBox_GlassTruth_A_01", "Obj_TBox_A_01_memo11", "Obj_HrlCity_Shop_TBox_A_02"] {
            assert_eq!(target(g, true), None, "{g}");
            assert_eq!(target(g, false), None, "{g}");
        }
        for (s, b) in PAIRS {
            assert_eq!(s.len(), b.len());
        }
        assert_eq!((area(false), area(true)), ("DropKeepAwayArea_TBox_A_01", "DropKeepAwayArea_TBox_A_02"));
        assert_eq!(AREAS[0].len(), AREAS[1].len());
    }

    #[test]
    fn maps_and_compasses_only_shrink_when_the_build_credits_them() {
        let m: Item = item("Map (Deku Tree)", "Map", false);
        let c: Item = item("Compass (Deku Tree)", "Compass", false);
        let r: Item = item("Rupees (20)", "Item", false);
        assert!(!shrinks(&m, false) && !shrinks(&c, false));
        assert!(shrinks(&m, true) && shrinks(&c, true));
        assert!(shrinks(&r, false));
    }

    #[test]
    fn scene_stems_pair_the_static_and_dynamic_files() {
        assert_eq!(stem("Banc/GerudoValley/GerudoValley_Adult_Static.bcett.byml.zs"), "Banc/GerudoValley/GerudoValley_Adult");
        assert_eq!(stem("Banc/GerudoValley/GerudoValley_Adult_Dynamic.bcett.byml.zs"), "Banc/GerudoValley/GerudoValley_Adult");
        assert_ne!(stem("Banc/GerudoValley/GerudoValley_Child_Dynamic.bcett.byml.zs"), "Banc/GerudoValley/GerudoValley_Adult");
        assert_eq!(stem("Banc/X/Other.byml"), "Banc/X/Other.byml");
    }

    fn at(hash: u64, gyaml: &str, x: f32) -> Seen {
        Seen { hash, gyaml: gyaml.to_string(), at: [x, 10.0, 0.0], rot: [0.0; 3], story: false }
    }

    #[test]
    fn the_clearance_rule_skips_crowded_and_tilted_spots() {
        let c: Seen = at(1, "Obj_TBox_A_01", 0.0);
        assert!(room(&c, &[c.clone()]));
        for (g, x) in [("Obj_Pot_A_01", 1.0), ("Enemy_Beamos", 0.0), ("Obj_TBox_A_01", 1.2), ("Obj_FireWall_A_01", 0.3)] {
            assert!(blocks(&c, &at(9, g, x)), "{g}");
            assert!(!room(&c, &[at(9, g, x)]), "{g}");
        }
        for (g, x) in [
            ("Obj_Pot_A_01", 1.6),
            ("Land_FireTemple_02_Wall_A_03", 0.4),
            ("DropKeepAwayArea_TBox_A_01", 0.0),
            ("DestinationAnchor", 0.0),
            ("Obj_TBoxPedestal_A_01", 0.0),
            ("PushAirWall_Box", 0.2),
            ("StartPos", 0.5),
            ("Obj_TBox_A_02", 0.0),
        ] {
            assert!(!blocks(&c, &at(9, g, x)), "{g}");
        }
        let mut t: Seen = c.clone();
        t.rot = [0.3, 0.0, 0.0];
        assert!(tilted(&t.rot) && !room(&t, &[]));
        t.rot = [std::f32::consts::PI, 1.2, -std::f32::consts::PI];
        assert!(!tilted(&t.rot) && room(&t, &[]));
        t.rot = [0.0, 2.0, 0.2];
        assert!(tilted(&t.rot));
    }

    #[test]
    fn moves_skip_tight_grows_keep_shrinks_and_pair_each_chest_with_its_area() {
        let grow = Want { big: true, strip: false };
        let shrink = Want { big: false, strip: false };
        let pot: Seen = at(50, "Obj_Pot_A_01", 20.8);
        let mut story: Seen = at(4, "Obj_TBox_A_01", 40.0);
        story.story = true;
        let scenes: BM<String, Vec<Seen>> = BM::from([
            ("a".to_string(), vec![at(1, "Obj_TBox_A_01", 0.0), at(2, "Obj_TBox_A_01", 20.0), pot.clone(), at(3, "Obj_TBox_A_02", 21.5), story, at(5, "Obj_TBox_A_01", 60.0)]),
            ("b".to_string(), vec![at(5, "Obj_TBox_A_01", 60.0), at(51, "Enemy_Beamos", 60.5), at(6, "Obj_TBox_A_06", 80.0)]),
        ]);
        let w: BM<u64, Want> = BM::from([(1, grow), (2, grow), (3, shrink), (4, grow), (5, grow), (6, grow)]);
        let m: Moves = moves(&scenes, &w);
        assert_eq!(m.chests, BS::from([1, 3]));
        assert_eq!(m.tight, BS::from([2, 5]));
        assert_eq!(m.areas.len(), 2);
        assert_eq!(m.areas.get(&("a".to_string(), bits(&[0.0, 10.0, 0.0]))), Some(&"DropKeepAwayArea_TBox_A_02"));
        assert_eq!(m.areas.get(&("a".to_string(), bits(&[21.5, 10.0, 0.0]))), Some(&"DropKeepAwayArea_TBox_A_01"));
        let strip: BM<u64, Want> = BM::from([(4, Want { big: true, strip: true })]);
        assert_eq!(moves(&scenes, &strip).chests, BS::from([4]));
    }

    fn placed(gyaml: &str, x: f32) -> Node {
        Node::Map(vec![
            ("Gyaml".to_string(), Node::Str(gyaml.to_string())),
            ("Hash".to_string(), Node::U64(1000 + x as u64)),
            ("Translate".to_string(), Node::Arr(vec![Node::F32(x), Node::F32(10.0), Node::F32(0.0)])),
        ])
    }

    fn file(nodes: Vec<Node>) -> Vec<u8> {
        Byml { root: Node::Map(vec![("Actors".to_string(), Node::Arr(nodes))]), version: 7, big: false }.write().unwrap()
    }

    #[test]
    fn the_area_follows_its_chest_into_the_other_file_of_the_scene() {
        let mut c: Node = chest(1, "Obj_TBox_A_01", false);
        c.set("Translate", Node::Arr(vec![Node::F32(0.0), Node::F32(10.0), Node::F32(0.0)]));
        let fs: &str = "Banc/S/S_Adult_Static.bcett.byml.zs";
        let fd: &str = "Banc/S/S_Adult_Dynamic.bcett.byml.zs";
        let fo: &str = "Banc/S/S_Child_Dynamic.bcett.byml.zs";
        let st: Vec<u8> = file(vec![c]);
        let dy: Vec<u8> = file(vec![placed("DropKeepAwayArea_TBox_A_01", 0.0), placed("DropKeepAwayArea_TBox_A_01", 5.0), placed("DropKeepAwayArea", 0.0)]);
        let scenes: BM<String, Vec<Seen>> = BM::from([(stem(fs), seen(&Byml::parse(&st).unwrap()).into_iter().chain(seen(&Byml::parse(&dy).unwrap())).collect())]);
        let w: BM<u64, Want> = BM::from([(1, Want { big: true, strip: false })]);
        let m: Moves = moves(&scenes, &w);
        assert_eq!(m.chests, BS::from([1]));
        let (b, n): (Vec<u8>, usize) = resize(&st, fs, &w, &m).unwrap().unwrap();
        assert_eq!((n, gyamls(&Byml::parse(&b).unwrap())), (1, vec!["Obj_TBox_A_02".to_string()]));
        let (b, n): (Vec<u8>, usize) = resize(&dy, fd, &w, &m).unwrap().unwrap();
        assert_eq!(n, 0);
        assert_eq!(gyamls(&Byml::parse(&b).unwrap()), vec!["DropKeepAwayArea_TBox_A_02", "DropKeepAwayArea_TBox_A_01", "DropKeepAwayArea"]);
        assert!(resize(&dy, fo, &w, &m).unwrap().is_none());
    }

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("C:/o/nup dump/updates/131072/romfs"));
        if p.join("Pack").join("ZsDic.pack.zs").is_file() { Some(p) } else { None }
    }

    #[test]
    fn every_catalog_chest_finds_one_area_in_the_real_files() {
        let Some(p) = dump() else {
            eprintln!("skipped, no RomFS dump");
            return;
        };
        let d: Data = Data::load().unwrap();
        let mut rf: Romfs = Romfs::open(&p).unwrap();
        let mut w: BM<u64, Want> = BM::new();
        for l in d.cat.locs.iter().filter(|l| l.kind == "chest") {
            for r in &l.refs {
                if crate::special::of(r.hash).is_none() && d.event_mode(r.hash) != Some(Ev::Keep) {
                    w.insert(r.hash, Want { big: true, strip: true });
                }
            }
        }
        let scenes: BM<String, Vec<Seen>> = scan(&mut rf, &w).unwrap();
        let grow: Moves = moves(&scenes, &w);
        let flip: BM<u64, Want> = w.keys().map(|h| (*h, Want { big: false, strip: true })).collect();
        let shrink: Moves = moves(&scenes, &flip);
        let mut chests: usize = 0;
        let mut areas: usize = 0;
        for f in banc_files(&rf).unwrap() {
            let raw: Vec<u8> = rf.read_zs(&f).unwrap();
            for (ww, m) in [(&w, &grow), (&flip, &shrink)] {
                let mut y: Byml = Byml::parse(&raw).unwrap();
                let (n, k): (usize, usize) = apply(&mut y, &stem(&f), ww, m);
                chests += n;
                areas += k;
            }
        }
        let names = |s: &BS<u64>| -> BS<String> { d.cat.locs.iter().filter(|l| l.refs.iter().any(|r| s.contains(&r.hash))).map(|l| l.name.clone()).collect() };
        eprintln!("grown {} shrunk {} tight {}", grow.chests.len(), shrink.chests.len(), grow.tight.len());
        for n in names(&grow.tight) {
            eprintln!("tight: {n}");
        }
        assert_eq!(chests, areas);
        assert_eq!(grow.areas.len() + shrink.areas.len(), areas);
        assert!(!grow.chests.is_empty() && !shrink.chests.is_empty() && !grow.tight.is_empty());
    }
}
