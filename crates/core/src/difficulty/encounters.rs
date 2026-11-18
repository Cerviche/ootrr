use std::f32::consts::{FRAC_PI_2, PI};

use crate::byml::{Byml, Node};
use crate::difficulty::wake;
use crate::difficulty::ambush::{actor, euler, fresh, list, text, vec3, Frame};
use crate::error::Res;
use crate::flow_locations::dict_id;
use crate::placement::hash_of;
use crate::qol::Have;
use crate::romfs::Romfs;
use crate::staging::{Out, BANC_DICT};
use crate::zs;

pub const HAND: &str = "Enemy_DeadHand";
pub const FINGERS: &str = "Enemy_DeadHand_Hand";
pub const KNIGHT: &str = "Enemy_IronKnuckle_Silver";
pub const SEED: &str = "ootrr special encounters";
pub const GROUPS: [&str; 2] = ["FarDeleteGroups", "SimultaneousGroups"];
pub const RING: f32 = 3.0;
pub const NOTE: &str = "Special encounters: a Dead Hand waits in the dark hole in east Hyrule Field (next to the ReDead and the chest) and in the foggy hole at the foot of the Death Mountain trail (with the Baby Dodongos), and an Iron Knuckle (it gets up when you come within about 9 units) stands in the open fields of Hyrule Field between Kakariko, Lon Lon Ranch and the Kokiri Forest. They are in the child and the adult map and come back when the area is loaded again. Experimental, untested in the game.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    DeadHand,
    IronKnuckle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub name: &'static str,
    pub dir: &'static str,
    pub maps: &'static [&'static str],
    pub what: What,
    pub at: [f32; 3],
    pub turn: f32,
}

pub const PLACES: [Place; 3] = [
    Place { name: "dead_hand_hyrule_field_hole", dir: "HyruleField", maps: &["HyruleField_Child", "HyruleField_Adult"], what: What::DeadHand, at: [155.72, 135.84, 20.09], turn: 0.7887344 },
    Place { name: "dead_hand_death_mountain_hole", dir: "DeathMountainTrail", maps: &["DeathMountainTrail_Child", "DeathMountainTrail_Adult"], what: What::DeadHand, at: [-5.0, 58.17, -38.5], turn: 0.2617994 },
    Place { name: "iron_knuckle_hyrule_field", dir: "HyruleField", maps: &["HyruleField_Child", "HyruleField_Adult"], what: What::IronKnuckle, at: [134.0, 157.7, 168.5], turn: 0.0 },
];

pub fn file(dir: &str, map: &str, kind: &str) -> String {
    format!("Banc/{dir}/{map}_{kind}.bcett.byml.zs")
}

pub fn files() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in PLACES.iter() {
        for m in p.maps {
            let f: String = file(p.dir, m, "Static");
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

fn next(raws: &[&[u8]], taken: &mut Vec<u64>, seed: &str) -> u64 {
    let mut n: u32 = 0;
    loop {
        let h: u64 = fresh(raws, &format!("{seed} {n}"));
        if !taken.contains(&h) {
            taken.push(h);
            return h;
        }
        n += 1;
    }
}

fn reference(src: u64, dst: u64) -> Node {
    let mut l: Node = Node::Map(Vec::new());
    l.set("Dst", Node::U64(dst));
    l.set("Gyaml", Node::Str("Reference".to_string()));
    l.set("Name", Node::Str("Reference".to_string()));
    l.set("Src", Node::U64(src));
    l
}

pub fn dead_hand(y: &mut Byml, p: &Place, raws: &[&[u8]], map: &str) -> Res<usize> {
    let seed: String = format!("{SEED} {map} {}", p.name);
    let mut taken: Vec<u64> = Vec::new();
    let body: u64 = next(raws, &mut taken, &format!("{seed} body"));
    let ring: Frame = Frame { at: p.at, yaw: p.turn };
    let spots: [([f32; 3], f32, Option<(&str, Node)>); 4] = [
        ([0.0, 0.0, -RING], 0.0, Some(("DeadHandHand_IsEnableAppearDelay", Node::Bool(false)))),
        ([0.0, 0.0, RING], PI, Some(("DeadHandHand_AppearOrderNum", Node::I32(2)))),
        ([-RING, 0.0, 0.0], FRAC_PI_2, Some(("DeadHandHand_AppearOrderNum", Node::I32(3)))),
        ([RING, 0.0, 0.0], -FRAC_PI_2, Some(("DeadHandHand_AppearOrderNum", Node::I32(3)))),
    ];
    let mut hands: Vec<Node> = Vec::new();
    let mut ids: Vec<u64> = Vec::new();
    for (i, (off, face, dynamic)) in spots.iter().enumerate() {
        let id: u64 = next(raws, &mut taken, &format!("{seed} hand {i}"));
        let mut h: Node = actor(FINGERS, id, ring.point(*off), euler(0.0, ring.turn(*face)));
        if let Some((k, v)) = dynamic {
            h.set("Dynamic", Node::Map(vec![(k.to_string(), v.clone())]));
        }
        hands.push(h);
        ids.push(id);
    }
    let mut b: Node = actor(HAND, body, p.at, [0.0; 3]);
    b.set(
        "Dynamic",
        Node::Map(vec![
            ("DeadHand_HoldActorCancelSleepFrame".to_string(), Node::F32(1800.0)),
            ("DeadHand_IsManageHandNumByLife".to_string(), Node::Bool(true)),
            ("Drop__DropTable".to_string(), Node::Str("None".to_string())),
        ]),
    );
    b.set("Links", Node::Arr(ids.iter().map(|i| reference(body, *i)).collect()));
    b.set("Name", Node::Str("DeadHand".to_string()));
    let acts: &mut Vec<Node> = list(y, "Actors")?;
    acts.push(b);
    acts.extend(hands);
    let mut all: Vec<u64> = ids;
    all.push(body);
    all.sort();
    for g in GROUPS {
        list(y, g)?.push(Node::Arr(all.iter().map(|h| Node::U64(*h)).collect()));
    }
    Ok(5)
}

pub fn iron_knuckle(y: &mut Byml, p: &Place, raws: &[&[u8]], map: &str) -> Res<usize> {
    let mut taken: Vec<u64> = Vec::new();
    let id: u64 = next(raws, &mut taken, &format!("{SEED} {map} {}", p.name));
    let mut k: Node = actor(KNIGHT, id, p.at, euler(0.0, p.turn));
    k.set("Dynamic", Node::Map(vec![("Drop__DropTable".to_string(), Node::Str("None".to_string())), (wake::KEY.to_string(), Node::Bool(true))]));
    list(y, "Actors")?.push(k);
    Ok(1)
}

pub fn place(y: &mut Byml, p: &Place, raws: &[&[u8]], map: &str) -> Res<usize> {
    match p.what {
        What::DeadHand => dead_hand(y, p, raws, map),
        What::IronKnuckle => iron_knuckle(y, p, raws, map),
    }
}

fn source(rf: &Romfs, have: Have, p: &str) -> Res<Vec<u8>> {
    match have(p) {
        Some(b) => Ok(b),
        None => rf.read(p),
    }
}

fn one(rf: &mut Romfs, have: Have, dir: &str, map: &str, only: &[Place]) -> Res<Out> {
    let p: String = file(dir, map, "Static");
    let z: Vec<u8> = source(rf, have, &p)?;
    let dict: u32 = dict_id(&z).unwrap_or(BANC_DICT);
    let raw: Vec<u8> = rf.zs.decompress(&z)?;
    let land: Vec<u8> = rf.zs.decompress(&source(rf, have, &file(dir, map, "Dynamic"))?)?;
    let mut y: Byml = Byml::parse(&raw)?;
    for pl in only {
        place(&mut y, pl, &[&raw, &land], map)?;
    }
    let bytes: Vec<u8> = y.write()?;
    Ok(Out { path: p, old: raw.len(), new: bytes.len(), z: zs::compress_raw(&bytes, Some(dict))? })
}

pub fn stage_places(rf: &mut Romfs, have: Have, places: &[Place]) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = Vec::new();
    for f in files_of(places) {
        let (dir, map): (&str, &str) = f;
        let here: Vec<Place> = places.iter().filter(|p| p.dir == dir && p.maps.contains(&map)).copied().collect();
        outs.push(one(rf, have, dir, map, &here)?);
    }
    Ok(outs)
}

fn files_of(places: &[Place]) -> Vec<(&'static str, &'static str)> {
    let mut out: Vec<(&'static str, &'static str)> = Vec::new();
    for p in places {
        for m in p.maps {
            if !out.contains(&(p.dir, *m)) {
                out.push((p.dir, *m));
            }
        }
    }
    out
}

pub fn stage(rf: &mut Romfs, have: Have) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = stage_places(rf, have, &PLACES)?;
    if PLACES.iter().any(|p| p.what == What::IronKnuckle) {
        outs.push(wake::stage(rf, have)?);
    }
    Ok(outs)
}

pub fn around(y: &Byml, at: [f32; 3]) -> Vec<(String, f32)> {
    let acts: &[Node] = y.root.get("Actors").and_then(|a| a.as_arr()).unwrap_or(&[]);
    acts.iter()
        .filter_map(|a| {
            let t: [f32; 3] = vec3(a, "Translate")?;
            Some((text(a, "Gyaml").to_string(), ((t[0] - at[0]).powi(2) + (t[1] - at[1]).powi(2) + (t[2] - at[2]).powi(2)).sqrt()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> Byml {
        let mut g: Node = Node::Map(Vec::new());
        g.set("Hash", Node::U64(5));
        let mut root: Node = Node::Map(Vec::new());
        root.set("Actors", Node::Arr(vec![actor("Enemy_Octorok", 5, [0.0; 3], [0.0; 3])]));
        root.set("FarDeleteGroups", Node::Arr(vec![Node::Arr(vec![Node::U64(5), Node::U64(6)])]));
        root.set("SimultaneousGroups", Node::Arr(vec![Node::Arr(vec![Node::U64(5), Node::U64(6)])]));
        Byml { root, version: 7, big: false }
    }

    fn actors(y: &Byml) -> &[Node] {
        y.root.get("Actors").unwrap().as_arr().unwrap()
    }

    fn count(y: &Byml, name: &str) -> usize {
        actors(y).iter().filter(|a| text(a, "Gyaml") == name).count()
    }

    fn close(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < tol)
    }

    #[test]
    fn a_dead_hand_comes_with_four_hands_that_it_links_to_and_one_group() {
        let mut y: Byml = map();
        let p: Place = PLACES[0];
        assert_eq!(place(&mut y, &p, &[], "Test").unwrap(), 5);
        assert_eq!((count(&y, HAND), count(&y, FINGERS)), (1, 4));
        let body: &Node = actors(&y).iter().find(|a| text(a, "Gyaml") == HAND).unwrap();
        assert_eq!(vec3(body, "Translate"), Some(p.at));
        assert_eq!(text(body, "Name"), "DeadHand");
        assert_eq!(body.get("Dynamic").and_then(|d| d.get("DeadHand_IsManageHandNumByLife")), Some(&Node::Bool(true)));
        let links: &[Node] = body.get("Links").unwrap().as_arr().unwrap();
        assert_eq!(links.len(), 4);
        let me: u64 = hash_of(body).unwrap();
        for l in links {
            assert_eq!(l.get("Src"), Some(&Node::U64(me)));
            assert_eq!(text(l, "Gyaml"), "Reference");
            let Some(Node::U64(dst)) = l.get("Dst") else { panic!() };
            let hand: &Node = actors(&y).iter().find(|a| hash_of(a) == Some(*dst)).unwrap();
            assert_eq!(text(hand, "Gyaml"), FINGERS);
            let t: [f32; 3] = vec3(hand, "Translate").unwrap();
            assert!(((t[0] - p.at[0]).powi(2) + (t[2] - p.at[2]).powi(2)).sqrt() - RING < 0.01 && t[1] == p.at[1]);
        }
        let mut hs: Vec<u64> = actors(&y).iter().filter_map(hash_of).collect();
        hs.sort();
        hs.dedup();
        assert_eq!(hs.len(), 6);
        for g in GROUPS {
            let gs: &[Node] = y.root.get(g).unwrap().as_arr().unwrap();
            assert_eq!(gs.len(), 2);
            let ids: Vec<u64> = gs[1].as_arr().unwrap().iter().map(|n| if let Node::U64(v) = n { *v } else { 0 }).collect();
            assert_eq!(ids.len(), 5);
            assert!(ids.windows(2).all(|w| w[0] < w[1]) && ids.contains(&me));
        }
        let back: Byml = Byml::parse(&y.write().unwrap()).unwrap();
        assert_eq!(back.root, y.root);
    }

    #[test]
    fn the_hands_face_the_middle_like_the_vanilla_ring() {
        let mut y: Byml = map();
        let p: Place = Place { turn: 0.0, ..PLACES[1] };
        dead_hand(&mut y, &p, &[], "Test").unwrap();
        let hands: Vec<&Node> = actors(&y).iter().filter(|a| text(a, "Gyaml") == FINGERS).collect();
        let north: &Node = hands.iter().find(|h| vec3(h, "Translate").unwrap()[2] < p.at[2] - 1.0).unwrap();
        assert!(close(vec3(north, "Translate").unwrap(), [p.at[0], p.at[1], p.at[2] - RING], 0.001));
        assert_eq!(vec3(north, "Rotate"), Some([0.0, 0.0, 0.0]));
        let south: &Node = hands.iter().find(|h| vec3(h, "Translate").unwrap()[2] > p.at[2] + 1.0).unwrap();
        assert!(close(vec3(south, "Rotate").unwrap(), [PI, 0.0, PI], 0.001));
        let west: &Node = hands.iter().find(|h| vec3(h, "Translate").unwrap()[0] < p.at[0] - 1.0).unwrap();
        assert!(close(vec3(west, "Rotate").unwrap(), [0.0, FRAC_PI_2, 0.0], 0.001));
        let order = |h: &Node, k: &str| h.get("Dynamic").and_then(|d| d.get(k)).cloned();
        assert_eq!(order(north, "DeadHandHand_IsEnableAppearDelay"), Some(Node::Bool(false)));
        assert_eq!(order(south, "DeadHandHand_AppearOrderNum"), Some(Node::I32(2)));
        assert_eq!(order(west, "DeadHandHand_AppearOrderNum"), Some(Node::I32(3)));
    }

    #[test]
    fn an_iron_knuckle_is_one_plain_actor_with_no_links_or_groups() {
        let mut y: Byml = map();
        let p: Place = PLACES[2];
        assert_eq!(place(&mut y, &p, &[], "Test").unwrap(), 1);
        assert_eq!(count(&y, KNIGHT), 1);
        let k: &Node = actors(&y).iter().find(|a| text(a, "Gyaml") == KNIGHT).unwrap();
        assert_eq!(vec3(k, "Translate"), Some(p.at));
        assert!(k.get("Links").is_none() && k.get("Name").is_none());
        assert_eq!(k.get("Dynamic").and_then(|d| d.get("Drop__DropTable")), Some(&Node::Str("None".to_string())));
        assert_eq!(k.get("Dynamic").and_then(|d| d.get(wake::KEY)), Some(&Node::Bool(true)));
        for g in GROUPS {
            assert_eq!(y.root.get(g).unwrap().as_arr().unwrap().len(), 1);
        }
    }

    #[test]
    fn two_places_in_one_map_do_not_share_a_hash_and_a_broken_map_is_refused() {
        let mut y: Byml = map();
        for p in PLACES.iter().filter(|p| p.dir == "HyruleField") {
            place(&mut y, p, &[], "Test").unwrap();
        }
        let mut hs: Vec<u64> = actors(&y).iter().filter_map(hash_of).collect();
        let n: usize = hs.len();
        hs.sort();
        hs.dedup();
        assert_eq!((n, hs.len()), (7, 7));
        let mut bad: Byml = map();
        bad.root.remove("FarDeleteGroups");
        assert!(dead_hand(&mut bad, &PLACES[0], &[], "Test").is_err());
    }

    #[test]
    fn each_place_is_listed_once_and_the_files_follow_the_places() {
        let names: std::collections::BTreeSet<&str> = PLACES.iter().map(|p| p.name).collect();
        assert_eq!(names.len(), PLACES.len());
        assert_eq!(files().len(), 4);
        assert!(files().iter().all(|f| f.ends_with("_Static.bcett.byml.zs")));
        assert_eq!(files_of(&PLACES[..1]).len(), 2);
    }

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        if p.is_dir() { Some(p) } else { None }
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    #[test]
    fn the_game_maps_take_each_encounter_alone_and_together() {
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        for single in 0..=PLACES.len() {
            let chosen: Vec<Place> = if single == PLACES.len() { PLACES.to_vec() } else { vec![PLACES[single]] };
            let outs: Vec<Out> = stage_places(&mut rf, &none, &chosen).unwrap();
            let mut paths: Vec<String> = Vec::new();
            for p in &chosen {
                for m in p.maps {
                    let f: String = file(p.dir, m, "Static");
                    if !paths.contains(&f) {
                        paths.push(f);
                    }
                }
            }
            assert_eq!(outs.iter().map(|o| o.path.clone()).collect::<Vec<String>>(), paths);
            for o in &outs {
                let was: Byml = rf.read_byml(&o.path).unwrap();
                let now: Byml = Byml::parse(&rf.zs.decompress(&o.z).unwrap()).unwrap();
                let here: Vec<&Place> = chosen.iter().filter(|p| p.maps.iter().any(|m| o.path == file(p.dir, m, "Static"))).collect();
                let hands: usize = here.iter().filter(|p| p.what == What::DeadHand).count();
                let knights: usize = here.iter().filter(|p| p.what == What::IronKnuckle).count();
                assert!(o.new > o.old);
                assert_eq!(count(&now, HAND), count(&was, HAND) + hands, "{}", o.path);
                assert_eq!(count(&now, FINGERS), count(&was, FINGERS) + 4 * hands, "{}", o.path);
                assert_eq!(count(&now, KNIGHT), count(&was, KNIGHT) + knights, "{}", o.path);
                assert_eq!(actors(&now).len(), actors(&was).len() + 5 * hands + knights);
                assert_eq!(&actors(&now)[..actors(&was).len()], actors(&was), "the actors that were there are untouched");
                let mut hs: Vec<u64> = actors(&now).iter().filter_map(hash_of).collect();
                let n: usize = hs.len();
                hs.sort();
                hs.dedup();
                assert_eq!(hs.len(), n, "no hash is used twice in {}", o.path);
                for g in GROUPS {
                    let a: usize = was.root.get(g).unwrap().as_arr().unwrap().len();
                    let b: usize = now.root.get(g).unwrap().as_arr().unwrap().len();
                    assert_eq!(b, a + hands, "{g} in {}", o.path);
                }
                assert_eq!(now.root.get("AiGroups"), was.root.get("AiGroups"));
                for a in &actors(&now)[actors(&was).len()..] {
                    let t: [f32; 3] = vec3(a, "Translate").unwrap();
                    let near: Vec<(String, f32)> = around(&was, t);
                    if text(a, "Gyaml") == HAND || text(a, "Gyaml") == KNIGHT {
                        for (g, d) in &near {
                            if g.starts_with("Enemy_") || g.starts_with("Obj_TBox") {
                                assert!(*d > 4.0, "{} sits {d} from {g}", text(a, "Gyaml"));
                            }
                        }
                    }
                    if text(a, "Gyaml") == HAND {
                        let landing: &Node = actors(&was)
                            .iter()
                            .filter(|x| text(x, "Name") == "SwapRestartPosAnchor")
                            .min_by(|x, y| {
                                let d = |n: &Node| -> f32 {
                                    let l: [f32; 3] = vec3(n, "Translate").unwrap();
                                    ((l[0] - t[0]).powi(2) + (l[2] - t[2]).powi(2)).sqrt()
                                };
                                d(x).total_cmp(&d(y))
                            })
                            .unwrap();
                        let l: [f32; 3] = vec3(landing, "Translate").unwrap();
                        let d: f32 = ((l[0] - t[0]).powi(2) + (l[2] - t[2]).powi(2)).sqrt();
                        assert!(d > 6.5, "the Dead Hand sits {d} from where Link lands");
                        assert!((t[1] - l[1]).abs() < 0.6, "the Dead Hand is level with the floor");
                    }
                    if text(a, "Gyaml") == HAND || text(a, "Gyaml") == FINGERS {
                        let exit: [f32; 3] = actors(&was)
                            .iter()
                            .filter(|x| text(x, "Name") == "HidingHole_FieldExitGo")
                            .filter_map(|x| vec3(x, "Translate"))
                            .min_by(|x, y| ((x[0] - t[0]).powi(2) + (x[2] - t[2]).powi(2)).total_cmp(&((y[0] - t[0]).powi(2) + (y[2] - t[2]).powi(2))))
                            .unwrap();
                        let far: f32 = ((exit[0] - t[0]).powi(2) + (exit[2] - t[2]).powi(2)).sqrt();
                        assert!(far >= 6.0, "{} sits {far} from where Link lands", text(a, "Gyaml"));
                    }
                }
            }
        }
    }
}
