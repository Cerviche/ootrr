use std::collections::{BTreeMap as BM, BTreeSet as BS};
use std::fs;
use std::path::Path as P;

use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Entry, Flow, Kind};
use crate::byml::{Byml, Node};
use crate::dungeon_credit::{action_slot, idx, push, talk, TALK, TALKER};
use crate::dungeon_text::{FILE, FILE_ID};
use crate::error::{Error as E, Res};
use crate::msbt::{self, Msbt};
use crate::placement::hash_of;
use crate::progressive::{add_rows, Size, ANALYZED_TABLE, SETTING_TABLE};
use crate::romfs::Romfs;
use crate::sarc;
use crate::slice::data::Data;
use crate::staging::Out;
use crate::story_skip::{stage_late, stage_late_file};

pub const NUM: u16 = 136;
pub const SETTING: &str = "debug_location_names";
pub const NOTE: &str = "Debug: every chest shows its location name in a text box after its item get.";
const PREFIX: &str = "OOTRR_Dbg_";
const ENTRY_PREFIX: &str = "Dbg_";
const LINE_PREFIX: &str = "OOTRR_DbgLoc_";
const CHEST: &str = "Obj_TBox";
const EVENT: &str = "GetEventName";
const DROP: &str = "Drop__DropActor";
const CHART: &str = "FlowchartFileName";
const ENTRY: &str = "EntryPointName";
const ROW_ID: &str = "__RowId";

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

fn current(rf: &mut Romfs, more: &[Out], work: &P, path: &str) -> Res<Vec<u8>> {
    if let Some(o) = more.iter().find(|o| o.path == path) {
        return rf.zs.decompress(&o.z);
    }
    let at = path.split('/').fold(work.to_path_buf(), |a, s| a.join(s));
    match fs::read(&at) {
        Ok(z) => rf.zs.decompress(&z),
        Err(_) => rf.read_zs(path),
    }
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

fn has_row(y: &Byml, id: &str) -> bool {
    y.root.as_arr().is_some_and(|a| a.iter().any(|r| text(r, ROW_ID) == id))
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
        let (vars, pad) = {
            let e: &Entry = f.entries.iter().find(|e| e.name == r.entry).ok_or_else(|| E::Msg(format!("debug names: {stem} has no entry {}", r.entry)))?;
            (e.vars.clone(), e.pad)
        };
        let (t, say): (u16, u16) = action_slot(f, TALKER, TALK)?;
        let at: u16 = idx(f.events.len())?;
        push(f, Kind::Sub { next: at + 1, params: None, flow: String::new(), entry: r.entry.clone() })?;
        push(f, Kind::Action { next: NONE, actor: t, act: say, params: Some(talk(line_id(w.loc))) })?;
        f.entries.push(Entry { name: w.entry.clone(), subs: vec![at], vars, main: at, pad });
    }
    Ok(())
}

pub fn put_lines(raw: &[u8], lines: &BM<usize, String>) -> Res<Vec<u8>> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let e: &sarc::Entry = sarc::get(&es, FILE).ok_or_else(|| E::Msg(format!("the language archive has no {FILE}")))?;
    let mut m: Msbt = msbt::parse(&e.data)?;
    for (i, name) in lines {
        let t = msbt::from_markup(&format!("[{name}]"))?;
        m.put_tokens(&line_label(*i), &t)?;
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
    let lines: BM<usize, String> = wraps.iter().map(|w| (w.loc, d.cat.locs[w.loc].name.clone())).collect();
    for l in crate::hints::languages(rf)? {
        stage_late_file(rf, more, work, &l, |raw: &[u8]| put_lines(raw, &lines))?;
    }
    Ok(found.values().map(|v| v.len()).sum())
}
