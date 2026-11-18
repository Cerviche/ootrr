use std::collections::BTreeSet as Set;

use crate::ainb::build::{Spec, QUERY};
use crate::ainb::{audit, root, set_root, Ainb};
use crate::byml::{Byml, Node};
use crate::error::{Error, Res};
use crate::pickup_pack::rows;
use crate::qol::{edit_zs, Have};
use crate::romfs::Romfs;
use crate::sarc::{self, Entry};
use crate::staging::Out;
use crate::zs;

pub const GLOBAL: &str = "Pack/AI.Global.Product.111.pack.zs";
pub const ACTOR_INFO: &str = rows::ACTOR_INFO;
pub const LIST: &str = crate::chest_slot::LIST;
pub const GRAPH: &str = "AI/EnemyCommon.common.reactiondamagecolor.module.ainb";
pub const STALFOS_GRAPH: &str = "AI/Stalfos.utility.reactiondamagecolor.module.ainb";
pub const GRAPHS: [&str; 2] = [GRAPH, STALFOS_GRAPH];
pub const MODULES: [&str; 2] = ["EnemyCommon.common.reactiondamagecolor.module", "Stalfos.utility.reactiondamagecolor.module"];
pub const FLAG: &str = "MiniGame_Diving_Test";
pub const GET: &str = "QueryGetNonStructGameDataBool";
pub const CLEAR: &str = "OneShotSetNonStructGameDataBool";
pub const HEAL: &str = "OneShotLifeSetOwnLife";
pub const TIMER: &str = "SelectorGenericFrameCounter";
pub const HAS_LIFE: &str = "QueryLifeHasComponent";
pub const IS_DEAD: &str = "QueryDeathIsDead";
pub const IDLE: &str = "ExecuteGenericDummyAction";
pub const FRAMES: f32 = 30.0;
pub const TO_MAX: i32 = 1;
pub const HEAP_EXTRA: i32 = 0x8000;
pub const ADDED: usize = 15;
pub const REFS: [&str; 2] = ["Component/BSAParam/", "AI/AIInfo/"];

const PARALLEL: u16 = 3;
const SWITCH: u16 = 7;
const LINK_EXTRA: usize = 8;

fn flag(v: bool) -> Vec<u8> {
    (v as u32).to_le_bytes().to_vec()
}

fn switch<'a>(seed: &'a str, input: usize, yes: usize, no: usize) -> Spec<'a> {
    let mut s: Spec = Spec::composite(SWITCH, seed)
        .prop(0, "CalculateTiming", vec![0; 4])
        .prop(1, "ChildFrameSync", flag(false))
        .prop(1, "InputValue", flag(false))
        .prop(1, "IsNoSelectWhenChildBusy", flag(false))
        .linked(1, "Input", input, 0)
        .kid(yes, "True")
        .kid(no, "False");
    s.link_extra = LINK_EXTRA;
    s
}

fn both<'a>(seed: &'a str, first: usize, second: usize) -> Spec<'a> {
    Spec::composite(PARALLEL, seed).prop(0, "EndPolicy", vec![0; 4]).prop(0, "ResultPolicy", vec![0; 4]).kid(first, "").kid(second, "")
}

pub fn patch_graph(raw: &[u8], name: &str) -> Res<Vec<u8>> {
    let mut a: Ainb = Ainb::parse(raw)?;
    if !a.find_els(CLEAR).is_empty() || !a.find_els(GET).is_empty() {
        return Err(Error::Format("fairy helps enemies: the graph is already patched"));
    }
    let host: usize = root(&a).ok_or(Error::Format("fairy helps enemies: the damage color graph has no root"))?;
    if a.parent_of(host).is_ok() {
        return Err(Error::Msg(format!("fairy helps enemies: the root of {name} has a parent")));
    }
    let key: Vec<u8> = a.pool.add(FLAG).to_le_bytes().to_vec();
    let mut get: Spec = Spec::node(GET, "ootrr fairy helps enemies used")
        .prop(1, "DispError", flag(false))
        .prop(1, "OutputOnlyOnEnter", flag(false))
        .constant(0, "GameDataIndex", (-1i32).to_le_bytes().to_vec())
        .constant(1, "IsDirect", flag(false))
        .constant(3, "GameDataFullName", key.clone())
        .out(1, "Value");
    get.role = QUERY;
    let used: usize = a.add_node(&get)?;
    let has: usize = a.add_node(&Spec::query(HAS_LIFE, "ootrr fairy helps enemies has life").out(1, "Bool"))?;
    let dead: usize = a.add_node(&Spec::query(IS_DEAD, "ootrr fairy helps enemies is dead").spare(0, "DeathReason").spare(0, "SubDeathReason").out(1, "IsDead"))?;
    let heal: usize = a.add_node(
        &Spec::node(HEAL, "ootrr fairy helps enemies heal")
            .prop(1, "ExecuteEveryFrame", flag(false))
            .constant(0, "SetType", TO_MAX.to_le_bytes().to_vec())
            .constant(0, "SetValue", 0i32.to_le_bytes().to_vec()),
    )?;
    let gone: usize = a.add_node(&Spec::node(IDLE, "ootrr fairy helps enemies idle dead"))?;
    let alive: usize = a.add_node(&switch("ootrr fairy helps enemies alive", dead, gone, heal))?;
    let bare: usize = a.add_node(&Spec::node(IDLE, "ootrr fairy helps enemies idle no life"))?;
    let gate: usize = a.add_node(&switch("ootrr fairy helps enemies gate", has, alive, bare))?;
    let clear: usize = a.add_node(
        &Spec::node(CLEAR, "ootrr fairy helps enemies clear")
            .prop(1, "ExecuteEveryFrame", flag(false))
            .constant(0, "GameDataIndex", (-1i32).to_le_bytes().to_vec())
            .constant(1, "Value", flag(false))
            .constant(3, "GameDataFullName", key),
    )?;
    let wait: usize = a.add_node(&Spec::node(IDLE, "ootrr fairy helps enemies idle wait"))?;
    let timer: usize = a.add_node(
        &Spec::node(TIMER, "ootrr fairy helps enemies timer")
            .prop(1, "IsCountUpdateOnReset", flag(false))
            .prop(1, "IsLoop", flag(true))
            .prop(1, "NoChangeWhenChildIsBusy", flag(false))
            .prop(1, "SelectChildOnEnterOnly", flag(false))
            .prop(2, "CountFrames", FRAMES.to_le_bytes().to_vec())
            .prop(2, "RandomAddCountFrames", 0f32.to_le_bytes().to_vec())
            .spare(2, "CurrentFrame")
            .spare(2, "FinishFrame")
            .kid(wait, "NotFinish")
            .kid(clear, "Finish"),
    )?;
    let work: usize = a.add_node(&both("ootrr fairy helps enemies work", gate, timer))?;
    let calm: usize = a.add_node(&Spec::node(IDLE, "ootrr fairy helps enemies idle calm"))?;
    let poll: usize = a.add_node(&switch("ootrr fairy helps enemies poll", used, work, calm))?;
    let top: usize = a.add_node(&both("ootrr fairy helps enemies root", host, poll))?;
    set_root(&mut a, top)?;
    let bad: Vec<String> = audit(&a);
    if !bad.is_empty() {
        return Err(Error::Msg(format!("fairy helps enemies: {}", bad.join("; "))));
    }
    let out: Vec<u8> = a.write()?;
    let back: Ainb = Ainb::parse(&out)?;
    if back.write()? != out || back.params != a.params || back.ins != a.ins || back.outs != a.outs || back.qids != a.qids || root(&back) != Some(top) {
        return Err(Error::Format("fairy helps enemies: the written graph does not read back"));
    }
    Ok(out)
}

pub fn patch_entries(es: &mut Vec<Entry>) -> Res<bool> {
    let mut hit: bool = false;
    for name in GRAPHS {
        let Some(g) = sarc::get(es, name).map(|e| e.data.clone()) else {
            continue;
        };
        sarc::set(es, name, patch_graph(&g, name)?);
        hit = true;
    }
    Ok(hit)
}

fn has(d: &[u8], name: &str) -> bool {
    let m: &[u8] = name.as_bytes();
    d.windows(m.len() + 1).any(|w| &w[..m.len()] == m && w[m.len()] == 0)
}

fn calls(d: &[u8]) -> bool {
    MODULES.iter().any(|m| has(d, m))
}

fn is_caller(e: &Entry) -> bool {
    e.name.ends_with(".ainb") && !GRAPHS.contains(&e.name.as_str()) && calls(&e.data)
}

pub fn callers(es: &[Entry]) -> Set<String> {
    es.iter().filter(|e| is_caller(e)).map(|e| e.name.clone()).collect()
}

fn graph_of(path: &str) -> Option<String> {
    let f: &str = path.rsplit('/').next()?;
    f.ends_with(".ain").then(|| format!("AI/{f}b"))
}

fn names(n: &Node, shared: &Set<String>) -> bool {
    match n {
        Node::Str(s) => graph_of(s).is_some_and(|g| shared.contains(&g)),
        Node::Arr(v) => v.iter().any(|x| names(x, shared)),
        Node::Map(v) => v.iter().any(|(_, x)| names(x, shared)),
        _ => false,
    }
}

pub fn runs(es: &[Entry], shared: &Set<String>) -> bool {
    es.iter().any(|e| is_caller(e) || (REFS.iter().any(|d| e.name.starts_with(d)) && Byml::parse(&e.data).is_ok_and(|y| names(&y.root, shared))))
}

pub fn actor_of(pack: &str) -> Option<&str> {
    pack.rsplit('/').next()?.strip_suffix(".pack.zs")
}

fn heap_of(row: &Node) -> Option<i32> {
    match row.get(rows::HEAP)? {
        Node::I32(v) => Some(*v),
        Node::U32(v) => i32::try_from(*v).ok(),
        _ => None,
    }
}

pub fn grow_heap(raw: &[u8], actors: &Set<String>) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.as_arr_mut().ok_or(Error::Format("fairy helps enemies: the actor table is not a row list"))?;
    let mut want: Vec<(String, i32)> = Vec::new();
    for row in list.iter_mut() {
        let Some(name) = rows::id(row).filter(|n| actors.contains(*n)).map(str::to_string) else {
            continue;
        };
        let Some(old) = heap_of(row).filter(|v| *v > 0) else {
            continue;
        };
        let new: i32 = old.checked_add(HEAP_EXTRA).ok_or(Error::Format("fairy helps enemies: the heap size does not fit"))?;
        let v: Node = match row.get(rows::HEAP) {
            Some(Node::U32(_)) => Node::U32(new as u32),
            _ => Node::I32(new),
        };
        row.set(rows::HEAP, v);
        want.push((name, new));
    }
    let out: Vec<u8> = y.write()?;
    let back: Byml = Byml::parse(&out)?;
    if want.iter().any(|(name, new)| rows::find(&back, name).and_then(heap_of) != Some(*new)) {
        return Err(Error::Format("fairy helps enemies: the actor table does not read back"));
    }
    Ok(out)
}

fn zipped(rf: &Romfs, have: Have, path: &str) -> Res<Vec<u8>> {
    match have(path) {
        Some(b) => Ok(b),
        None => rf.read(path),
    }
}

fn visit(rf: &mut Romfs, have: Have, path: &str, shared: &Set<String>) -> Res<(bool, Option<Out>)> {
    let z: Vec<u8> = zipped(rf, have, path)?;
    let raw: Vec<u8> = rf.zs.decompress(&z)?;
    let mut es: Vec<Entry> = sarc::parse(&raw)?;
    let used: bool = runs(&es, shared);
    if !patch_entries(&mut es)? {
        return Ok((used, None));
    }
    let out: Vec<u8> = sarc::write(&es)?;
    Ok((used, Some(Out { path: path.to_string(), old: raw.len(), new: out.len(), z: zs::compress_raw(&out, zs::dict_of(&z))? })))
}

pub fn stage(rf: &mut Romfs, have: Have, dir: &str) -> Res<(Set<String>, Vec<Out>)> {
    let z: Vec<u8> = zipped(rf, have, GLOBAL)?;
    let shared: Set<String> = callers(&sarc::parse(&rf.zs.decompress(&z)?)?);
    let mut outs: Vec<Out> = Vec::new();
    outs.push(visit(rf, have, GLOBAL, &shared)?.1.ok_or(Error::Msg(format!("fairy helps enemies: the AI pack has no {GRAPH}")))?);
    let mut who: Set<String> = Set::new();
    for f in rf.list(dir)? {
        let Some(name) = actor_of(&f).map(str::to_string) else {
            continue;
        };
        let (used, out): (bool, Option<Out>) = visit(rf, have, &f, &shared)?;
        if used {
            who.insert(name);
        }
        outs.extend(out);
    }
    if who.is_empty() {
        return Err(Error::Format("fairy helps enemies: no actor runs the damage color graph"));
    }
    outs.push(edit_zs(rf, have, ACTOR_INFO, |raw| grow_heap(raw, &who))?);
    Ok((who, outs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ainb::show::show;

    #[test]
    fn a_call_is_a_whole_name_in_the_string_pool() {
        let mut d: Vec<u8> = b"x\0".to_vec();
        d.extend_from_slice(MODULES[0].as_bytes());
        assert!(!calls(&d));
        d.push(0);
        assert!(calls(&d));
        let mut longer: Vec<u8> = MODULES[1].as_bytes().to_vec();
        longer.extend_from_slice(b".other\0");
        assert!(!calls(&longer));
    }

    #[test]
    fn a_graph_path_turns_into_its_packed_name() {
        assert_eq!(graph_of("Work/AI/Root/Enemy/Enemy_Stalfos.utility.root.ain").as_deref(), Some("AI/Enemy_Stalfos.utility.root.ainb"));
        assert_eq!(graph_of("Work/BSA/SchedulerParam/Enemy_Stalfos.game__bsa__SchedulerParam.gyml"), None);
        assert_eq!(graph_of(""), None);
        assert_eq!(actor_of("Pack/Actor/Enemy_Stalfos.pack.zs"), Some("Enemy_Stalfos"));
        assert_eq!(actor_of("Pack/Actor/readme.txt"), None);
    }

    fn bsa(name: &str, graph: &str) -> Entry {
        let y: Byml = Byml { root: Node::Map(vec![("UtilityAinPath".to_string(), Node::Str(graph.to_string()))]), version: 7, big: false };
        Entry { name: name.to_string(), data: y.write().unwrap() }
    }

    fn graph(name: &str, pool: &[u8]) -> Entry {
        Entry { name: name.to_string(), data: pool.to_vec() }
    }

    #[test]
    fn an_actor_runs_the_module_through_its_own_graph_or_a_shared_one() {
        let mut call: Vec<u8> = MODULES[0].as_bytes().to_vec();
        call.push(0);
        let global: Vec<Entry> = vec![graph(GRAPH, &call), graph("AI/Enemy_Stalfos.utility.root.ainb", &call), graph("AI/Npc.utility.root.ainb", b"none\0")];
        let shared: Set<String> = callers(&global);
        assert_eq!(shared.into_iter().collect::<Vec<String>>(), vec!["AI/Enemy_Stalfos.utility.root.ainb".to_string()]);
        let shared: Set<String> = callers(&global);
        assert!(runs(&[bsa("Component/BSAParam/Enemy_Stalfos.game__component__BSAParam.bgyml", "Work/AI/Root/Enemy/Enemy_Stalfos.utility.root.ain")], &shared));
        assert!(!runs(&[bsa("Component/BSAParam/Npc.game__component__BSAParam.bgyml", "Work/AI/Root/Npc.utility.root.ain")], &shared));
        assert!(!runs(&[bsa("Component/Other/Enemy_Stalfos.bgyml", "Work/AI/Root/Enemy/Enemy_Stalfos.utility.root.ain")], &shared));
        assert!(runs(&[graph("AI/Enemy_Octorok.utility.root.ainb", &call)], &shared));
        assert!(!runs(&[graph(GRAPH, &call)], &shared));
    }

    fn row(name: &str, heap: Node) -> Node {
        Node::Map(vec![(rows::HEAP.to_string(), heap), (rows::ROW_ID.to_string(), Node::Str(name.to_string()))])
    }

    #[test]
    fn only_the_named_actors_with_a_heap_grow() {
        let y: Byml = Byml { root: Node::Arr(vec![row("Enemy_A", Node::I32(1000)), row("Enemy_B", Node::I32(0)), row("Npc_C", Node::I32(500)), row("Enemy_D", Node::U32(700))]), version: 7, big: false };
        let actors: Set<String> = ["Enemy_A", "Enemy_B", "Enemy_D", "Enemy_Missing"].iter().map(|s| s.to_string()).collect();
        let back: Byml = Byml::parse(&grow_heap(&y.write().unwrap(), &actors).unwrap()).unwrap();
        let at = |n: &str| -> Node { rows::find(&back, n).unwrap().get(rows::HEAP).unwrap().clone() };
        assert_eq!(at("Enemy_A"), Node::I32(1000 + HEAP_EXTRA));
        assert_eq!(at("Enemy_B"), Node::I32(0));
        assert_eq!(at("Npc_C"), Node::I32(500));
        assert_eq!(at("Enemy_D"), Node::U32((700 + HEAP_EXTRA) as u32));
    }

    #[test]
    fn a_file_that_is_not_a_graph_is_refused() {
        assert!(patch_graph(b"nope", GRAPH).is_err());
    }

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        if p.is_dir() { Some(p) } else { None }
    }

    #[test]
    fn the_game_has_the_flag_and_does_not_save_it() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let y: Byml = rf.read_byml(LIST).unwrap();
        let want: i64 = crate::chest_slot::key(FLAG) as i64;
        let num = |n: &Node| -> Option<i64> {
            match n {
                Node::U32(v) => Some(*v as i64),
                Node::I32(v) => Some(*v as i64),
                _ => None,
            }
        };
        let bools: &[Node] = y.root.get("Data").unwrap().get("Bool").unwrap().as_arr().unwrap();
        let mine: Vec<&Node> = bools.iter().filter(|b| b.get("Hash").and_then(num) == Some(want)).collect();
        assert_eq!(mine.len(), 1, "{FLAG} is a bool of the game");
        assert_eq!(mine[0].get("SaveFileIndex").and_then(num), Some(-1), "{FLAG} is not saved");
        assert_eq!(mine[0].get("DefaultValue"), Some(&Node::Bool(false)));
        let name: &[u8] = FLAG.as_bytes();
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(GLOBAL).unwrap()).unwrap();
        assert!(!es.iter().any(|e| e.data.windows(name.len()).any(|w| w == name)), "no shared graph uses {FLAG}");
    }

    #[test]
    fn the_game_graph_heals_while_the_flag_is_set_and_then_clears_it() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(GLOBAL).unwrap()).unwrap();
        let was: &Entry = sarc::get(&es, GRAPH).unwrap();
        let old: Ainb = Ainb::parse(&was.data).unwrap();
        assert_eq!(old.write().unwrap(), was.data, "the untouched graph rewrites byte for byte");
        let out: Vec<u8> = patch_graph(&was.data, GRAPH).unwrap();
        let a: Ainb = Ainb::parse(&out).unwrap();
        assert_eq!(a.els.len(), old.els.len() + ADDED);
        let top: usize = root(&a).unwrap();
        assert_eq!((a.el_type(top), a.role(top) & 4, a.role(0) & 4), (PARALLEL, 4, 0));
        let kids = |i: usize| -> Vec<usize> { a.params[i].group(2).iter().map(|g| g.a as usize).collect() };
        let poll: usize = kids(top)[1];
        assert_eq!((kids(top).len(), kids(top)[0], a.el_type(poll)), (2, root(&old).unwrap(), SWITCH));
        let work: usize = kids(poll)[0];
        assert_eq!((a.el_type(work), a.el_name(kids(poll)[1])), (PARALLEL, IDLE));
        let timer: usize = kids(work)[1];
        assert_eq!((a.el_type(kids(work)[0]), a.el_name(timer)), (SWITCH, TIMER));
        assert_eq!((a.el_name(kids(timer)[0]), a.el_name(kids(timer)[1])), (IDLE, CLEAR));
        let heal: usize = a.find_els(HEAL)[0];
        assert_eq!((a.in_s32(heal, "SetType"), a.in_s32(heal, "SetValue")), (Some(TO_MAX), Some(0)));
        let shown: String = show(&a);
        for line in ["prop bool IsLoop = true", "prop f32 CountFrames = 30", "out bool IsDead flags 0x2", "out bool Value flags 0x2", "in bool Value = false", "\"NotFinish\"", "\"Finish\""] {
            assert!(shown.contains(line), "{line} is missing from\n{shown}");
        }
        assert_eq!(shown.matches(&format!("in str GameDataFullName = \"{FLAG}\"")).count(), 2, "{shown}");
        assert!(patch_graph(&out, GRAPH).is_err(), "a patched graph is not patched twice");
        let mut packed: Vec<Entry> = es.clone();
        assert!(patch_entries(&mut packed).unwrap());
        let changed: Vec<&str> = packed.iter().zip(es.iter()).filter(|(x, y)| x.name != y.name || x.data != y.data).map(|(x, _)| x.name.as_str()).collect();
        assert_eq!(changed, vec![GRAPH]);
    }

    #[test]
    fn the_game_nodes_have_the_shapes_the_patch_copies() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(GLOBAL).unwrap()).unwrap();
        let mine: Ainb = Ainb::parse(&patch_graph(&sarc::get(&es, GRAPH).unwrap().data, GRAPH).unwrap()).unwrap();
        let shape = |a: &Ainb, i: usize| -> Vec<String> {
            let all: String = show(a);
            let from: usize = all.find(&format!("element {i} type")).unwrap();
            let body: &str = &all[from..];
            let end: usize = body.find(&format!("\nelement {} type", i + 1)).unwrap_or(body.len());
            body[..end].lines().skip(1).filter(|l| !l.contains("plug ")).map(|l| l.split(" = ").next().unwrap_or(l).split(" from ").next().unwrap_or(l).to_string()).collect()
        };
        let boss: Vec<Entry> = sarc::parse(&rf.read_zs("Pack/Actor/Enemy_DungeonBoss_Twinrova.pack.zs").unwrap()).unwrap();
        let twin: Ainb = Ainb::parse(&sarc::get(&boss, "AI/Enemy_DungeonBoss_Twinrova.utility.root.ainb").unwrap().data).unwrap();
        let theirs: usize = twin.find_els(TIMER).into_iter().find(|i| shape(&twin, *i).iter().any(|l| l.contains("IsLoop"))).unwrap();
        assert_eq!(shape(&mine, mine.find_els(TIMER)[0]), shape(&twin, theirs));
        let glass: Ainb = Ainb::parse(&sarc::get(&es, "AI/EnemyCommon.common.enableglasstruth.module.ainb").unwrap().data).unwrap();
        assert_eq!(shape(&mine, mine.find_els(IS_DEAD)[0]), shape(&glass, glass.find_els(IS_DEAD)[0]));
        let sw: usize = (0..glass.els.len()).find(|i| glass.el_type(*i) == SWITCH).unwrap();
        let my_sw: usize = (0..mine.els.len()).find(|i| mine.el_type(*i) == SWITCH).unwrap();
        assert_eq!(shape(&mine, my_sw), shape(&glass, sw));
        assert_eq!(mine.params[my_sw].pl, glass.params[sw].pl);
        assert_eq!(mine.params[my_sw].plugs[0].extra.len(), glass.params[sw].plugs[0].extra.len());
        let npc: Ainb = Ainb::parse(&sarc::get(&es, "AI/SimpleNpc.ExecuteInterruptReaction.module.ainb").unwrap().data).unwrap();
        assert_eq!(shape(&mine, mine.find_els(HAS_LIFE)[0]), shape(&npc, npc.find_els(HAS_LIFE)[0]));
        let base: Ainb = Ainb::parse(&sarc::get(&es, "AI/EnemyBase.utility.root.ainb").unwrap().data).unwrap();
        let par: usize = (0..base.els.len()).find(|i| base.el_type(*i) == PARALLEL).unwrap();
        assert_eq!(shape(&mine, root(&mine).unwrap()), shape(&base, par));
        let floor: Vec<Entry> = sarc::parse(&rf.read_zs("Pack/Actor/Enemy_Floormaster.pack.zs").unwrap()).unwrap();
        let down: Ainb = Ainb::parse(&sarc::get(&floor, "AI/Floormaster.action.interruptdown.module.ainb").unwrap().data).unwrap();
        assert_eq!(shape(&mine, mine.find_els(HEAL)[0]), shape(&down, down.find_els(HEAL)[0]));
        let stone: Vec<Entry> = sarc::parse(&rf.read_zs("Pack/Actor/AgonyStoneReactArea.pack.zs").unwrap()).unwrap();
        let area: Ainb = Ainb::parse(&sarc::get(&stone, "AI/AgonyStoneReactArea.root.ainb").unwrap().data).unwrap();
        assert_eq!(shape(&mine, mine.find_els(CLEAR)[0]), shape(&area, area.find_els(CLEAR)[0]));
        let away: Ainb = Ainb::parse(&sarc::get(&es, "AI/NpcCommon.IsLookAway.module.ainb").unwrap().data).unwrap();
        assert_eq!(shape(&mine, mine.find_els(GET)[0]), shape(&away, away.find_els(GET)[0]));
        let player: Ainb = Ainb::parse(&sarc::get(&es, "AI/Player.event.root.ainb").unwrap().data).unwrap();
        let full: usize = player.find_els("EventTriggerHealLifeMax")[0];
        let does: usize = player.params[full].group(2)[0].a as usize;
        assert_eq!((player.el_name(does), player.in_s32(does, "SetType"), player.in_s32(does, "SetValue")), (HEAL, Some(TO_MAX), Some(0)), "the player is healed to full the same way");
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    #[test]
    fn the_game_actors_that_run_the_graph_get_room() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let (who, outs): (Set<String>, Vec<Out>) = stage(&mut rf, &none, "Pack/Actor").unwrap();
        for a in ["Enemy_Stalfos", "Enemy_Stalfos_ForChild", "Enemy_Keese", "Enemy_Octorok", "Enemy_Spike", "Enemy_DungeonBoss_Gohma"] {
            assert!(who.contains(a), "{a} runs the graph");
        }
        for a in ["Player_Child", "Player_Adult", "Npc_Navi", "Enemy_Beamos", "Enemy_Stalfos_Dictionary"] {
            assert!(!who.contains(a), "{a} does not run the graph");
        }
        let paths: Vec<&str> = outs.iter().map(|x| x.path.as_str()).collect();
        assert_eq!(zs::dict_of(&outs[0].z), zs::dict_of(&rf.read(GLOBAL).unwrap()), "the AI pack keeps its dictionary");
        assert_eq!((paths[0], paths[paths.len() - 1]), (GLOBAL, ACTOR_INFO));
        assert!(paths.contains(&"Pack/Actor/Enemy_Stalfos.pack.zs") && paths.len() < 12, "{paths:?}");
        let last: usize = outs.len() - 1;
        assert!(outs[..last].iter().all(|x| x.new > x.old) && outs[last].new == outs[last].old);
        let first: Vec<Entry> = sarc::parse(&rf.zs.decompress(&outs[1].z).unwrap()).unwrap();
        for x in &outs[1..last] {
            let now: Vec<Entry> = sarc::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap();
            let old: Vec<Entry> = sarc::parse(&rf.read_zs(&x.path).unwrap()).unwrap();
            let changed: Vec<&str> = now.iter().zip(old.iter()).filter(|(a, b)| a.name != b.name || a.data != b.data).map(|(a, _)| a.name.as_str()).collect();
            assert_eq!(changed, vec![STALFOS_GRAPH], "{}", x.path);
            assert_eq!(sarc::get(&now, STALFOS_GRAPH).unwrap().data, sarc::get(&first, STALFOS_GRAPH).unwrap().data, "every copy is the same");
        }
        let was: Byml = rf.read_byml(ACTOR_INFO).unwrap();
        let now: Byml = Byml::parse(&rf.zs.decompress(&outs[last].z).unwrap()).unwrap();
        let (a, b): (&[Node], &[Node]) = (was.root.as_arr().unwrap(), now.root.as_arr().unwrap());
        assert_eq!(a.len(), b.len());
        let mut grown: usize = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            if x == y {
                continue;
            }
            let name: &str = rows::id(x).unwrap();
            assert!(who.contains(name), "{name} changed");
            assert_eq!(heap_of(y), Some(heap_of(x).unwrap() + HEAP_EXTRA));
            let mut z: Node = y.clone();
            z.set(rows::HEAP, x.get(rows::HEAP).unwrap().clone());
            assert_eq!(&z, x, "only the heap of {name} changed");
            grown += 1;
        }
        assert!(grown >= 80, "{grown} actors grew");
        let mut t: crate::rsizetable::Rst = crate::rsizetable::load(&mut rf).unwrap();
        let before: Vec<Option<u32>> = GRAPHS.iter().map(|g| t.get(g)).collect();
        assert!(before.iter().all(|v| v.is_some()), "both graphs have their own size table entry");
        for o in &outs {
            crate::staging::grow_out(&mut rf, &mut t, o).unwrap();
        }
        for (g, was) in GRAPHS.iter().zip(before) {
            assert!(t.get(g) > was, "the size table entry of {g} grew");
        }
    }
}
