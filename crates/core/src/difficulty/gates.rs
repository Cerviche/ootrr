use crate::ainb::build::{Spec, QUERY};
use crate::ainb::{audit, Ainb};
use crate::byml::{Byml, Node};
use crate::difficulty::ambush::{actor, euler, floats, fresh, link, list, text, vec3};
use crate::error::{Error as E, Res};
use crate::flow_locations::dict_id;
use crate::murmur::murmur3_32;
use crate::placement::hash_of;
use crate::qol::Have;
use crate::romfs::Romfs;
use crate::staging::{Out, BANC_DICT};
use crate::zs;

pub const MODULE: &str = "Logic/Module/Gimmick_AllDay_Skulltula/Gimmick_AllDay_Skulltula_115c.logic.module.ain";
pub const GRAPH: &str = "Logic/Gimmick_AllDay_Skulltula_115c.logic.module.ainb";
pub const NODE: &str = "ActorLogicSimpleActor";
pub const NAME: &str = "InstanceName";
pub const CREATE: &str = "Logic_Create";
pub const DELETE: &str = "Logic_Delete";
pub const RESPAWN: &str = "RespawnTrigger";
pub const BOOL: usize = 1;
pub const STR: usize = 3;
pub const SEED: &str = "ootrr enemy gates";
pub const NEAR: f32 = 0.05;
pub const SHARED: [&str; 2] = ["FarDeleteGroups", "SimultaneousGroups"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gate {
    pub name: &'static str,
    pub dir: &'static str,
    pub map: &'static str,
    pub guard: &'static str,
    pub guards: &'static [[f32; 3]],
    pub wall: &'static str,
    pub wall_at: [f32; 3],
    pub wall_scale: [f32; 3],
    pub wall_turn: f32,
    pub closed_forest_only: bool,
}

pub const GATES: [Gate; 1] = [Gate {
    name: "the Deku Babas before the Deku Tree",
    dir: "KokiriForest",
    map: "KokiriForest_Child",
    guard: "Enemy_Dekubaba_Withered",
    guards: &[[39.124573, 113.19064, -18.576721], [37.743263, 113.25967, -19.590355]],
    wall: "FldObj_AirWall_Player_GenerateNavMesh",
    wall_at: [38.43, 113.5, -19.08],
    wall_scale: [1.6, 1.5, 0.03],
    wall_turn: 0.0,
    closed_forest_only: true,
}];

pub fn file(g: &Gate, kind: &str) -> String {
    format!("Banc/{}/{}_{kind}.bcett.byml.zs", g.dir, g.map)
}

pub fn active(g: &Gate, forest_open: bool, entrances: bool) -> bool {
    !entrances && !(g.closed_forest_only && forest_open)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    pub guard: String,
    pub wall: String,
}

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < NEAR)
}

pub fn take(y: &mut Byml, g: &Gate) -> Res<Vec<Node>> {
    let acts: &mut Vec<Node> = list(y, "Actors")?;
    let mut found: Vec<Node> = Vec::new();
    for s in g.guards {
        let at: Vec<usize> = (0..acts.len()).filter(|&i| text(&acts[i], "Gyaml") == g.guard && vec3(&acts[i], "Translate").is_some_and(|t| near(t, *s))).collect();
        let [i] = at.as_slice() else {
            return Err(E::Msg(format!("enemy gates: {} has {} {} at {:?} and not one, the game files differ", g.map, at.len(), g.guard, s)));
        };
        if acts[*i].get("Presence").is_none() {
            return Err(E::Msg(format!("enemy gates: the {} at {:?} has no presence flag, the game files differ", g.guard, s)));
        }
        found.push(acts[*i].clone());
    }
    let hs: Vec<u64> = found.iter().filter_map(hash_of).collect();
    acts.retain(|a| !hash_of(a).is_some_and(|h| hs.contains(&h)));
    Ok(found)
}

pub fn id_for(gyaml: &str, seed: &str, taken: &[String]) -> String {
    let mut n: u32 = murmur3_32(seed.as_bytes(), 0) & 0xffff;
    let mut id: String = format!("{gyaml}_{n:04x}");
    while taken.contains(&id) {
        n = (n + 1) & 0xffff;
        id = format!("{gyaml}_{n:04x}");
    }
    id
}

pub fn place(y: &mut Byml, raws: &[&[u8]], g: &Gate, guards: Vec<Node>) -> Res<Vec<Pair>> {
    let mut pairs: Vec<Pair> = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    let mut refs: Vec<Node> = Vec::new();
    for (i, node) in guards.into_iter().enumerate() {
        let gh: u64 = hash_of(&node).ok_or(E::Format("enemy gates: a guard has no hash"))?;
        if list(y, "Actors")?.iter().any(|a| hash_of(a) == Some(gh)) {
            return Err(E::Msg(format!("enemy gates: the {} is already in the static map of {}", g.guard, g.map)));
        }
        let seed: String = format!("{SEED} {} {i}", g.map);
        let wh: u64 = fresh(raws, &format!("{seed} wall"));
        let gid: String = id_for(g.guard, &format!("{seed} guard"), &taken);
        taken.push(gid.clone());
        let wid: String = id_for(g.wall, &format!("{seed} wall id"), &taken);
        taken.push(wid.clone());
        let mut w: Node = actor(g.wall, wh, g.wall_at, euler(0.0, g.wall_turn));
        w.set("Scale", floats(g.wall_scale));
        if let Some(p) = node.get("Presence") {
            w.set("Presence", p.clone());
        }
        let acts: &mut Vec<Node> = list(y, "Actors")?;
        acts.push(node);
        acts.push(w);
        refs.push(link(&gid, gh));
        refs.push(link(&wid, wh));
        for name in SHARED {
            list(y, name)?.push(Node::Arr(vec![Node::U64(gh.min(wh)), Node::U64(gh.max(wh))]));
        }
        pairs.push(Pair { guard: gid, wall: wid });
    }
    let group: u64 = fresh(raws, &format!("{SEED} {} group", g.map));
    let mut n: Node = Node::Map(Vec::new());
    n.set("Hash", Node::U64(group));
    n.set("Logic", Node::Str(MODULE.to_string()));
    n.set("References", Node::Arr(refs));
    let gs: &mut Vec<Node> = list(y, "AiGroups")?;
    let i: usize = gs.iter().position(|x| hash_of(x).is_some_and(|v| v > group)).unwrap_or(gs.len());
    gs.insert(i, n);
    Ok(pairs)
}

fn named(a: &Ainb, id: &str) -> Vec<usize> {
    (0..a.els.len()).filter(|&i| a.prop_str(i, NAME) == Some(id)).collect()
}

pub fn wire(a: &mut Ainb, p: &Pair) -> Res<(usize, usize)> {
    for id in [&p.guard, &p.wall] {
        if !named(a, id).is_empty() {
            return Err(E::Msg(format!("enemy gates: the graph already has a node named {id}")));
        }
    }
    let one: Vec<u8> = 1u32.to_le_bytes().to_vec();
    let zero: Vec<u8> = 0u32.to_le_bytes().to_vec();
    let gname: Vec<u8> = a.pool.add(&p.guard).to_le_bytes().to_vec();
    let gseed: String = format!("{SEED} {}", p.guard);
    let mut gs: Spec = Spec::node(NODE, &gseed)
        .prop(STR, NAME, gname)
        .idle(BOOL, CREATE, one.clone())
        .idle(BOOL, DELETE, zero.clone())
        .idle(BOOL, RESPAWN, zero.clone())
        .out(BOOL, "Dead")
        .spare(BOOL, "Logic_Alive");
    gs.role = QUERY;
    let gi: usize = a.add_node(&gs)?;
    let wname: Vec<u8> = a.pool.add(&p.wall).to_le_bytes().to_vec();
    let wseed: String = format!("{SEED} {}", p.wall);
    let mut ws: Spec = Spec::node(NODE, &wseed)
        .prop(STR, NAME, wname)
        .idle(BOOL, CREATE, one)
        .linked(BOOL, DELETE, gi, 0)
        .idle(BOOL, RESPAWN, zero)
        .spare(BOOL, "Dead")
        .spare(BOOL, "Logic_Alive");
    ws.role = QUERY;
    let wi: usize = a.add_node(&ws)?;
    let bad: Vec<String> = audit(a);
    if !bad.is_empty() {
        return Err(E::Msg(format!("the gate graph no longer checks out: {}", bad.join(", "))));
    }
    Ok((gi, wi))
}

fn source(rf: &mut Romfs, have: Have, p: &str) -> Res<Vec<u8>> {
    match have(p) {
        Some(b) => Ok(b),
        None => rf.read(p),
    }
}

pub fn pairs_of(g: &Gate) -> Res<Vec<Pair>> {
    let mut out: Vec<Pair> = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    for i in 0..g.guards.len() {
        let seed: String = format!("{SEED} {} {i}", g.map);
        let gid: String = id_for(g.guard, &format!("{seed} guard"), &taken);
        taken.push(gid.clone());
        let wid: String = id_for(g.wall, &format!("{seed} wall id"), &taken);
        taken.push(wid.clone());
        out.push(Pair { guard: gid, wall: wid });
    }
    Ok(out)
}

pub fn stage(rf: &mut Romfs, have: Have, gates: &[Gate]) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = Vec::new();
    let mut all: Vec<Pair> = Vec::new();
    for g in gates {
        let (ps, pd): (String, String) = (file(g, "Static"), file(g, "Dynamic"));
        let zs_s: Vec<u8> = source(rf, have, &ps)?;
        let zs_d: Vec<u8> = source(rf, have, &pd)?;
        let dict_s: u32 = dict_id(&zs_s).unwrap_or(BANC_DICT);
        let dict_d: u32 = dict_id(&zs_d).unwrap_or(BANC_DICT);
        let raw_s: Vec<u8> = rf.zs.decompress(&zs_s)?;
        let raw_d: Vec<u8> = rf.zs.decompress(&zs_d)?;
        let mut ys: Byml = Byml::parse(&raw_s)?;
        let mut yd: Byml = Byml::parse(&raw_d)?;
        let guards: Vec<Node> = take(&mut yd, g)?;
        let ps_: Vec<Pair> = place(&mut ys, &[&raw_s, &raw_d], g, guards)?;
        all.extend(ps_);
        let bs: Vec<u8> = ys.write()?;
        let bd: Vec<u8> = yd.write()?;
        outs.push(Out { path: ps, old: raw_s.len(), new: bs.len(), z: zs::compress_raw(&bs, Some(dict_s))? });
        outs.push(Out { path: pd, old: raw_d.len(), new: bd.len(), z: zs::compress_raw(&bd, Some(dict_d))? });
    }
    let was: Vec<u8> = source(rf, have, GRAPH)?;
    let mut a: Ainb = Ainb::parse(&was)?;
    for p in &all {
        wire(&mut a, p)?;
    }
    let g: Vec<u8> = a.write()?;
    outs.push(Out { path: GRAPH.to_string(), old: was.len(), new: g.len(), z: g });
    Ok(outs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: Gate = Gate { name: "t", dir: "T", map: "T_Child", guard: "Enemy_X", guards: &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], wall: "Wall_Y", wall_at: [2.0, 3.0, 4.0], wall_scale: [1.0, 2.0, 0.1], wall_turn: 0.5, closed_forest_only: true };

    fn foe(h: u64, at: [f32; 3], flag: bool) -> Node {
        let mut a: Node = actor("Enemy_X", h, at, [0.0; 3]);
        if flag {
            a.set("Presence", Node::Map(vec![("FlagName".to_string(), Node::Str("F".to_string())), ("IsNegation".to_string(), Node::Bool(true))]));
        }
        a
    }

    fn dynamic() -> Byml {
        let acts: Vec<Node> = vec![foe(10, [1.0, 2.0, 3.0], true), foe(11, [9.0, 9.0, 9.0], true), foe(12, [4.0, 5.0, 6.0], true)];
        Byml { root: Node::Map(vec![("Actors".to_string(), Node::Arr(acts))]), version: 7, big: false }
    }

    fn fixed() -> Byml {
        let mut root: Node = Node::Map(Vec::new());
        root.set("Actors", Node::Arr(vec![actor("Other", 1, [0.0; 3], [0.0; 3])]));
        root.set("AiGroups", Node::Arr(vec![Node::Map(vec![("Hash".to_string(), Node::U64(u64::MAX))])]));
        root.set("FarDeleteGroups", Node::Arr(Vec::new()));
        root.set("SimultaneousGroups", Node::Arr(Vec::new()));
        Byml { root, version: 7, big: false }
    }

    #[test]
    fn the_guards_move_out_of_the_dynamic_map_with_their_flag() {
        let mut y: Byml = dynamic();
        let got: Vec<Node> = take(&mut y, &G).unwrap();
        assert_eq!(got.iter().filter_map(hash_of).collect::<Vec<u64>>(), vec![10, 12]);
        assert!(got.iter().all(|a| a.get("Presence").is_some()));
        let left: &[Node] = y.root.get("Actors").unwrap().as_arr().unwrap();
        assert_eq!(left.iter().filter_map(hash_of).collect::<Vec<u64>>(), vec![11]);
    }

    #[test]
    fn a_guard_that_is_missing_or_has_no_flag_is_refused() {
        let mut y: Byml = dynamic();
        y.root.set("Actors", Node::Arr(vec![foe(10, [1.0, 2.0, 3.0], true)]));
        assert!(take(&mut y, &G).is_err());
        let mut y: Byml = dynamic();
        y.root.set("Actors", Node::Arr(vec![foe(10, [1.0, 2.0, 3.0], false), foe(12, [4.0, 5.0, 6.0], true)]));
        assert!(take(&mut y, &G).is_err());
    }

    #[test]
    fn each_guard_gets_a_wall_with_its_own_flag_and_one_shared_graph_group() {
        let mut d: Byml = dynamic();
        let got: Vec<Node> = take(&mut d, &G).unwrap();
        let mut y: Byml = fixed();
        let ps: Vec<Pair> = place(&mut y, &[], &G, got).unwrap();
        assert_eq!(ps, pairs_of(&G).unwrap());
        let acts: &[Node] = y.root.get("Actors").unwrap().as_arr().unwrap();
        assert_eq!(acts.len(), 5);
        let walls: Vec<&Node> = acts.iter().filter(|a| text(a, "Gyaml") == "Wall_Y").collect();
        assert_eq!(walls.len(), 2);
        for w in &walls {
            assert_eq!(w.get("Presence"), acts[1].get("Presence"));
            assert_eq!(vec3(w, "Scale"), Some(G.wall_scale));
            assert_eq!(vec3(w, "Translate"), Some(G.wall_at));
        }
        let mut hs: Vec<u64> = acts.iter().filter_map(hash_of).collect();
        hs.sort();
        hs.dedup();
        assert_eq!(hs.len(), 5);
        let gs: &[Node] = y.root.get("AiGroups").unwrap().as_arr().unwrap();
        assert_eq!(gs.len(), 2);
        assert!(gs.windows(2).all(|w| hash_of(&w[0]) < hash_of(&w[1])));
        let g: &Node = gs.iter().find(|g| text(g, "Logic") == MODULE).unwrap();
        let rs: &[Node] = g.get("References").unwrap().as_arr().unwrap();
        assert_eq!(rs.iter().map(|r| text(r, "Id").to_string()).collect::<Vec<String>>(), vec![ps[0].guard.clone(), ps[0].wall.clone(), ps[1].guard.clone(), ps[1].wall.clone()]);
        for (r, a) in rs.iter().zip([&acts[1], &acts[2], &acts[3], &acts[4]]) {
            assert_eq!(r.get("Reference"), a.get("Hash"));
        }
        for name in SHARED {
            assert_eq!(y.root.get(name).unwrap().as_arr().unwrap().len(), 2);
        }
        let back: Byml = Byml::parse(&y.write().unwrap()).unwrap();
        assert_eq!(back.root, y.root);
    }

    #[test]
    fn a_guard_already_in_the_static_map_is_refused() {
        let mut y: Byml = fixed();
        let got: Vec<Node> = vec![foe(1, [1.0, 2.0, 3.0], true), foe(12, [4.0, 5.0, 6.0], true)];
        assert!(place(&mut y, &[], &G, got).is_err());
    }

    #[test]
    fn the_deku_tree_gate_is_skipped_where_it_would_add_a_requirement() {
        let g: &Gate = &GATES[0];
        assert!(active(g, false, false));
        assert!(!active(g, true, false));
        assert!(!active(g, false, true));
    }

    #[test]
    fn names_stay_distinct_and_stable() {
        let a: String = id_for("Enemy_X", "s", &[]);
        assert_eq!(a, id_for("Enemy_X", "s", &[]));
        assert_ne!(a, id_for("Enemy_X", "s", &[a.clone()]));
        assert!(a.starts_with("Enemy_X_") && a.len() == "Enemy_X_".len() + 4);
    }

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        if p.is_dir() { Some(p) } else { None }
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    #[test]
    fn the_game_files_take_the_deku_tree_gate() {
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let outs: Vec<Out> = stage(&mut rf, &none, &GATES).unwrap();
        assert_eq!(outs.len(), 3);
        let g: &Gate = &GATES[0];
        let ws: Byml = rf.read_byml(&file(g, "Static")).unwrap();
        let wd: Byml = rf.read_byml(&file(g, "Dynamic")).unwrap();
        let ns: Byml = Byml::parse(&rf.zs.decompress(&outs[0].z).unwrap()).unwrap();
        let nd: Byml = Byml::parse(&rf.zs.decompress(&outs[1].z).unwrap()).unwrap();
        let count = |y: &Byml, name: &str| y.root.get("Actors").unwrap().as_arr().unwrap().iter().filter(|a| text(a, "Gyaml") == name).count();
        assert_eq!(count(&wd, g.guard), 5);
        assert_eq!(count(&nd, g.guard), 3);
        assert_eq!(count(&ns, g.guard), count(&ws, g.guard) + 2);
        assert_eq!(count(&ns, g.wall), count(&ws, g.wall) + 2);
        assert_eq!(count(&nd, g.wall), count(&wd, g.wall));
        let all: usize = count(&ws, g.guard) + count(&wd, g.guard);
        assert_eq!(count(&ns, g.guard) + count(&nd, g.guard), all, "no guard is added or lost");
        let olds: &[Node] = ws.root.get("Actors").unwrap().as_arr().unwrap();
        assert_eq!(&ns.root.get("Actors").unwrap().as_arr().unwrap()[..olds.len()], olds);
        let hs: Vec<u64> = ns.root.get("Actors").unwrap().as_arr().unwrap().iter().filter_map(hash_of).collect();
        let mut u: Vec<u64> = hs.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), hs.len());
        let gs: &[Node] = ns.root.get("AiGroups").unwrap().as_arr().unwrap();
        assert!(gs.windows(2).all(|w| hash_of(&w[0]) < hash_of(&w[1])));
        assert_eq!(gs.iter().filter(|x| text(x, "Logic") == MODULE).count(), ws.root.get("AiGroups").unwrap().as_arr().unwrap().iter().filter(|x| text(x, "Logic") == MODULE).count() + 1);
        let raw: Vec<u8> = rf.read(GRAPH).unwrap();
        let old: Ainb = Ainb::parse(&raw).unwrap();
        let a: Ainb = Ainb::parse(&outs[2].z).unwrap();
        assert_eq!(a.els.len(), old.els.len() + 4);
        assert!(audit(&a).is_empty());
        let ps: Vec<Pair> = pairs_of(g).unwrap();
        for p in &ps {
            let gi: usize = (0..a.els.len()).find(|&i| a.prop_str(i, NAME) == Some(p.guard.as_str())).unwrap();
            let wi: usize = (0..a.els.len()).find(|&i| a.prop_str(i, NAME) == Some(p.wall.as_str())).unwrap();
            let x: crate::ainb::Input = a.input(BOOL, a.find_in(wi, BOOL, DELETE).unwrap()).unwrap();
            assert_eq!((x.src as usize, x.plug), (gi, 0));
        }
        let again: Vec<Out> = stage(&mut rf, &none, &GATES).unwrap();
        assert!(outs.iter().zip(again.iter()).all(|(x, y)| x.path == y.path && x.z == y.z));
    }
}
