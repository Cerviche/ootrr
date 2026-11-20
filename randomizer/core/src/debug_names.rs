use std::collections::{BTreeMap as BM, BTreeSet as BS};
use std::fs;
use std::path::Path as P;

use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Cont, Entry, Flow, Kind, Val};
use crate::byml::{Byml, Node};
use crate::dungeon_credit::{action_slot, idx, push, set_bool, talk, HOLDER, SET, TALK, TALKER};
use crate::dungeon_text::{FILE, FILE_ID};
use crate::error::{Error as E, Res};
use crate::msbt::{self, Msbt};
use crate::placement::hash_of;
use crate::progressive::{add_children, add_rows, Size, ANALYZED_TABLE, SETTING_TABLE};
use crate::romfs::Romfs;
use crate::sarc;
use crate::slice::data::Data;
use crate::staging::Out;
use crate::story_skip::{stage_late, stage_late_file};

pub const NUM: u16 = 136;
pub const SETTING: &str = "debug_location_names";
pub const NOTE: &str = "Debug: chests, NPC, song and event rewards, Gold Skulltulas and items on the ground show their location name in a text box after the item get.";
const PREFIX: &str = "OOTRR_Dbg_";
const ENTRY_PREFIX: &str = "Dbg_";
const LINE_PREFIX: &str = "OOTRR_DbgLoc_";
const FLOW_PREFIX: &str = "OOTRR_DbgFlow_";
const CHEST: &str = "Obj_TBox";
const EVENT: &str = "GetEventName";
const DROP: &str = "Drop__DropActor";
const CHART: &str = "FlowchartFileName";
const ENTRY: &str = "EntryPointName";
const ROW_ID: &str = "__RowId";
const KIDS: &str = "ChildEventList";
const GET_FLOW: &str = "GetPouchContent";
const WINDOW_PREFIX: &str = "OpenGetWindow";
const SWEEP: &str = "OOTRRDbgSweep";
const SKULL_KIND: &str = "gs";
const ASK_BOOL: &str = "EventQueryIsGameDataBool";
const SCRUB_FLOW: &str = "Enemy_Scrub_Business_Common";
pub const PICK: &str = "OOTRRDbgPick";
const PICK_TEMPLATE: &str = "UI_MessageTargetActorName";
const ASK_TEXT: &str = "EventQueryIsGameDataString64";
const SET_TEXT: &str = "EventTriggerSetGameDataString64";
const ADD_OWN: &str = "EventTriggerAddOwnToPouch";
const OWN_LINK: &str = "QueryGenericGetOwnActorLink";
const ACTOR_NAME: &str = "QueryGenericGetActorName";
const SET_STRING: &str = "OneShotSetNonStructGameDataString64";
const GLOBAL_AI: &str = "Pack/AI.Global.Product.111.pack.zs";
const PICKUP_KINDS: [&str; 4] = ["freestanding", "heart", "bossheart", "key"];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wrap {
    pub loc: usize,
    pub old: String,
    pub id: String,
    pub entry: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub stem: String,
    pub entry: String,
}

pub fn line_label(loc: usize) -> String {
    format!("{LINE_PREFIX}{loc}")
}

pub fn line_id(loc: usize) -> String {
    format!("{FILE_ID}:{}", line_label(loc))
}

pub fn flow_label(loc: usize) -> String {
    format!("{FLOW_PREFIX}{loc}")
}

pub fn flow_id(loc: usize) -> String {
    format!("{FILE_ID}:{}", flow_label(loc))
}

pub fn scene_line(d: &Data, loc: &str) -> Option<String> {
    d.flow.locs.iter().position(|l| l.name == loc).map(flow_id)
}

fn current(rf: &mut Romfs, more: &[Out], work: &P, path: &str) -> Res<Vec<u8>> {
    if let Some(o) = more.iter().find(|o| o.path == path) {
        return rf.zs.decompress(&o.z);
    }
    let at = path.split('/').fold(work.to_path_buf(), |a, s| a.join(s));
    let out: Res<Vec<u8>> = match fs::read(&at) {
        Ok(z) => rf.zs.decompress(&z),
        Err(_) => rf.read_zs(path),
    };
    out.map_err(|e| E::Msg(format!("debug names: {path}: {e}")))
}

fn text<'a>(n: &'a Node, k: &str) -> &'a str {
    n.get(k).and_then(|v| v.as_str()).unwrap_or("")
}

fn actors(y: &mut Byml) -> Option<&mut Vec<Node>> {
    y.root.get_mut("Actors")?.as_arr_mut()
}

pub fn chest_event(a: &Node) -> Option<String> {
    let g: &str = text(a, "Gyaml");
    if !g.starts_with(CHEST) {
        return None;
    }
    let d: &Node = a.get("Dynamic")?;
    d.get(DROP)?;
    let ev: &str = d.get(EVENT).and_then(|v| v.as_str()).unwrap_or("");
    Some(if ev.is_empty() { Size::of_gyaml(g).template().to_string() } else { ev.to_string() })
}

pub fn row_of(y: &Byml, id: &str) -> Option<Row> {
    let r: &Node = y.root.as_arr()?.iter().find(|r| text(r, ROW_ID) == id)?;
    let chart: &str = text(r, CHART);
    let stem: &str = chart.rsplit('/').next()?.strip_suffix(".evfc")?;
    Some(Row { stem: stem.to_string(), entry: text(r, ENTRY).to_string() })
}

pub fn parents(y: &Byml, ws: &[Wrap]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let Some(rows) = y.root.as_arr() else { return out };
    for r in rows {
        let Some(kids) = r.get(KIDS).and_then(|k| k.as_arr()) else { continue };
        for w in ws {
            if kids.iter().any(|k| k.as_str() == Some(w.old.as_str())) {
                out.push((text(r, ROW_ID).to_string(), w.id.clone()));
            }
        }
    }
    out
}

fn has_row(y: &Byml, id: &str) -> bool {
    y.root.as_arr().is_some_and(|a| a.iter().any(|r| text(r, ROW_ID) == id))
}

fn next_of(k: &Kind) -> Vec<u16> {
    match k {
        Kind::Action { next, .. } | Kind::Join { next } | Kind::Sub { next, .. } => vec![*next],
        Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter().map(|c| c.ev).collect(),
        Kind::Fork { join, forks } => forks.iter().copied().chain([*join]).collect(),
        Kind::Label { .. } => Vec::new(),
    }
}

pub fn args_from(f: &Flow, main: u16) -> BS<String> {
    let mut out: BS<String> = BS::new();
    let mut seen: BS<u16> = BS::new();
    let mut todo: Vec<u16> = vec![main];
    while let Some(i) = todo.pop() {
        if i == NONE || !seen.insert(i) {
            continue;
        }
        let Some(e) = f.events.get(usize::from(i)) else { continue };
        if let Some(c) = e.kind.params() {
            for (_, v) in &c.items {
                if let Val::Arg(a) = v {
                    out.insert(a.clone());
                }
            }
        }
        todo.extend(next_of(&e.kind));
    }
    out
}

pub fn wrap_flow(b: &mut Bf, stem: &str, ws: &[(&Wrap, &Row)]) -> Res<()> {
    let f: &mut Flow = match b.flows.iter().position(|f| f.name == stem) {
        Some(i) => &mut b.flows[i],
        None => b.flows.first_mut().ok_or_else(|| E::Msg(format!("debug names: {stem} has no flow")))?,
    };
    for (w, r) in ws {
        if f.entries.iter().any(|e| e.name == w.entry) {
            continue;
        }
        let (vars, pad, main) = {
            let e: &Entry = f.entries.iter().find(|e| e.name == r.entry).ok_or_else(|| E::Msg(format!("debug names: {stem} has no entry {}", r.entry)))?;
            (e.vars.clone(), e.pad, e.main)
        };
        let args: BS<String> = args_from(f, main);
        let params: Option<Cont> = if args.is_empty() { None } else { Some(Cont { items: args.iter().map(|a| (a.clone(), Val::Arg(a.clone()))).collect() }) };
        let (t, say): (u16, u16) = action_slot(f, TALKER, TALK)?;
        let at: u16 = idx(f.events.len())?;
        push(f, Kind::Sub { next: at + 1, params, flow: String::new(), entry: r.entry.clone() })?;
        push(f, Kind::Action { next: NONE, actor: t, act: say, params: Some(talk(line_id(w.loc))) })?;
        f.entries.push(Entry { name: w.entry.clone(), subs: vec![at], vars, main: at, pad });
    }
    Ok(())
}

pub fn put_lines(raw: &[u8], lines: &BM<String, String>) -> Res<Vec<u8>> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let e: &sarc::Entry = sarc::get(&es, FILE).ok_or_else(|| E::Msg(format!("the language archive has no {FILE}")))?;
    let mut m: Msbt = msbt::parse(&e.data)?;
    for (label, name) in lines {
        let t = msbt::from_markup(&format!("[{name}]"))?;
        m.put_tokens(label, &t)?;
    }
    sarc::set(&mut es, FILE, m.write()?);
    sarc::write(&es)
}

pub fn stage(rf: &mut Romfs, d: &Data, work: &P, more: &mut Vec<Out>) -> Res<usize> {
    let mut by_file: BM<String, BM<u64, usize>> = BM::new();
    for (i, l) in d.cat.locs.iter().enumerate() {
        for r in &l.refs {
            for f in &r.files {
                by_file.entry(f.clone()).or_default().insert(r.hash, i);
            }
        }
    }
    let mut found: BM<String, Vec<(u64, usize, String)>> = BM::new();
    for (f, want) in &by_file {
        let raw: Vec<u8> = current(rf, more, work, f)?;
        let mut y: Byml = Byml::parse(&raw)?;
        let Some(list) = actors(&mut y) else { continue };
        for a in list.iter() {
            let Some(h) = hash_of(a) else { continue };
            let Some(i) = want.get(&h) else { continue };
            if let Some(ev) = chest_event(a) {
                found.entry(f.clone()).or_default().push((h, *i, ev));
            }
        }
    }
    let pairs: BS<(usize, String)> = found.values().flatten().map(|(_, i, ev)| (*i, ev.clone())).collect();
    if pairs.is_empty() {
        return Ok(0);
    }
    let wraps: Vec<Wrap> = pairs.iter().enumerate().map(|(n, (i, ev))| Wrap { loc: *i, old: ev.clone(), id: format!("{PREFIX}{n}"), entry: format!("{ENTRY_PREFIX}{n}") }).collect();
    let id_of: BM<(usize, String), String> = wraps.iter().map(|w| ((w.loc, w.old.clone()), w.id.clone())).collect();
    let table: Byml = Byml::parse(&current(rf, more, work, SETTING_TABLE)?)?;
    let mut rows: Vec<(&Wrap, Row)> = Vec::new();
    for w in &wraps {
        let r: Row = row_of(&table, &w.old).ok_or_else(|| E::Msg(format!("debug names: the event table has no {} row for {}", w.old, d.cat.locs[w.loc].name)))?;
        rows.push((w, r));
    }
    let mut by_flow: BM<String, Vec<(&Wrap, &Row)>> = BM::new();
    for (w, r) in &rows {
        by_flow.entry(r.stem.clone()).or_default().push((w, r));
    }
    for (stem, ws) in &by_flow {
        stage_late(rf, more, work, stem, |b| wrap_flow(b, stem, ws))?;
    }
    for t in [SETTING_TABLE, ANALYZED_TABLE] {
        let analyzed: bool = t == ANALYZED_TABLE;
        stage_late_file(rf, more, work, t, |raw: &[u8]| {
            let mut y: Byml = Byml::parse(raw)?;
            for w in &wraps {
                if analyzed && !has_row(&y, &w.old) {
                    continue;
                }
                add_rows(&mut y, &w.old, &[(w.id.clone(), if analyzed { None } else { Some(w.entry.clone()) })])?;
            }
            if analyzed {
                let kids: Vec<(String, String)> = parents(&y, &wraps);
                add_children(&mut y, &kids);
            }
            y.write()
        })?;
    }
    for (f, hits) in &found {
        stage_late_file(rf, more, work, f, |raw: &[u8]| {
            let mut y: Byml = Byml::parse(raw)?;
            if let Some(list) = actors(&mut y) {
                for a in list.iter_mut() {
                    let Some(h) = hash_of(a) else { continue };
                    let Some((_, i, ev)) = hits.iter().find(|x| x.0 == h) else { continue };
                    let Some(id) = id_of.get(&(*i, ev.clone())) else { continue };
                    if let Some(dy) = a.get_mut("Dynamic") {
                        dy.set(EVENT, Node::Str(id.clone()));
                    }
                }
            }
            y.write()
        })?;
    }
    let lines: BM<String, String> = wraps.iter().map(|w| (line_label(w.loc), d.cat.locs[w.loc].name.clone())).collect();
    for l in crate::hints::languages(rf)? {
        stage_late_file(rf, more, work, &l, |raw: &[u8]| put_lines(raw, &lines))?;
    }
    Ok(found.values().map(|v| v.len()).sum())
}

fn say_after(f: &mut Flow, at: usize, id: String) -> Res<bool> {
    let then: u16 = match &f.events[at].kind {
        Kind::Action { next, .. } | Kind::Sub { next, .. } => *next,
        _ => return Ok(false),
    };
    let (t, say): (u16, u16) = action_slot(f, TALKER, TALK)?;
    let n: u16 = push(f, Kind::Action { next: then, actor: t, act: say, params: Some(talk(id)) })?;
    match &mut f.events[at].kind {
        Kind::Action { next, .. } | Kind::Sub { next, .. } => *next = n,
        _ => {}
    }
    Ok(true)
}

fn say_before(f: &mut Flow, at: usize, id: String) -> Res<()> {
    let (t, say): (u16, u16) = action_slot(f, TALKER, TALK)?;
    let w: u16 = idx(at)?;
    let n: u16 = push(f, Kind::Action { next: w, actor: t, act: say, params: Some(talk(id)) })?;
    for (i, ev) in f.events.iter_mut().enumerate() {
        if i == usize::from(n) {
            continue;
        }
        match &mut ev.kind {
            Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } if *next == w => *next = n,
            Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => {
                for c in cases.iter_mut().filter(|c| c.ev == w) {
                    c.ev = n;
                }
            }
            Kind::Fork { forks, join } => {
                for x in forks.iter_mut().filter(|x| **x == w) {
                    *x = n;
                }
                if *join == w {
                    *join = n;
                }
            }
            _ => {}
        }
    }
    for e in f.entries.iter_mut() {
        if e.main == w {
            e.main = n;
        }
    }
    Ok(())
}

pub fn hook(b: &mut Bf, event: &str, id: &str) -> Res<bool> {
    for f in b.flows.iter_mut() {
        let Some(at) = f.events.iter().position(|e| e.name == event) else { continue };
        if !say_after(f, at, id.to_string())? {
            say_before(f, at, id.to_string())?;
        }
        return Ok(true);
    }
    Ok(false)
}

pub fn stage_flows(rf: &mut Romfs, d: &Data, work: &P, late: &mut Vec<Out>, scenes: &BS<&str>, sales: &[(String, String)]) -> Res<Vec<String>> {
    let mut missed: Vec<String> = Vec::new();
    let mut by_file: BM<String, Vec<(String, String)>> = BM::new();
    for (i, l) in d.flow.locs.iter().enumerate() {
        if scenes.contains(l.name.as_str()) {
            continue;
        }
        if sales.iter().any(|(_, loc)| loc == &l.name) {
            continue;
        }
        let mut grants: Vec<&crate::flow_locations::Edit> = l.edits.iter().filter(|e| e.role == "grant").collect();
        if grants.is_empty() {
            grants = l.edits.iter().filter(|e| e.role == "offer").collect();
        }
        if grants.is_empty() {
            missed.push(l.name.clone());
            continue;
        }
        for e in grants {
            for f in std::iter::once(&e.file).chain(e.also.iter()) {
                let stem: String = f.trim_end_matches(".bfevfl").to_string();
                by_file.entry(stem).or_default().push((e.event.clone(), flow_id(i)));
            }
        }
    }
    for (stem, hs) in &by_file {
        let path: String = format!("Event/EventFlow/{stem}.bfevfl.zs");
        let staged: bool = late.iter().any(|o| o.path == path) || path.split('/').fold(work.to_path_buf(), |a, s| a.join(s)).is_file();
        if !staged && rf.read(&path).is_err() {
            missed.extend(hs.iter().map(|(ev, _)| format!("{stem}:{ev}")));
            continue;
        }
        let mut found: Vec<bool> = Vec::new();
        stage_late(rf, late, work, stem, |b| {
            for (ev, id) in hs {
                found.push(hook(b, ev, id)?);
            }
            Ok(())
        })?;
        for ((ev, _), ok) in hs.iter().zip(found) {
            if !ok {
                missed.push(format!("{stem}:{ev}"));
            }
        }
    }
    if !sales.is_empty() {
        let mut found: Vec<bool> = Vec::new();
        stage_late(rf, late, work, SCRUB_FLOW, |b| {
            for (entry, loc) in sales {
                let i: Option<usize> = d.flow.locs.iter().position(|l| &l.name == loc);
                found.push(match i {
                    Some(i) => hook_sale(b, entry, &flow_id(i))?,
                    None => false,
                });
            }
            Ok(())
        })?;
        for ((_, loc), ok) in sales.iter().zip(found) {
            if !ok {
                missed.push(loc.clone());
            }
        }
    }
    let lines: BM<String, String> = d.flow.locs.iter().enumerate().map(|(i, l)| (flow_label(i), l.name.clone())).collect();
    for l in crate::hints::languages(rf)? {
        stage_late_file(rf, late, work, &l, |raw: &[u8]| put_lines(raw, &lines))?;
    }
    Ok(missed)
}

pub fn skull_flag(h: u64) -> String {
    format!("IsGoldSkullDead.IsGoldSkullDead_{h}")
}

pub fn shown_flag(h: u64) -> String {
    format!("OOTRRDbgGs_{h}")
}

pub fn skulls(d: &Data) -> Vec<(u64, usize)> {
    let mut out: Vec<(u64, usize)> = Vec::new();
    for (i, l) in d.cat.locs.iter().enumerate().filter(|(_, l)| l.kind == SKULL_KIND) {
        for r in &l.refs {
            if !out.iter().any(|x| x.0 == r.hash) {
                out.push((r.hash, i));
            }
        }
    }
    out
}

fn row_hash(r: &Node) -> Option<u32> {
    match r.get("Hash") {
        Some(Node::U32(h)) => Some(*h),
        _ => None,
    }
}

pub fn add_shown(raw: &[u8], gs: &[(u64, usize)]) -> Res<(Vec<u8>, Vec<u64>)> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.get_mut("Data").and_then(|d| d.get_mut("Bool")).and_then(|b| b.as_arr_mut()).ok_or(E::Format("debug names: the game data list has no Bool table"))?;
    let have: BS<u32> = list.iter().filter_map(row_hash).collect();
    let mut added: Vec<u64> = Vec::new();
    for (h, _) in gs {
        let k: u32 = crate::flags::hash_name(&skull_flag(*h));
        let Some(base) = list.iter().find(|r| row_hash(r) == Some(k)).cloned() else { continue };
        let n: u32 = crate::flags::hash_name(&shown_flag(*h));
        if have.contains(&n) {
            return Err(E::Msg(format!("debug names: the flag {} clashes with a game flag", shown_flag(*h))));
        }
        let mut r: Node = base;
        r.set("Hash", Node::U32(n));
        r.set("DefaultValue", Node::Bool(false));
        list.push(r);
        added.push(*h);
    }
    Ok((y.write()?, added))
}

fn ask(f: &mut Flow, name: &str, yes: u16, no: u16) -> Res<Kind> {
    let (a, q): (u16, u16) = crate::start_state::query_slot(f, "GameDataActor", ASK_BOOL)?;
    Ok(Kind::Switch {
        actor: a,
        query: q,
        cases: vec![Case { val: 0, ev: yes }, Case { val: 1, ev: no }],
        params: Some(Cont { items: vec![("GameDataName".to_string(), Val::Str(name.to_string())), ("Index".to_string(), Val::Int(-1))] }),
    })
}

pub fn add_sweep(b: &mut Bf, gs: &[(u64, usize)], picks: &[(String, usize)]) -> Res<usize> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == GET_FLOW).ok_or(E::Format("debug names: GetPouchContent has no flow"))?;
    if f.entries.iter().any(|e| e.name == SWEEP) || (gs.is_empty() && picks.is_empty()) {
        return Ok(0);
    }
    let pad: u16 = f.entries.first().map(|e| e.pad).unwrap_or(0);
    let base: usize = f.events.len();
    let (ta, ts): (u16, u16) = action_slot(f, TALKER, TALK)?;
    let (sa, ss): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let tail: usize = base + 4 * gs.len() + 2 * picks.len();
    for (k, (h, i)) in gs.iter().enumerate() {
        let at: usize = base + 4 * k;
        let after: u16 = idx(at + 4)?;
        let dead: Kind = ask(f, &skull_flag(*h), idx(at + 1)?, after)?;
        let seen: Kind = ask(f, &shown_flag(*h), after, idx(at + 2)?)?;
        push(f, dead)?;
        push(f, seen)?;
        push(f, Kind::Action { next: idx(at + 3)?, actor: ta, act: ts, params: Some(talk(line_id(*i))) })?;
        push(f, Kind::Action { next: after, actor: sa, act: ss, params: Some(set_bool(shown_flag(*h))) })?;
    }
    let (qa, qt): (u16, u16) = crate::start_state::query_slot(f, HOLDER, ASK_TEXT)?;
    let (_, st): (u16, u16) = action_slot(f, HOLDER, SET_TEXT)?;
    for (k, (name, i)) in picks.iter().enumerate() {
        let at: usize = base + 4 * gs.len() + 2 * k;
        let after: u16 = idx(at + 2)?;
        let items: Vec<(String, Val)> = vec![("GameDataName".to_string(), Val::Str(PICK.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(name.clone()))];
        push(f, Kind::Switch { actor: qa, query: qt, cases: vec![Case { val: 0, ev: idx(at + 1)? }, Case { val: 1, ev: after }], params: Some(Cont { items }) })?;
        push(f, Kind::Action { next: after, actor: ta, act: ts, params: Some(talk(line_id(*i))) })?;
    }
    let clear: Vec<(String, Val)> = vec![("GameDataName".to_string(), Val::Str(PICK.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(String::new()))];
    if push(f, Kind::Action { next: NONE, actor: qa, act: st, params: Some(Cont { items: clear }) })? != idx(tail)? {
        return Err(E::Format("debug names: the sweep events are out of order"));
    }
    f.entries.push(Entry { name: SWEEP.to_string(), subs: Vec::new(), vars: Vec::new(), main: idx(base)?, pad });
    let starts: Vec<u16> = f.entries.iter().filter(|e| e.name.starts_with(WINDOW_PREFIX)).map(|e| e.main).collect();
    let mut seen: BS<u16> = BS::new();
    let mut todo: Vec<u16> = starts;
    let mut ends: Vec<u16> = Vec::new();
    while let Some(i) = todo.pop() {
        if i == NONE || usize::from(i) >= base || !seen.insert(i) {
            continue;
        }
        let k: &Kind = &f.events[usize::from(i)].kind;
        let nx: Vec<u16> = next_of(k);
        if matches!(k, Kind::Action { .. } | Kind::Sub { .. } | Kind::Join { .. } | Kind::Switch { .. } | Kind::SubSwitch { .. }) && nx.contains(&NONE) {
            ends.push(i);
        }
        todo.extend(nx);
    }
    let mut n: usize = 0;
    for i in ends {
        let call: u16 = push(f, Kind::Sub { next: NONE, params: None, flow: String::new(), entry: SWEEP.to_string() })?;
        let owners: Vec<usize> = f.entries.iter().enumerate().filter(|(_, e)| e.name.starts_with(WINDOW_PREFIX)).map(|(j, _)| j).collect();
        for j in owners {
            if !f.entries[j].subs.contains(&call) {
                f.entries[j].subs.push(call);
            }
        }
        match &mut f.events[usize::from(i)].kind {
            Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => *next = call,
            Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => {
                for c in cases.iter_mut().filter(|c| c.ev == NONE) {
                    c.ev = call;
                }
            }
            _ => {}
        }
        n += 1;
    }
    Ok(n)
}

pub fn add_pick_flag(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.get_mut("Data").and_then(|d| d.get_mut("String64")).and_then(|b| b.as_arr_mut()).ok_or(E::Format("debug names: the game data list has no String64 table"))?;
    let k: u32 = crate::flags::hash_name(PICK_TEMPLATE);
    let n: u32 = crate::flags::hash_name(PICK);
    if !list.iter().any(|r| row_hash(r) == Some(n)) {
        let mut r: Node = list.iter().find(|r| row_hash(r) == Some(k)).cloned().ok_or(E::Format("debug names: the game data list has no UI_MessageTargetActorName row"))?;
        r.set("Hash", Node::U32(n));
        r.set("DefaultValue", Node::Str(String::new()));
        list.push(r);
    }
    y.write()
}

fn ptr_class(a: &crate::ainb::Ainb, el: usize) -> Option<String> {
    let st: usize = a.params.get(el)?.ioe[5][2] as usize;
    let rec: &[u8] = a.outs[5].get(st * 8..st * 8 + 8)?;
    let o: u32 = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]);
    a.pool.get(o).map(|x| x.to_string())
}

pub fn patch_pick(raw: &[u8], tag: &str) -> Res<Option<Vec<u8>>> {
    use crate::ainb::build::Spec;
    use crate::ainb::{audit, Ainb, Plug};
    let mut a: Ainb = Ainb::parse(raw)?;
    let sites: Vec<usize> = a.find_els(ADD_OWN);
    if sites.is_empty() || a.find_els(SET_STRING).iter().any(|&i| a.in_str(i, "GameDataFullName") == Some(PICK)) {
        return Ok(None);
    }
    let class: String = a.find_els(OWN_LINK).into_iter().find_map(|i| ptr_class(&a, i)).ok_or_else(|| E::Msg(format!("debug names: {tag} has no actor link to copy")))?;
    for (n, site) in sites.into_iter().enumerate() {
        let kid: usize = {
            let p = &a.params[site];
            p.plugs.get(p.pl[2][1] as usize).map(|x| x.a as usize).ok_or_else(|| E::Msg(format!("debug names: the add handler in {tag} has no body")))?
        };
        if a.el_type(kid) != 3 {
            return Err(E::Msg(format!("debug names: the add handler in {tag} is not a parallel")));
        }
        let seed: String = format!("ootrr debug pick {tag} {n}");
        let s1: String = format!("{seed} link");
        let s2: String = format!("{seed} name");
        let s3: String = format!("{seed} set");
        let link: usize = a.add_node(&Spec::query(OWN_LINK, &s1).out_ptr("Actor", &class))?;
        let name: usize = a.add_node(&Spec::query(ACTOR_NAME, &s2).linked_ptr("Actor", &class, link).spare(3, "ActorFileName").out(3, "ActorInstanceName").spare(3, "ActorName"))?;
        let key: Vec<u8> = a.pool.add(PICK).to_le_bytes().to_vec();
        let set: usize = a.add_node(
            &Spec::node(SET_STRING, &s3)
                .prop(1, "ExecuteEveryFrame", vec![0; 4])
                .constant(0, "GameDataIndex", (-1i32).to_le_bytes().to_vec())
                .constant(3, "GameDataFullName", key)
                .linked(3, "Value", name, 1),
        )?;
        let at: usize = a.params[kid].pl[2][0] as usize;
        let nm: u32 = a.pool.add("");
        a.params[kid].add_plug(2, at, Plug { a: set as u32, b: nm, extra: Vec::new() })?;
    }
    let bad: Vec<String> = audit(&a);
    if !bad.is_empty() {
        return Err(E::Msg(format!("debug names: {tag}: {}", bad.join("; "))));
    }
    let out: Vec<u8> = a.write()?;
    let back: Ainb = Ainb::parse(&out)?;
    if back.write()? != out {
        return Err(E::Msg(format!("debug names: {tag} does not read back")));
    }
    Ok(Some(out))
}

pub fn patch_pack(raw: &[u8], tag: &str) -> Res<Option<Vec<u8>>> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let mut hit: bool = false;
    for e in es.iter_mut().filter(|e| e.name.ends_with(".event.root.ainb")) {
        if let Some(out) = patch_pick(&e.data, &format!("{tag} {}", e.name))? {
            e.data = out;
            hit = true;
        }
    }
    if hit {
        Ok(Some(sarc::write(&es)?))
    } else {
        Ok(None)
    }
}

fn edit_pack(rf: &mut Romfs, tail: &mut Vec<Out>, work: &P, path: &str, f: impl FnOnce(&[u8]) -> Res<Vec<u8>>) -> Res<()> {
    let at = path.split('/').fold(work.to_path_buf(), |a, s| a.join(s));
    let z: Vec<u8> = match tail.iter().find(|o| o.path == path) {
        Some(o) => o.z.clone(),
        None => match fs::read(&at) {
            Ok(z) => z,
            Err(_) => rf.read(path)?,
        },
    };
    let id: Option<u32> = crate::zs::dict_of(&z);
    let raw: Vec<u8> = rf.zs.decompress(&z)?;
    let out: Vec<u8> = f(&raw)?;
    let o: Out = Out { path: path.to_string(), z: crate::zs::compress_raw(&out, id)?, old: raw.len(), new: out.len() };
    match tail.iter().position(|x| x.path == path) {
        Some(i) => tail[i] = o,
        None => tail.push(o),
    }
    Ok(())
}

pub fn pick_name(h: u64) -> String {
    format!("OOTRRDbg{h}")
}

pub fn stage_sweep(rf: &mut Romfs, d: &Data, work: &P, tail: &mut Vec<Out>) -> Res<(usize, usize)> {
    let gs: Vec<(u64, usize)> = skulls(d);
    let mut by_file: BM<String, BM<u64, usize>> = BM::new();
    for (i, l) in d.cat.locs.iter().enumerate().filter(|(_, l)| PICKUP_KINDS.contains(&l.kind.as_str())) {
        for r in &l.refs {
            for f in &r.files {
                by_file.entry(f.clone()).or_default().insert(r.hash, i);
            }
        }
    }
    let mut picks: BM<String, usize> = BM::new();
    let mut seen: BS<String> = BS::new();
    for (f, want) in &by_file {
        let raw: Vec<u8> = current(rf, tail, work, f)?;
        let y: Byml = Byml::parse(&raw)?;
        let Some(list) = y.root.get("Actors").and_then(|x| x.as_arr()) else { continue };
        let mut names: BM<u64, String> = BM::new();
        for a in list {
            let Some(h) = hash_of(a) else { continue };
            let Some(i) = want.get(&h) else { continue };
            let g: &str = text(a, "Gyaml");
            if g.starts_with(CHEST) {
                continue;
            }
            let name: String = match a.get("Name").and_then(|v| v.as_str()) {
                Some(n) if !n.is_empty() => n.to_string(),
                _ => pick_name(h),
            };
            seen.insert(g.to_string());
            picks.insert(name.clone(), *i);
            names.insert(h, name);
        }
        if names.is_empty() {
            continue;
        }
        stage_late_file(rf, tail, work, f, |raw: &[u8]| {
            let mut y: Byml = Byml::parse(raw)?;
            if let Some(list) = actors(&mut y) {
                for a in list.iter_mut() {
                    let Some(h) = hash_of(a) else { continue };
                    if let Some(n) = names.get(&h)
                        && a.get("Name").is_none()
                    {
                        a.set("Name", Node::Str(n.clone()));
                    }
                }
            }
            y.write()
        })?;
    }
    let mut packs: Vec<String> = vec![GLOBAL_AI.to_string()];
    packs.extend(seen.iter().map(|g| crate::pickup_pack::pack_of(g)));
    for p in &packs {
        let Ok(raw) = current(rf, tail, work, p) else { continue };
        if patch_pack(&raw, p)?.is_none() {
            continue;
        }
        edit_pack(rf, tail, work, p, |raw: &[u8]| Ok(patch_pack(raw, p)?.unwrap_or_else(|| raw.to_vec())))?;
    }
    let mut added: Vec<u64> = Vec::new();
    stage_late_file(rf, tail, work, crate::chest_slot::LIST, |raw: &[u8]| {
        let (b, a) = add_shown(raw, &gs)?;
        added = a;
        add_pick_flag(&b)
    })?;
    let live: Vec<(u64, usize)> = gs.into_iter().filter(|x| added.contains(&x.0)).collect();
    let pl: Vec<(String, usize)> = picks.into_iter().collect();
    stage_late(rf, tail, work, GET_FLOW, |b| add_sweep(b, &live, &pl).map(|_| ()))?;
    let lines: BM<String, String> = live.iter().map(|(_, i)| *i).chain(pl.iter().map(|(_, i)| *i)).map(|i| (line_label(i), d.cat.locs[i].name.clone())).collect();
    for l in crate::hints::languages(rf)? {
        stage_late_file(rf, tail, work, &l, |raw: &[u8]| put_lines(raw, &lines))?;
    }
    Ok((live.len(), pl.len()))
}

fn case_to(k: &Kind, val: u32) -> Option<u16> {
    match k {
        Kind::Switch { cases, .. } => cases.iter().find(|c| c.val == val).map(|c| c.ev),
        _ => None,
    }
}

pub fn hook_sale(b: &mut Bf, entry: &str, id: &str) -> Res<bool> {
    let Some(f) = b.flows.iter_mut().find(|f| f.name == SCRUB_FLOW) else { return Ok(false) };
    let Some(gate) = f.entries.iter().find(|e| e.name == entry).map(|e| e.main) else { return Ok(false) };
    let Some(sub) = case_to(&f.events[usize::from(gate)].kind, 1) else { return Ok(false) };
    let chk: u16 = match &f.events[usize::from(sub)].kind {
        Kind::Sub { next, .. } => *next,
        _ => return Ok(false),
    };
    let Some(set) = case_to(&f.events[usize::from(chk)].kind, 0) else { return Ok(false) };
    say_after(f, usize::from(set), id.to_string())
}
