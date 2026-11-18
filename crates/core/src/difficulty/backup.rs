use std::collections::{BTreeMap as Map, BTreeSet as Set};

use crate::ainb::build::Spec;
use crate::ainb::{audit, root, set_root, Ainb, Plug};
use crate::byml::{Byml, Node};
use crate::error::{Error, Res};
use crate::pickup_pack::rows;
use crate::pickup_pack::sizes;
use crate::qol::{edit_zs, Have};
use crate::romfs::Romfs;
use crate::rsizetable::Rst;
use crate::sarc::{self, Entry};
use crate::staging::Out;
use crate::zs;

pub const GLOBAL: &str = "Pack/AI.Global.Product.111.pack.zs";
pub const ACTOR_INFO: &str = rows::ACTOR_INFO;
pub const GRAPH: &str = "AI/EnemyCommon.common.reactiondamagecolor.module.ainb";
pub const STALFOS_GRAPH: &str = "AI/Stalfos.utility.reactiondamagecolor.module.ainb";
pub const GRAPHS: [&str; 2] = [GRAPH, STALFOS_GRAPH];
pub const MODULES: [&str; 2] = ["EnemyCommon.common.reactiondamagecolor.module", "Stalfos.utility.reactiondamagecolor.module"];
pub const HOST: &str = "ExecuteSetEnemyDamageColorAndLight";
pub const REQUEST: &str = "OneShotActorHolderTryRequestCreateActor";
pub const WAKE: &str = "OneShotActorHolderTryCancelSleepActor";
pub const READY: &str = "QueryActorHolderIsExistEnableCancelSleepActorHolderData";
pub const EMPTY: &str = "QueryActorHolderIsEmptyActorHolderData";
pub const FIGHT: &str = "QueryTargetIsValid";
pub const PLACE: &str = "QueryActorGetPostionAndRotation";
pub const SPLIT: &str = "QueryMathMtx34fToVec3fMat33f";
pub const LIFT: &str = "QueryMathAddVector3f";
pub const TIMER: &str = "SelectorGenericFrameCounter";
pub const IDLE: &str = "ExecuteGenericDummyAction";
pub const GETB: &str = "QueryBlackboardGetBoolNoError";
pub const SETB: &str = "ExecuteBlackboardSetBool";
pub const KEY: &str = "Backup";
pub const ELIGIBLE: &str = "OotrrBackupEligible";
pub const USED: &str = "OotrrBackupUsed";
pub const TABLE: &str = "Component/Blackboard/BlackboardParamTable/EnemyCommon.engine__component__BlackboardParamTable.bgyml";
pub const BOOLS: &str = "BlackboardParamBoolArray";
pub const CALLERS: &str = "Enemy_";
pub const HELPER: &str = "Enemy_Peahat_Larva_Independent";
pub const HELPERS: [(&str, &str); 3] = [
    (HELPER, NEW),
    ("Enemy_Shabom", "Component/ActorHolderParam/OotrrBackupBubble.game__component__ActorHolderParam.bgyml"),
    ("Enemy_FlyingPot", "Component/ActorHolderParam/OotrrBackupPot.game__component__ActorHolderParam.bgyml"),
];
pub const OUT_OF_BOUNDS: [&str; 8] = ["Enemy_DungeonBoss_", "Enemy_MiddleBoss_", "Enemy_IronKnuckle_", "Enemy_DeadHand", "Enemy_PhantomGanon", "Enemy_GhostPicturePerformer_", "Enemy_Guay_Boss", "Enemy_KotakeKoumeBattleManager"];
pub const BUBBLE_CALLERS: [&str; 8] = ["Enemy_Bari", "Enemy_Biri", "Enemy_Stinger", "Enemy_Stinger_UnderWater", "Enemy_ShellBlade", "Enemy_Tentacle_Blue", "Enemy_Tentacle_Green", "Enemy_Tentacle_Red"];
pub const POT_CALLERS: [&str; 11] = ["Enemy_Freezard", "Enemy_Freezard_Fixed", "Enemy_Gibdo", "Enemy_ReDead", "Enemy_ReDead_AlwaysRespawn", "Enemy_ReDead_ClimaxOnly", "Enemy_Wallmaster", "Enemy_LikeLike", "Enemy_Floormaster", "Enemy_Floormaster_Little", "Enemy_Floormaster_Truth"];
pub const FRAMES: f32 = 750.0;
pub const RISE: f32 = 1.5;
pub const MAX: i32 = 1;
pub const HEAP_EXTRA: i32 = 0x10000;
pub const REFS: [&str; 2] = ["Component/BSAParam/", "AI/AIInfo/"];
pub const ACTOR_DIR: &str = "Actor/";
pub const ACTOR_EXT: &str = ".engine__actor__ActorParam.bgyml";
pub const HOLDER_DIR: &str = "Component/ActorHolderParam/";
pub const NEW: &str = "Component/ActorHolderParam/OotrrBackup.game__component__ActorHolderParam.bgyml";
pub const PEER: (&str, &str) = ("Pack/Actor/Enemy_Bari.pack.zs", "Component/ActorHolderParam/Enemy_Bari.game__component__ActorHolderParam.bgyml");
pub const PARTS: &str = "Components";
pub const SLOT: &str = "ActorHolderRef";
pub const HELD: &str = "ActorHoldSettings";
pub const PARENT: &str = "$parent";
pub const VERSION: u16 = 7;

const PARALLEL: u16 = 3;
const SWITCH: u16 = 7;
const LINK_EXTRA: usize = 8;
const DEPTH: usize = 16;
pub const MAT34: &str = "const sead::Matrix34f";
pub const MAT33: &str = "const sead::Matrix33f";
const OVER: &str = "bb::OverwriteParam";

pub fn flag(v: bool) -> Vec<u8> {
    (v as u32).to_le_bytes().to_vec()
}

fn vec3(x: f32, y: f32, z: f32) -> Vec<u8> {
    [x, y, z].iter().flat_map(|v| v.to_le_bytes()).collect()
}

pub fn switch<'a>(seed: &'a str, input: usize, yes: usize, no: usize) -> Spec<'a> {
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

pub fn flag_query<'a>(seed: &'a str, key: Vec<u8>) -> Spec<'a> {
    Spec::query(GETB, seed).prop(0, "BlackboardType", 1i32.to_le_bytes().to_vec()).prop(3, "BlackboardKey", key).out(1, "Bool")
}

fn both<'a>(seed: &'a str, first: usize, second: usize) -> Spec<'a> {
    Spec::composite(PARALLEL, seed).prop(0, "EndPolicy", vec![0; 4]).prop(0, "ResultPolicy", vec![0; 4]).kid(first, "").kid(second, "")
}

pub fn feed(a: &mut Ainb, from: usize, to: usize, out: &str) -> Res<()> {
    let name: u32 = a.pool.add(out);
    let reader: &mut crate::ainb::Param = a.params.get_mut(to).ok_or(Error::Format("enemy backup: no such reader"))?;
    let n: u8 = reader.pl[0][0];
    if n != 1 || reader.plugs.len() != 1 || reader.plugs[0].a as usize != from {
        return Err(Error::Format("enemy backup: the reader does not have one data link"));
    }
    reader.plugs.clear();
    reader.pl[0][0] = 0;
    for g in reader.pl.iter_mut().skip(1) {
        g[1] -= n;
    }
    let source: &mut crate::ainb::Param = a.params.get_mut(from).ok_or(Error::Format("enemy backup: no such source"))?;
    let at: usize = source.pl[0][0] as usize;
    source.plugs.insert(at, Plug { a: to as u32, b: name, extra: Vec::new() });
    source.pl[0][0] += 1;
    for g in source.pl.iter_mut().skip(1) {
        g[1] += 1;
    }
    Ok(())
}

pub fn patch_graph(raw: &[u8], name: &str) -> Res<Vec<u8>> {
    let mut a: Ainb = Ainb::parse(raw)?;
    if !a.find_els(REQUEST).is_empty() {
        return Err(Error::Format("enemy backup: the graph is already patched"));
    }
    let host: usize = root(&a).ok_or(Error::Format("enemy backup: the damage color graph has no root"))?;
    if a.find_els(HOST).len() != 1 || a.parent_of(host).is_ok() {
        return Err(Error::Msg(format!("enemy backup: {name} no longer runs one {HOST}")));
    }
    let key: Vec<u8> = a.pool.add(KEY).to_le_bytes().to_vec();
    let ready: usize = a.add_node(&Spec::query(READY, "ootrr enemy backup ready").constant(3, "Key", key.clone()).out(1, "IsExist"))?;
    let empty: usize = a.add_node(&Spec::query(EMPTY, "ootrr enemy backup empty").constant(3, "Key", key.clone()).out(1, "Bool"))?;
    let fight: usize = a.add_node(&Spec::query(FIGHT, "ootrr enemy backup fight").idle(0, "TargetSlot", vec![0; 4]).out(1, "IsValid"))?;
    let place: usize = a.add_node(&Spec::query(PLACE, "ootrr enemy backup place").out_ptr("Mat34F", MAT34))?;
    let split: usize = a.add_node(&Spec::query(SPLIT, "ootrr enemy backup split").linked_ptr("Matrix34f", MAT34, place).out(4, "Translation").spare_ptr("Rotation", MAT33))?;
    feed(&mut a, place, split, "Mat34F")?;
    let lift: usize = a.add_node(&Spec::query(LIFT, "ootrr enemy backup lift").linked(4, "InputA", split, 0).constant(4, "InputB", vec3(0.0, RISE, 0.0)).out(4, "Result"))?;
    feed(&mut a, split, lift, "Translation")?;
    let wake: usize = a.add_node(
        &Spec::node(WAKE, "ootrr enemy backup wake")
            .prop(1, "ExecuteEveryFrame", flag(true))
            .prop(1, "IsForeachActor", flag(false))
            .idle(0, "Index", vec![0; 4])
            .constant(3, "Key", key.clone())
            .linked(4, "Pos", lift, 0)
            .idle(4, "Scale", vec3(1.0, 1.0, 1.0))
            .idle_ptr("OverwriteParam", OVER)
            .idle_ptr("Rot", MAT33),
    )?;
    feed(&mut a, lift, wake, "Result")?;
    let eligible_key: Vec<u8> = a.pool.add(ELIGIBLE).to_le_bytes().to_vec();
    let used_key: Vec<u8> = a.pool.add(USED).to_le_bytes().to_vec();
    let eligible: usize = a.add_node(&flag_query("ootrr enemy backup eligible", eligible_key))?;
    let used: usize = a.add_node(&flag_query("ootrr enemy backup used", used_key.clone()))?;
    let latch: usize = a.add_node(
        &Spec::node(SETB, "ootrr enemy backup latch")
            .prop(0, "BlackboardType", 1i32.to_le_bytes().to_vec())
            .prop(1, "ExecuteOnce", flag(true))
            .prop(1, "IsIgnoreNotExistsError", flag(true))
            .prop(3, "BlackboardKey", used_key)
            .constant(1, "Bool", flag(true)),
    )?;
    let call: usize = a.add_node(&Spec::node(REQUEST, "ootrr enemy backup call").prop(1, "ExecuteEveryFrame", flag(false)).prop(1, "IsForeachActor", flag(false)).idle(0, "Index", vec![0; 4]).constant(3, "Key", key))?;
    let request: usize = a.add_node(&both("ootrr enemy backup request", call, latch))?;
    let wait: usize = a.add_node(&Spec::node(IDLE, "ootrr enemy backup idle wait"))?;
    let timer: usize = a.add_node(
        &Spec::node(TIMER, "ootrr enemy backup timer")
            .prop(1, "IsCountUpdateOnReset", flag(false))
            .prop(1, "IsLoop", flag(false))
            .prop(1, "NoChangeWhenChildIsBusy", flag(false))
            .prop(1, "SelectChildOnEnterOnly", flag(false))
            .prop(2, "CountFrames", FRAMES.to_le_bytes().to_vec())
            .prop(2, "RandomAddCountFrames", 0f32.to_le_bytes().to_vec())
            .spare(2, "CurrentFrame")
            .spare(2, "FinishFrame")
            .kid(wait, "NotFinish")
            .kid(request, "Finish"),
    )?;
    let spent: usize = a.add_node(&Spec::node(IDLE, "ootrr enemy backup idle spent"))?;
    let unused: usize = a.add_node(&switch("ootrr enemy backup unused", used, spent, timer))?;
    let kept: usize = a.add_node(&Spec::node(IDLE, "ootrr enemy backup idle kept"))?;
    let alone: usize = a.add_node(&switch("ootrr enemy backup alone", empty, unused, kept))?;
    let gate: usize = a.add_node(&switch("ootrr enemy backup gate", ready, wake, alone))?;
    let calm: usize = a.add_node(&Spec::node(IDLE, "ootrr enemy backup idle calm"))?;
    let battle: usize = a.add_node(&switch("ootrr enemy backup battle", fight, gate, calm))?;
    let off: usize = a.add_node(&Spec::node(IDLE, "ootrr enemy backup idle off"))?;
    let guard: usize = a.add_node(&switch("ootrr enemy backup guard", eligible, battle, off))?;
    let top: usize = a.add_node(&both("ootrr enemy backup root", host, guard))?;
    set_root(&mut a, top)?;
    let bad: Vec<String> = audit(&a);
    if !bad.is_empty() {
        return Err(Error::Msg(format!("enemy backup: {}", bad.join("; "))));
    }
    let out: Vec<u8> = a.write()?;
    let back: Ainb = Ainb::parse(&out)?;
    if back.write()? != out || back.params != a.params || back.ins != a.ins || back.outs != a.outs || back.qids != a.qids || root(&back) != Some(top) {
        return Err(Error::Format("enemy backup: the written graph does not read back"));
    }
    Ok(out)
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

pub fn actor_file(actor: &str) -> String {
    format!("{ACTOR_DIR}{actor}{ACTOR_EXT}")
}

fn packed(path: &str) -> Option<String> {
    let p: &str = path.strip_prefix('?').unwrap_or(path);
    if p.is_empty() {
        return None;
    }
    match p.strip_prefix("Work/") {
        Some(w) => Some(match w.strip_suffix(".gyml") {
            Some(stem) => format!("{stem}.bgyml"),
            None => w.to_string(),
        }),
        None => Some(p.to_string()),
    }
}

pub fn holder_of(es: &[Entry], actor: &str) -> Res<Option<String>> {
    let mut at: Option<String> = Some(actor_file(actor));
    for _ in 0..DEPTH {
        let Some(e) = at.as_deref().and_then(|f| sarc::get(es, f)) else {
            return Ok(None);
        };
        let y: Byml = Byml::parse(&e.data)?;
        if let Some(held) = y.root.get(PARTS).and_then(|c| c.get(SLOT)).and_then(Node::as_str) {
            return Ok(packed(held));
        }
        at = y.root.get(PARENT).and_then(Node::as_str).and_then(packed);
    }
    Ok(None)
}

pub fn entry(helper: &str) -> Node {
    Node::Map(vec![
        ("Actor".to_string(), Node::Str(format!("Work/Actor/{helper}.engine__actor__ActorParam.gyml"))),
        ("IsCreateFirstWithHolder".to_string(), Node::Bool(false)),
        ("IsEnableWaitCreateAndIgnition".to_string(), Node::Bool(false)),
        ("MaxActors".to_string(), Node::I32(MAX)),
    ])
}

pub fn holder_file(helper: &str) -> Res<Vec<u8>> {
    let y: Byml = Byml { root: Node::Map(vec![(HELD.to_string(), Node::Map(vec![(KEY.to_string(), entry(helper))]))]), version: VERSION, big: false };
    y.write()
}

fn put(raw: &[u8], group: &str, k: &str, v: Node) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    if !matches!(y.root, Node::Map(_)) {
        return Err(Error::Format("enemy backup: a parameter file is not a map"));
    }
    if !matches!(y.root.get(group), Some(Node::Map(_))) {
        y.root.set(group, Node::Map(Vec::new()));
    }
    let m: &mut Node = y.root.get_mut(group).ok_or(Error::Format("enemy backup: a parameter group is missing"))?;
    m.set(k, v.clone());
    let out: Vec<u8> = y.write()?;
    if Byml::parse(&out)?.root.get(group).and_then(|g| g.get(k)) != Some(&v) {
        return Err(Error::Format("enemy backup: a parameter file does not read back"));
    }
    Ok(out)
}

pub fn add_ref(raw: &[u8], file: &str) -> Res<Vec<u8>> {
    put(raw, PARTS, SLOT, Node::Str(format!("?{file}")))
}

pub fn add_key(raw: &[u8], helper: &str) -> Res<Vec<u8>> {
    put(raw, HELD, KEY, entry(helper))
}

pub fn excluded(actor: &str) -> bool {
    OUT_OF_BOUNDS.iter().any(|p| actor.starts_with(p))
}

pub fn helper_of(actor: &str) -> usize {
    if BUBBLE_CALLERS.contains(&actor) {
        1
    } else if POT_CALLERS.contains(&actor) {
        2
    } else {
        0
    }
}

pub fn flag_row(key: &str, start: bool) -> Node {
    Node::Map(vec![
        ("BBKey".to_string(), Node::Str(key.to_string())),
        ("InitVal".to_string(), Node::Bool(start)),
        ("IsInstanceParam".to_string(), Node::Bool(true)),
        ("IsRecord".to_string(), Node::Bool(false)),
        ("IsSafeParamForExternalTools".to_string(), Node::Bool(false)),
    ])
}

pub fn add_flags(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.get_mut(BOOLS).and_then(Node::as_arr_mut).ok_or(Error::Format("enemy backup: the common blackboard has no bool list"))?;
    if list.iter().any(|r| matches!(r.get("BBKey").and_then(Node::as_str), Some(ELIGIBLE | USED))) {
        return Err(Error::Format("enemy backup: the common blackboard already has the backup flags"));
    }
    list.push(flag_row(ELIGIBLE, true));
    list.push(flag_row(USED, false));
    let n: usize = list.len();
    let out: Vec<u8> = y.write()?;
    let back: Byml = Byml::parse(&out)?;
    let rows: &[Node] = back.root.get(BOOLS).and_then(Node::as_arr).ok_or(Error::Format("enemy backup: the common blackboard does not read back"))?;
    if rows.len() != n || rows.last() != Some(&flag_row(USED, false)) || back.root != y.root {
        return Err(Error::Format("enemy backup: the common blackboard does not read back"));
    }
    Ok(out)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub who: Set<String>,
    pub off: Set<String>,
    pub bare: Map<String, usize>,
    pub held: Map<String, usize>,
}

fn settle(m: &mut Map<String, usize>, file: String, helper: usize) {
    let at: &mut usize = m.entry(file).or_insert(helper);
    if *at != helper {
        *at = 0;
    }
}

impl Plan {
    pub fn note(&mut self, es: &[Entry], actor: &str) -> Res<()> {
        self.who.insert(actor.to_string());
        if !actor.starts_with(CALLERS) {
            return Ok(());
        }
        if excluded(actor) {
            self.off.insert(actor.to_string());
            return Ok(());
        }
        let helper: usize = helper_of(actor);
        match holder_of(es, actor)? {
            Some(h) => settle(&mut self.held, h, helper),
            None => settle(&mut self.bare, actor_file(actor), helper),
        };
        Ok(())
    }

    pub fn callers(&self) -> impl Iterator<Item = &String> {
        self.who.iter().filter(|a| a.starts_with(CALLERS) && !self.off.contains(*a))
    }

    pub fn wants(&self, name: &str) -> bool {
        name == STALFOS_GRAPH || name == TABLE || self.bare.contains_key(name) || self.held.contains_key(name)
    }
}

pub fn patch_entries(es: &mut Vec<Entry>, plan: &Plan, actor: &str) -> Res<Set<String>> {
    let mut hit: Set<String> = Set::new();
    let mut fresh: Set<usize> = Set::new();
    let off: bool = excluded(actor);
    for e in es.iter_mut() {
        let new: Vec<u8> = if GRAPHS.contains(&e.name.as_str()) {
            patch_graph(&e.data, &e.name)?
        } else if let Some(h) = plan.bare.get(&e.name) {
            fresh.insert(*h);
            add_ref(&e.data, HELPERS[*h].1)?
        } else if let Some(h) = plan.held.get(&e.name) {
            add_key(&e.data, HELPERS[*h].0)?
        } else if e.name == TABLE && !off && plan.callers().any(|c| c == actor) {
            add_flags(&e.data)?
        } else {
            continue;
        };
        e.data = new;
        hit.insert(e.name.clone());
    }
    for h in fresh {
        let (helper, file): (&str, &str) = HELPERS[h];
        if sarc::get(es, file).is_some() {
            return Err(Error::Format("enemy backup: the pack already has a backup holder"));
        }
        sarc::set(es, file, holder_file(helper)?);
        hit.insert(file.to_string());
    }
    Ok(hit)
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
    let list: &mut Vec<Node> = y.root.as_arr_mut().ok_or(Error::Format("enemy backup: the actor table is not a row list"))?;
    let mut want: Vec<(String, i32)> = Vec::new();
    for row in list.iter_mut() {
        let Some(name) = rows::id(row).filter(|n| actors.contains(*n)).map(str::to_string) else {
            continue;
        };
        let Some(old) = heap_of(row).filter(|v| *v > 0) else {
            continue;
        };
        let new: i32 = old.checked_add(HEAP_EXTRA).ok_or(Error::Format("enemy backup: the heap size does not fit"))?;
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
        return Err(Error::Format("enemy backup: the actor table does not read back"));
    }
    Ok(out)
}

fn zipped(rf: &Romfs, have: Have, path: &str) -> Res<Vec<u8>> {
    match have(path) {
        Some(b) => Ok(b),
        None => rf.read(path),
    }
}

fn opened(rf: &mut Romfs, have: Have, path: &str) -> Res<(Vec<u8>, usize, Vec<Entry>)> {
    let z: Vec<u8> = zipped(rf, have, path)?;
    let raw: Vec<u8> = rf.zs.decompress(&z)?;
    let es: Vec<Entry> = sarc::parse(&raw)?;
    Ok((z, raw.len(), es))
}

fn visit(rf: &mut Romfs, have: Have, path: &str, plan: &Plan, seen: &mut Set<String>) -> Res<Option<Out>> {
    let (z, old, mut es): (Vec<u8>, usize, Vec<Entry>) = opened(rf, have, path)?;
    let hit: Set<String> = patch_entries(&mut es, plan, actor_of(path).unwrap_or(""))?;
    if hit.is_empty() {
        return Ok(None);
    }
    seen.extend(hit);
    let out: Vec<u8> = sarc::write(&es)?;
    Ok(Some(Out { path: path.to_string(), old, new: out.len(), z: zs::compress_raw(&out, zs::dict_of(&z))? }))
}

pub fn stage(rf: &mut Romfs, have: Have, dir: &str) -> Res<(Plan, Vec<Out>)> {
    let shared: Set<String> = callers(&opened(rf, have, GLOBAL)?.2);
    let mut plan: Plan = Plan::default();
    let mut index: Vec<(String, Vec<String>)> = Vec::new();
    for f in rf.list(dir)? {
        let Some(name) = actor_of(&f).map(str::to_string) else {
            continue;
        };
        let es: Vec<Entry> = opened(rf, have, &f)?.2;
        if runs(&es, &shared) {
            plan.note(&es, &name)?;
        }
        let keep: Vec<String> = es.iter().filter(|e| e.name == STALFOS_GRAPH || e.name == TABLE || e.name.starts_with(HOLDER_DIR) || (e.name.starts_with(ACTOR_DIR) && e.name.ends_with(ACTOR_EXT))).map(|e| e.name.clone()).collect();
        if !keep.is_empty() {
            index.push((f, keep));
        }
    }
    if plan.who.is_empty() {
        return Err(Error::Format("enemy backup: no actor runs the damage color graph"));
    }
    let mut seen: Set<String> = Set::new();
    let mut outs: Vec<Out> = Vec::new();
    outs.push(visit(rf, have, GLOBAL, &plan, &mut seen)?.ok_or(Error::Msg(format!("enemy backup: the AI pack has no {GRAPH}")))?);
    for (f, keep) in &index {
        if keep.iter().any(|n| plan.wants(n)) {
            outs.extend(visit(rf, have, f, &plan, &mut seen)?);
        }
    }
    if let Some(lost) = plan.bare.keys().chain(plan.held.keys()).find(|n| !seen.contains(*n)) {
        return Err(Error::Msg(format!("enemy backup: {lost} is in no actor pack")));
    }
    outs.push(edit_zs(rf, have, ACTOR_INFO, |raw| grow_heap(raw, &plan.who))?);
    Ok((plan, outs))
}

pub fn size(rf: &mut Romfs, t: &mut Rst) -> Res<()> {
    let es: Vec<Entry> = sarc::parse(&rf.read_zs(PEER.0)?)?;
    let peer: &Entry = sarc::get(&es, PEER.1).ok_or(Error::Msg(format!("enemy backup: {} has no {}", PEER.0, PEER.1)))?;
    for (helper, file) in HELPERS {
        let n: u32 = sizes::file_size(t, &[(peer.name.clone(), peer.data.len())], file, holder_file(helper)?.len())?;
        t.set_size(file, n)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ainb::show::show;
    use std::collections::BTreeMap as Map;

    #[test]
    fn a_call_is_a_whole_name_in_the_string_pool() {
        let mut d: Vec<u8> = b"x\0".to_vec();
        d.extend_from_slice(MODULES[0].as_bytes());
        assert!(!calls(&d));
        d.push(0);
        assert!(calls(&d));
        assert_eq!(graph_of("Work/AI/Root/Enemy/Enemy_Stalfos.utility.root.ain").as_deref(), Some("AI/Enemy_Stalfos.utility.root.ainb"));
        assert_eq!(graph_of("Work/BSA/SchedulerParam/Enemy_Stalfos.game__bsa__SchedulerParam.gyml"), None);
        assert_eq!(actor_of("Pack/Actor/Enemy_Stalfos.pack.zs"), Some("Enemy_Stalfos"));
        assert_eq!(actor_of("Pack/Actor/readme.txt"), None);
    }

    fn file(name: &str, items: Vec<(&str, Node)>) -> Entry {
        let y: Byml = Byml { root: Node::Map(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect()), version: VERSION, big: false };
        Entry { name: name.to_string(), data: y.write().unwrap() }
    }

    fn parts(items: Vec<(&str, &str)>) -> Node {
        Node::Map(items.into_iter().map(|(k, v)| (k.to_string(), Node::Str(v.to_string()))).collect())
    }

    const D: &str = "Component/ActorHolderParam/Enemy_D.game__component__ActorHolderParam.bgyml";

    fn pack() -> Vec<Entry> {
        vec![
            file(&actor_file("Enemy_A"), vec![("Components", parts(vec![("ASRef", "?AS/a.asb"), ("LifeRef", "?Life/a.bgyml")]))]),
            file(&actor_file("Enemy_B"), vec![("$parent", Node::Str("Work/Actor/Enemy_A.engine__actor__ActorParam.gyml".to_string())), ("Components", parts(vec![("LifeRef", "?Life/b.bgyml")]))]),
            file(&actor_file("Enemy_C"), vec![("$parent", Node::Str("Work/Actor/Enemy_D.engine__actor__ActorParam.gyml".to_string()))]),
            file(&actor_file("Enemy_D"), vec![("Components", parts(vec![("ActorHolderRef", "?Component/ActorHolderParam/Enemy_D.game__component__ActorHolderParam.bgyml")]))]),
            file(&actor_file("Enemy_E"), vec![("$parent", Node::Str("Work/Actor/Enemy_D.engine__actor__ActorParam.gyml".to_string())), ("Components", parts(vec![("ActorHolderRef", "")]))]),
            file(D, vec![("ActorHoldSettings", Node::Map(vec![("Weapon".to_string(), Node::Map(vec![("MaxActors".to_string(), Node::I32(3))]))]))]),
        ]
    }

    #[test]
    fn a_holder_is_found_through_the_parents() {
        let es: Vec<Entry> = pack();
        assert_eq!(holder_of(&es, "Enemy_A").unwrap(), None);
        assert_eq!(holder_of(&es, "Enemy_B").unwrap(), None);
        assert_eq!(holder_of(&es, "Enemy_C").unwrap().as_deref(), Some(D));
        assert_eq!(holder_of(&es, "Enemy_D").unwrap().as_deref(), Some(D));
        assert_eq!(holder_of(&es, "Enemy_E").unwrap(), None, "an empty slot switches the parent's holder off");
        assert_eq!(holder_of(&es, "Enemy_Missing").unwrap(), None);
        assert_eq!(packed("Work/Actor/X.engine__actor__ActorParam.gyml").as_deref(), Some("Actor/X.engine__actor__ActorParam.bgyml"));
        assert_eq!(packed("?Component/X.bgyml").as_deref(), Some("Component/X.bgyml"));
        assert_eq!(packed(""), None);
    }

    #[test]
    fn a_pack_gains_the_call_slot_once_for_every_planned_actor() {
        let mut es: Vec<Entry> = pack();
        let mut plan: Plan = Plan::default();
        for a in ["Enemy_A", "Enemy_B", "Enemy_C", "Npc_D"] {
            plan.note(&es, a).unwrap();
        }
        assert_eq!((plan.who.len(), plan.callers().count(), plan.bare.len(), plan.held.len()), (4, 3, 2, 1));
        assert!(plan.wants(STALFOS_GRAPH) && plan.wants(&actor_file("Enemy_A")) && plan.wants(D));
        assert!(!plan.wants(&actor_file("Enemy_C")) && !plan.wants(GRAPH));
        let hit: Set<String> = patch_entries(&mut es, &plan, "Enemy_A").unwrap();
        assert_eq!(hit.len(), 4);
        for a in ["Enemy_A", "Enemy_B"] {
            assert_eq!(holder_of(&es, a).unwrap().as_deref(), Some(NEW));
        }
        let a: Byml = Byml::parse(&sarc::get(&es, &actor_file("Enemy_A")).unwrap().data).unwrap();
        let Some(Node::Map(m)) = a.root.get(PARTS) else { panic!() };
        assert_eq!(m.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), vec!["ASRef", SLOT, "LifeRef"]);
        let fresh: Byml = Byml::parse(&sarc::get(&es, NEW).unwrap().data).unwrap();
        assert_eq!(fresh.root.get(HELD).and_then(|h| h.get(KEY)), Some(&entry(HELPER)));
        let d: Byml = Byml::parse(&sarc::get(&es, D).unwrap().data).unwrap();
        let Some(Node::Map(h)) = d.root.get(HELD) else { panic!() };
        assert_eq!(h.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), vec![KEY, "Weapon"]);
        assert_eq!(h[0].1, entry(HELPER));
        assert!(patch_entries(&mut es, &plan, "Enemy_A").is_err(), "a patched pack is not patched twice");
        let mut other: Vec<Entry> = vec![file("Life/a.bgyml", vec![("MaxLife", Node::I32(1))])];
        assert!(patch_entries(&mut other, &plan, "Enemy_A").unwrap().is_empty());
        assert_eq!(other.len(), 1);
    }

    #[test]
    fn the_called_actor_is_made_on_demand_and_one_at_a_time() {
        let e: Node = entry(HELPER);
        assert_eq!(e.get("Actor").and_then(Node::as_str), Some("Work/Actor/Enemy_Peahat_Larva_Independent.engine__actor__ActorParam.gyml"));
        assert_eq!(e.get("IsCreateFirstWithHolder"), Some(&Node::Bool(false)));
        assert_eq!(e.get("MaxActors"), Some(&Node::I32(1)));
        assert_eq!(holder_file(HELPER).unwrap(), holder_file(HELPER).unwrap());
        assert_eq!(entry(HELPERS[2].0).get("Actor").and_then(Node::as_str), Some("Work/Actor/Enemy_FlyingPot.engine__actor__ActorParam.gyml"));
        assert!(HELPERS.iter().all(|h| h.1.starts_with(HOLDER_DIR)));
        assert!(NEW.starts_with(HOLDER_DIR));
    }

    #[test]
    fn bosses_and_their_kin_are_left_out_and_helpers_follow_the_caller() {
        for a in ["Enemy_DungeonBoss_Gohma", "Enemy_MiddleBoss_DarkLink", "Enemy_IronKnuckle_Black", "Enemy_DeadHand", "Enemy_DeadHand_Hand", "Enemy_PhantomGanonHorse", "Enemy_GhostPicturePerformer_PhantomGanon", "Enemy_Guay_Boss"] {
            assert!(excluded(a), "{a}");
        }
        for a in ["Enemy_Stalfos", "Enemy_Keese", "Enemy_Dodongo", "Enemy_Guay"] {
            assert!(!excluded(a), "{a}");
        }
        assert_eq!((helper_of("Enemy_Keese"), helper_of("Enemy_Octorok"), helper_of("Enemy_Biri"), helper_of("Enemy_Freezard"), helper_of("Enemy_Stalfos")), (0, 0, 1, 2, 0));
        let mut m: Map<String, usize> = Map::new();
        settle(&mut m, "f".to_string(), 2);
        settle(&mut m, "f".to_string(), 2);
        assert_eq!(m["f"], 2);
        settle(&mut m, "f".to_string(), 1);
        assert_eq!(m["f"], 0, "a holder shared by callers that want different helpers falls back to the larva");
        let mut es: Vec<Entry> = pack();
        let mut plan: Plan = Plan::default();
        plan.note(&es, "Enemy_MiddleBoss_X").unwrap();
        plan.note(&es, "Enemy_A").unwrap();
        assert_eq!((plan.who.len(), plan.off.len(), plan.callers().count(), plan.bare.len()), (2, 1, 1, 1));
        let table: Entry = Entry { name: TABLE.to_string(), data: Byml { root: Node::Map(vec![(BOOLS.to_string(), Node::Arr(vec![flag_row("IsX", false)]))]), version: VERSION, big: false }.write().unwrap() };
        es.push(table);
        let mut boss: Vec<Entry> = es.clone();
        assert!(!patch_entries(&mut boss, &plan, "Enemy_MiddleBoss_X").unwrap().contains(TABLE));
        assert!(patch_entries(&mut es, &plan, "Enemy_A").unwrap().contains(TABLE));
        let rows: Vec<Node> = Byml::parse(&sarc::get(&es, TABLE).unwrap().data).unwrap().root.get(BOOLS).and_then(Node::as_arr).unwrap().to_vec();
        assert_eq!(rows, vec![flag_row("IsX", false), flag_row(ELIGIBLE, true), flag_row(USED, false)]);
        assert!(add_flags(&sarc::get(&es, TABLE).unwrap().data).is_err());
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

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        if p.is_dir() { Some(p) } else { None }
    }

    fn graph(rf: &mut Romfs, pack: &str, name: &str) -> Ainb {
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(pack).unwrap()).unwrap();
        Ainb::parse(&sarc::get(&es, name).unwrap().data).unwrap()
    }

    fn lines(a: &Ainb, i: usize) -> Vec<String> {
        let all: String = show(a);
        let from: usize = all.find(&format!("element {i} type")).unwrap();
        let body: &str = &all[from..];
        let end: usize = body.find(&format!("\nelement {} type", i + 1)).unwrap_or(body.len());
        body[..end].lines().skip(1).map(str::to_string).collect()
    }

    fn field(l: &str) -> String {
        l.split(" = ").next().unwrap_or(l).to_string()
    }

    fn shape(a: &Ainb, i: usize) -> Vec<String> {
        lines(a, i)
            .into_iter()
            .filter(|l| !l.contains("plug "))
            .map(|l| {
                let wired: &str = if l.contains(" from ") { " wired" } else { "" };
                let tail: &str = if l.trim_start().starts_with("in ") { l.rsplit(" flags ").next().unwrap_or("") } else { "" };
                format!("{}{wired} {tail}", field(&l))
            })
            .collect()
    }

    fn fields(a: &Ainb, i: usize) -> Vec<String> {
        lines(a, i).into_iter().filter(|l| !l.contains("plug ")).map(|l| field(&l)).collect()
    }

    fn plugs(a: &Ainb, i: usize) -> (Vec<(String, usize)>, Vec<[u8; 2]>) {
        (a.params[i].plugs.iter().map(|g| (a.s(g.b).to_string(), g.extra.len())).collect(), a.params[i].pl.to_vec())
    }

    fn one(a: &Ainb, name: &str) -> usize {
        a.find_els(name)[0]
    }

    fn a_once(a: &Ainb, i: usize) -> bool {
        lines(a, i).iter().any(|l| l.contains("ExecuteOnce = true"))
    }

    fn classes(a: &Ainb, i: usize) -> Vec<String> {
        let p: &crate::ainb::Param = &a.params[i];
        let word = |b: &[u8]| -> String { a.s(u32::from_le_bytes([b[4], b[5], b[6], b[7]])).to_string() };
        let ins = (p.ioe[5][0] as usize..(p.ioe[5][0] + p.ioe[5][1]) as usize).map(|g| word(&a.ins[5][g * 20..]));
        let outs = (p.ioe[5][2] as usize..(p.ioe[5][2] + p.ioe[5][3]) as usize).map(|g| word(&a.outs[5][g * 8..]));
        ins.chain(outs).collect()
    }

    #[test]
    fn a_graph_without_the_damage_color_node_is_refused() {
        assert!(patch_graph(b"nope", GRAPH).is_err());
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(GLOBAL).unwrap()).unwrap();
        assert!(patch_graph(&sarc::get(&es, "AI/EnemyCommon.common.guardbreakomen.module.ainb").unwrap().data, GRAPH).is_err());
    }

    #[test]
    fn the_game_graph_gains_a_battle_timer_that_calls_and_wakes_one_helper() {
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
        assert_eq!(a.els.len(), old.els.len() + 24);
        let top: usize = root(&a).unwrap();
        assert_eq!((a.el_type(top), a.role(top) & 4, a.role(0) & 4), (PARALLEL, 4, 0));
        let kids = |i: usize| -> Vec<usize> { a.params[i].group(2).iter().map(|g| g.a as usize).collect() };
        let guard: usize = kids(top)[1];
        assert_eq!((a.el_name(kids(top)[0]), a.el_type(guard)), (HOST, SWITCH));
        let battle: usize = kids(guard)[0];
        assert_eq!((a.el_type(battle), a.el_name(kids(guard)[1])), (SWITCH, IDLE));
        let gate: usize = kids(battle)[0];
        assert_eq!((a.el_type(gate), a.el_name(kids(battle)[1])), (SWITCH, IDLE));
        let alone: usize = kids(gate)[1];
        assert_eq!((a.el_name(kids(gate)[0]), a.el_type(alone)), (WAKE, SWITCH));
        let unused: usize = kids(alone)[0];
        assert_eq!((a.el_type(unused), a.el_name(kids(alone)[1])), (SWITCH, IDLE));
        let timer: usize = kids(unused)[1];
        assert_eq!((a.el_name(kids(unused)[0]), a.el_name(timer)), (IDLE, TIMER));
        let request: usize = kids(timer)[1];
        assert_eq!((a.el_name(kids(timer)[0]), a.el_type(request)), (IDLE, PARALLEL));
        assert_eq!((a.el_name(kids(request)[0]), a.el_name(kids(request)[1])), (REQUEST, SETB));
        let source = |i: usize| -> usize { a.input(1, a.input_ids(i, 1)[0]).unwrap().src as usize };
        let reads = |i: usize| -> String { a.el_name(source(i)).to_string() };
        assert_eq!((reads(guard), reads(unused), reads(gate), reads(alone), reads(battle)), (GETB.to_string(), GETB.to_string(), READY.to_string(), EMPTY.to_string(), FIGHT.to_string()));
        assert_eq!((a.prop_str(source(guard), "BlackboardKey"), a.prop_str(source(unused), "BlackboardKey"), a.prop_str(kids(request)[1], "BlackboardKey")), (Some(ELIGIBLE), Some(USED), Some(USED)));
        for n in [READY, EMPTY, WAKE, REQUEST] {
            assert_eq!(a.in_str(one(&a, n), "Key"), Some(KEY), "{n}");
        }
        let shown: String = show(&a);
        for line in ["prop bool ExecuteOnce = true", "prop bool IsIgnoreNotExistsError = true", "in bool Bool = true flags 0x800000", "prop bool IsLoop = false", "prop f32 CountFrames = 750", "in vec3 InputB = (0, 1.5, 0) flags 0x800000", "\"NotFinish\"", "\"Finish\""] {
            assert!(shown.contains(line), "{line} is missing from\n{shown}");
        }
        assert!(patch_graph(&out, GRAPH).is_err(), "a patched graph is not patched twice");
    }

    #[test]
    fn the_game_nodes_have_the_shapes_the_patch_copies() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let mine: Ainb = Ainb::parse(&patch_graph(&graph(&mut rf, GLOBAL, GRAPH).write().unwrap(), GRAPH).unwrap()).unwrap();
        let lure: Ainb = graph(&mut rf, "Pack/Actor/Weapon_Lure.pack.zs", "AI/Weapon_Lure.root.ainb");
        let tektite: Ainb = graph(&mut rf, "Pack/Actor/Enemy_Tektite_Blue.pack.zs", "AI/Enemy_Tektite.action.interruptboomerang.module.ainb");
        let set: usize = tektite.find_els(SETB).into_iter().find(|i| a_once(&tektite, *i)).unwrap();
        assert_eq!(shape(&mine, one(&mine, SETB)), shape(&tektite, set));
        assert_eq!(plugs(&mine, one(&mine, SETB)), plugs(&tektite, set));
        assert_eq!(classes(&mine, one(&mine, SETB)), classes(&tektite, set));
        assert_eq!(tektite.prop_str(set, "BlackboardKey"), Some("IsStunLightEnable"));
        let amos: Ainb = graph(&mut rf, "Pack/Actor/Enemy_Amos.pack.zs", "AI/Amos.brain.inaccessibletotarget.module.ainb");
        let get: usize = amos.find_els(GETB)[0];
        assert_eq!(mine.find_els(GETB).len(), 2);
        for g in mine.find_els(GETB) {
            assert_eq!(shape(&mine, g), shape(&amos, get));
            assert_eq!(plugs(&mine, g), plugs(&amos, get));
            assert_eq!(classes(&mine, g), classes(&amos, get));
        }
        for n in [READY, EMPTY, REQUEST, PLACE] {
            assert_eq!(shape(&mine, one(&mine, n)), shape(&lure, one(&lure, n)), "{n}");
            assert_eq!(plugs(&mine, one(&mine, n)), plugs(&lure, one(&lure, n)), "{n}");
            assert_eq!(classes(&mine, one(&mine, n)), classes(&lure, one(&lure, n)), "{n}");
        }
        let dodongo: Ainb = graph(&mut rf, "Pack/Actor/Enemy_Dodongo.pack.zs", "AI/Enemy_Dodongo.utility.root.ainb");
        for n in [WAKE, SPLIT] {
            assert_eq!(shape(&mine, one(&mine, n)), shape(&dodongo, one(&dodongo, n)), "{n}");
            assert_eq!(plugs(&mine, one(&mine, n)), plugs(&dodongo, one(&dodongo, n)), "{n}");
            assert_eq!(classes(&mine, one(&mine, n)), classes(&dodongo, one(&dodongo, n)), "{n}");
        }
        let gohma: Ainb = graph(&mut rf, "Pack/Actor/Enemy_DungeonBoss_Gohma.pack.zs", "AI/DungeonBossGohma.brain.hangingbattle.module.ainb");
        assert_eq!(shape(&mine, one(&mine, FIGHT)), shape(&gohma, one(&gohma, FIGHT)));
        let bari: Ainb = graph(&mut rf, "Pack/Actor/Enemy_Bari.pack.zs", "AI/Bari.action.interruptdie.module.ainb");
        assert_eq!(fields(&mine, one(&mine, LIFT)), fields(&bari, one(&bari, LIFT)));
        assert_eq!(plugs(&mine, one(&mine, LIFT)), plugs(&bari, one(&bari, LIFT)));
        let twin: Ainb = graph(&mut rf, "Pack/Actor/Enemy_DungeonBoss_Twinrova.pack.zs", "AI/Enemy_DungeonBoss_Twinrova.utility.root.ainb");
        let theirs: usize = twin.find_els(TIMER).into_iter().find(|i| shape(&twin, *i).iter().any(|l| l.contains("IsLoop"))).unwrap();
        assert_eq!(shape(&mine, one(&mine, TIMER)), shape(&twin, theirs));
        let glass: Ainb = graph(&mut rf, GLOBAL, "AI/EnemyCommon.common.enableglasstruth.module.ainb");
        let sw: usize = (0..glass.els.len()).find(|i| glass.el_type(*i) == SWITCH).unwrap();
        let switches: Vec<usize> = (0..mine.els.len()).filter(|i| mine.el_type(*i) == SWITCH).collect();
        assert_eq!(switches.len(), 5);
        for i in switches {
            assert_eq!(shape(&mine, i), shape(&glass, sw));
            assert_eq!(plugs(&mine, i), plugs(&glass, sw));
        }
        let base: Ainb = graph(&mut rf, GLOBAL, "AI/EnemyBase.utility.root.ainb");
        let par: usize = (0..base.els.len()).find(|i| base.el_type(*i) == PARALLEL).unwrap();
        assert_eq!(shape(&mine, root(&mine).unwrap()), shape(&base, par));
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    #[test]
    fn the_game_actors_that_run_the_graph_can_all_call_the_helper() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let (plan, outs): (Plan, Vec<Out>) = stage(&mut rf, &none, "Pack/Actor").unwrap();
        for a in ["Enemy_Stalfos", "Enemy_Stalfos_ForChild", "Enemy_Keese", "Enemy_Octorok", "Enemy_Poe", "Enemy_Dodongo", "Enemy_DungeonBoss_Gohma"] {
            assert!(plan.who.contains(a), "{a} runs the graph");
        }
        for a in [HELPERS[0].0, HELPERS[1].0, HELPERS[2].0, "Enemy_Peahat_Larva", "Player_Child", "Player_Adult", "Npc_Navi", "Enemy_Beamos", "Enemy_Stalfos_Dictionary"] {
            assert!(!plan.who.contains(a), "{a} does not run the graph");
        }
        assert!(plan.bare.contains_key(&actor_file("Enemy_Keese")) && !plan.bare.contains_key(&actor_file("Enemy_Poe")));
        assert!(plan.held.contains_key("Component/ActorHolderParam/Enemy_Poe.game__component__ActorHolderParam.bgyml"));
        let last: usize = outs.len() - 1;
        assert_eq!((outs[0].path.as_str(), outs[last].path.as_str()), (GLOBAL, ACTOR_INFO));
        assert_eq!(zs::dict_of(&outs[0].z), zs::dict_of(&rf.read(GLOBAL).unwrap()), "the AI pack keeps its dictionary");
        let mut copies: Map<String, Vec<u8>> = Map::new();
        let mut packs: Map<String, Vec<Entry>> = Map::new();
        for x in &outs[1..last] {
            assert!(x.new > x.old, "{} grew", x.path);
            let now: Vec<Entry> = sarc::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap();
            let old: Vec<Entry> = sarc::parse(&rf.read_zs(&x.path).unwrap()).unwrap();
            for e in &now {
                match sarc::get(&old, &e.name) {
                    Some(o) if o.data == e.data => continue,
                    Some(_) => assert!(plan.wants(&e.name), "{} changed in {}", e.name, x.path),
                    None => assert!(HELPERS.iter().any(|h| h.1 == e.name), "{}", x.path),
                }
                let first: &Vec<u8> = copies.entry(e.name.clone()).or_insert_with(|| e.data.clone());
                assert_eq!(first, &e.data, "every copy of {} is the same", e.name);
            }
            assert_eq!(now.len(), old.len() + HELPERS.iter().filter(|h| sarc::get(&now, h.1).is_some() && sarc::get(&old, h.1).is_none()).count(), "{}", x.path);
            packs.insert(actor_of(&x.path).unwrap().to_string(), now);
        }
        for a in ["Npc_SkullDad", "Obj_InsideJabu_Jabu_SwayingFloor_A_01"] {
            assert!(plan.who.contains(a) && !packs.contains_key(a), "{a} runs the graph and keeps its files");
        }
        assert!(plan.held.keys().all(|h| h.starts_with("Component/ActorHolderParam/Enemy_")), "{:?}", plan.held);
        for a in plan.callers() {
            let es: &Vec<Entry> = packs.get(a).unwrap_or_else(|| panic!("the pack of {a} was not staged"));
            let h: String = holder_of(es, a).unwrap().unwrap_or_else(|| panic!("{a} has no holder"));
            let y: Byml = Byml::parse(&sarc::get(es, &h).unwrap_or_else(|| panic!("{a} lacks {h}")).data).unwrap();
            let want: usize = plan.held.get(&h).or(plan.bare.get(&actor_file(a))).copied().unwrap_or_else(|| panic!("{a} has no helper"));
            assert_eq!(y.root.get(HELD).and_then(|m| m.get(KEY)), Some(&entry(HELPERS[want].0)), "{a} through {h}");
            assert!(want == helper_of(a) || want == 0, "{a} gets its group helper or the larva");
            let flags: Byml = Byml::parse(&sarc::get(es, TABLE).unwrap_or_else(|| panic!("{a} has no common blackboard")).data).unwrap();
            let rows: &[Node] = flags.root.get(BOOLS).and_then(Node::as_arr).unwrap();
            assert_eq!(&rows[rows.len() - 2..], &[flag_row(ELIGIBLE, true), flag_row(USED, false)], "{a}");
        }
        assert!(plan.off.len() >= 30, "{:?}", plan.off);
        for a in &plan.off {
            assert!(excluded(a) && !plan.callers().any(|c| c == a), "{a}");
            if let Some(es) = packs.get(a) {
                let has: bool = sarc::get(es, TABLE).is_some_and(|t| t.data.windows(ELIGIBLE.len()).any(|w| w == ELIGIBLE.as_bytes()));
                assert!(!has, "{a} must not carry the eligible flag");
            }
        }
        for a in BUBBLE_CALLERS.iter().chain(POT_CALLERS.iter()) {
            assert!(plan.callers().any(|c| c == a), "{a} is a caller");
        }
        for (helper, _) in HELPERS {
            assert!(!plan.who.contains(helper), "{helper} does not run the shared graph");
            assert!(rf.read(&format!("Pack/Actor/{helper}.pack.zs")).is_ok(), "{helper} has a pack");
        }
        let peer: Byml = Byml::parse(&sarc::get(&sarc::parse(&rf.read_zs(PEER.0).unwrap()).unwrap(), PEER.1).unwrap().data).unwrap();
        assert_eq!((peer.version, peer.big), (VERSION, false));
        assert!(rf.read(&format!("Pack/Actor/{HELPER}.pack.zs")).is_ok(), "the helper has a pack");
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
            assert!(plan.who.contains(name), "{name} changed");
            assert_eq!(heap_of(y), Some(heap_of(x).unwrap() + HEAP_EXTRA));
            grown += 1;
        }
        assert!(grown >= 80, "{grown} actors grew");
        let mut t: Rst = crate::rsizetable::load(&mut rf).unwrap();
        let before: Vec<Option<u32>> = GRAPHS.iter().map(|g| t.get(g)).collect();
        let stalfos: &str = "Actor/Enemy_Stalfos.engine__actor__ActorParam.bgyml";
        let param: Option<u32> = t.get(stalfos);
        assert!(before.iter().all(|v| v.is_some()) && param.is_some() && HELPERS.iter().all(|h| t.get(h.1).is_none()));
        for o in &outs {
            crate::staging::grow_out(&mut rf, &mut t, o).unwrap();
        }
        for (g, was) in GRAPHS.iter().zip(before) {
            assert!(t.get(g) > was, "the size table entry of {g} grew");
        }
        assert!(t.get(stalfos) > param);
        size(&mut rf, &mut t).unwrap();
        for (helper, file) in HELPERS {
            assert!(t.get(file).is_some_and(|n| n as usize > holder_file(helper).unwrap().len()), "{file}");
        }
        println!("{} actors, {} callers, {} off, {} bare, {} held, {} files", plan.who.len(), plan.callers().count(), plan.off.len(), plan.bare.len(), plan.held.len(), outs.len());
    }

    #[test]
    fn the_vanilla_damage_color_node_is_untouched_after_regen_fairy_and_backup() {
        let Some(dir) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&dir).unwrap();
        let es: Vec<Entry> = sarc::parse(&rf.read_zs(GLOBAL).unwrap()).unwrap();
        let raw: Vec<u8> = sarc::get(&es, GRAPH).unwrap().data.clone();
        let old: Ainb = Ainb::parse(&raw).unwrap();
        assert_eq!((old.els.len(), root(&old), old.el_name(0)), (1, Some(0), HOST));
        let vanilla: String = show(&old);
        let element: &str = vanilla.split("element 0 ").nth(1).unwrap();
        let first: Vec<u8> = super::super::regen::patch_graph(&raw, GRAPH).unwrap();
        let second: Vec<u8> = super::super::fairy::patch_graph(&first, GRAPH).unwrap();
        let third: Vec<u8> = patch_graph(&second, GRAPH).unwrap();
        for out in [&first, &second, &third] {
            let a: Ainb = Ainb::parse(out).unwrap();
            let text: String = show(&a);
            assert_eq!(a.find_els(HOST), vec![0]);
            assert!(text.contains(&format!("element 0 {element}").split("element 1 ").next().unwrap().trim_end().to_string()));
            let mut at: usize = 0;
            while Some(at) != root(&a) {
                let (parent, _): (usize, usize) = a.parent_of(at).unwrap();
                assert_eq!((a.el_type(parent), a.params[parent].group(2).first().map(|g| g.a as usize)), (3, Some(at)));
                at = parent;
            }
            assert_eq!(a.params[0].group(2).len(), 0);
            assert_eq!(a.find_els(HOST).len(), 1);
        }
    }
}
