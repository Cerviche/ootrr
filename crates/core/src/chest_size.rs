use std::collections::BTreeMap as BM;

use crate::byml::{Byml, Node};
use crate::error::Res;
use crate::placement::hash_of;
use crate::settings::seed::SeedFile;
use crate::settings::Settings;
use crate::slice::data::{Data, Ev, Item};

pub const SETTING: &str = "chests_match_contents";

/// The plain wooden chests, small model first. Other chests (the shield chest, invisible
/// chests, the minigame chests, A_03 and A_04) keep their model.
pub const PAIRS: [(&str, &str); 2] = [("Obj_TBox_A_01", "Obj_TBox_A_02"), ("Obj_TBox_A_01_NoBake", "Obj_TBox_A_02_NoBake")];

pub const NOTE: &str = "Chest size matches contents: chests holding major items and boss keys are big, chests holding anything else are small. Chests that kept their vanilla item, run a story scene or are not plain wooden chests keep their size. Experimental, untested in the game.";

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
pub fn wanted(d: &Data, seed: &SeedFile, chus: bool) -> BM<u64, Want> {
    let mut out: BM<u64, Want> = BM::new();
    for (loc, item) in &seed.placements {
        let (Some(l), Some(it)) = (d.cat.by_name(loc), d.item(item)) else {
            continue;
        };
        if l.kind == "bossheart" {
            continue;
        }
        for r in &l.refs {
            if crate::special::of(r.hash).is_some() {
                continue;
            }
            let ev: Option<Ev> = d.event_mode(r.hash);
            if ev == Some(Ev::Keep) {
                continue;
            }
            out.insert(r.hash, Want { big: major(it, chus), strip: ev == Some(Ev::Strip) });
        }
    }
    out
}

pub fn target(gyaml: &str, big: bool) -> Option<&'static str> {
    PAIRS.iter().find(|(s, b)| gyaml == *s || gyaml == *b).map(|(s, b)| if big { *b } else { *s })
}

pub fn apply(y: &mut Byml, w: &BM<u64, Want>) -> usize {
    let Some(list) = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()) else {
        return 0;
    };
    let mut n: usize = 0;
    for a in list.iter_mut() {
        let Some(want) = hash_of(a).and_then(|h| w.get(&h)) else {
            continue;
        };
        let g: &str = a.get("Gyaml").and_then(|v| v.as_str()).unwrap_or("");
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
    n
}

/// Resizes the chests of one scene file. None when no chest in it changes, else the new
/// file and how many chests changed.
pub fn resize(raw: &[u8], w: &BM<u64, Want>) -> Res<Option<(Vec<u8>, usize)>> {
    if w.is_empty() {
        return Ok(None);
    }
    let mut y: Byml = Byml::parse(raw)?;
    let n: usize = apply(&mut y, w);
    if n == 0 {
        return Ok(None);
    }
    Ok(Some((y.write()?, n)))
}

pub fn report(n: usize) -> String {
    format!("{n} chests changed size to match their contents.")
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
        assert_eq!(apply(&mut y, &w), 3);
        assert_eq!(
            gyamls(&y),
            vec!["Obj_TBox_A_02", "Obj_TBox_A_01_NoBake", "Obj_TBox_A_06", "Obj_TBox_A_01", "Obj_TBox_A_02", "Obj_TBox_A_02", "Obj_TBox_A_01"]
        );
        let raw: Vec<u8> = y.write().unwrap();
        assert!(resize(&raw, &w).unwrap().is_none());
        assert!(resize(&raw, &BM::new()).unwrap().is_none());
        let (back, n): (Vec<u8>, usize) = resize(&raw, &BM::from([(1, Want { big: false, strip: false })])).unwrap().unwrap();
        assert_eq!(n, 1);
        assert_eq!(gyamls(&Byml::parse(&back).unwrap())[0], "Obj_TBox_A_01");
    }
}
