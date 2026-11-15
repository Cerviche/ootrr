use std::collections::BTreeSet as BS;

use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Cont, Entry, Event, Flow, Kind, Val};
use crate::dungeon_credit::{action_slot, sound};
use crate::error::{Error as E, Res};
use crate::progressive::tiers;

pub const FLOW: &str = "Event/EventFlow/DisplayShop.bfevfl.zs";
pub const ENTRY: &str = "OOTRR_ShelfHook";
pub const NAME: &str = "Shop_SelectItemName";
pub const GRANTS: [(usize, &str, u16); 2] = [(156, "Event281", 277), (461, "Event543", 430)];

const GRANT: &str = "EventTriggerGetItemAtShop";
const STARTER: &str = "Npc_EventStarter";
const DATA: &str = "GameDataActor";
const ASK: &str = "EventQueryIsGameDataString64";
const POUCH: &str = "PouchSystemActor";
const ADD: &str = "EventTriggerAddOrRemovePouchContent";
const UI: &str = "UISystemActor";
const WAITER: &str = "EventSystemActor";
const WAIT: &str = "EventWait";
const LIMIT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Add(&'static str, i32),
    Ui(&'static str, Option<(&'static str, bool)>),
    Wait(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hook {
    pub family: &'static str,
    pub shelf: &'static str,
    pub name: &'static str,
    pub steps: &'static [Step],
}

pub const HOOKS: [Hook; 4] = [
    Hook {
        family: "Bow",
        shelf: "Item_Range_Bow",
        name: "Weapon_Range_Bow",
        steps: &[Step::Add("Item_Quiver_Small", 1), Step::Add("Item_Arrow_Normal", 30), Step::Add("Item_Quiver_Large", 1)],
    },
    Hook {
        family: "Slingshot",
        shelf: "Item_Range_SlingShot",
        name: "Weapon_Range_SlingShot",
        steps: &[Step::Add("Item_BulletBag_Small", 1), Step::Add("Item_DekuSeed", 30), Step::Add("Item_BulletBag_Large", 1)],
    },
    Hook { family: "BombBag", shelf: "Item_BombBag_Small", name: "Item_BombBag_Small", steps: &[Step::Add("Item_Bomb", 20), Step::Add("Item_BombBag_Large", 1)] },
    Hook {
        family: "Magic",
        shelf: "Item_FairyBless_MP",
        name: "Item_FairyBless_MP",
        steps: &[
            Step::Add("Item_FairyBless_MPDouble", 1),
            Step::Ui("EventTriggerReserveBornMPGaugeDouble", None),
            Step::Ui("EventTriggerSetDisplayHeartGauge", Some(("IsDisplayHeartGauge", true))),
            Step::Wait(100),
            Step::Ui("EventBornMPGaugeDouble", Some(("IsWaitEnd", true))),
            Step::Wait(30),
        ],
    },
];

pub fn by_family(f: &str) -> Option<&'static Hook> {
    HOOKS.iter().find(|h| h.family == f)
}

pub fn by_shelf(item: &str) -> Option<&'static Hook> {
    HOOKS.iter().find(|h| h.shelf == item)
}

pub fn by_name(name: &str) -> Option<&'static Hook> {
    HOOKS.iter().find(|h| h.name == name)
}

pub fn shelf_item(actor: &str) -> &str {
    HOOKS.iter().find(|h| tiers(h.family).is_ok_and(|t| t.contains(&actor))).map(|h| h.shelf).unwrap_or(actor)
}

fn idx(n: usize) -> Res<u16> {
    u16::try_from(n).map_err(|_| E::Format("shelf hook: index does not fit in 16 bits"))
}

fn b(v: bool) -> Val {
    Val::Bool(u32::from(v))
}

fn add_cont(actor: &str, n: i32) -> Cont {
    Cont {
        items: vec![
            ("Num".to_string(), Val::Int(n)),
            ("IsEquipDIrect".to_string(), b(false)),
            ("IsSetMessageTargetActorName".to_string(), b(false)),
            ("Actor".to_string(), Val::Str(actor.to_string())),
            ("IsGetEvent".to_string(), b(false)),
        ],
    }
}

fn ask_cont(name: &str) -> Cont {
    Cont { items: vec![("GameDataName".to_string(), Val::Str(NAME.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(name.to_string()))] }
}

fn wait_cont(n: i32) -> Cont {
    Cont { items: vec![("IsWaitEnd".to_string(), b(true)), ("Frames".to_string(), Val::Int(n))] }
}

fn push(f: &mut Flow, kind: Kind) -> Res<u16> {
    let i: u16 = idx(f.events.len())?;
    f.events.push(Event { name: format!("OOTRR_Shelf{i}"), kind });
    Ok(i)
}

fn step_kind(f: &mut Flow, s: &Step, next: u16) -> Res<Kind> {
    Ok(match s {
        Step::Add(a, n) => {
            let (actor, act): (u16, u16) = action_slot(f, POUCH, ADD)?;
            Kind::Action { next, actor, act, params: Some(add_cont(a, *n)) }
        }
        Step::Ui(name, p) => {
            let (actor, act): (u16, u16) = action_slot(f, UI, name)?;
            Kind::Action { next, actor, act, params: p.map(|(k, v)| Cont { items: vec![(k.to_string(), b(v))] }) }
        }
        Step::Wait(n) => {
            let (actor, act): (u16, u16) = action_slot(f, WAITER, WAIT)?;
            Kind::Action { next, actor, act, params: Some(wait_cont(*n)) }
        }
    })
}

fn query_slot(f: &mut Flow, who: &str, q: &str) -> Res<(u16, u16)> {
    let i: usize = f.actors.iter().position(|a| a.name == who && a.sub.is_empty()).ok_or_else(|| E::Msg(format!("shelf hook: the shop flow has no {who}, the game files differ")))?;
    let j: usize = match f.actors[i].queries.iter().position(|x| x == q) {
        Some(j) => j,
        None => {
            f.actors[i].queries.push(q.to_string());
            f.actors[i].queries.len() - 1
        }
    };
    Ok((idx(i)?, idx(j)?))
}

fn targets(k: &Kind) -> Vec<u16> {
    match k {
        Kind::Action { next, .. } | Kind::Join { next } | Kind::Sub { next, .. } => vec![*next],
        Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter().map(|c| c.ev).collect(),
        Kind::Fork { join, forks } => forks.iter().copied().chain([*join]).collect(),
        Kind::Label { .. } => Vec::new(),
    }
}

fn reach(f: &Flow, from: u16) -> BS<u16> {
    let mut seen: BS<u16> = BS::new();
    let mut todo: Vec<u16> = vec![from];
    while let Some(i) = todo.pop() {
        if i == NONE || !seen.insert(i) {
            continue;
        }
        if let Some(e) = f.events.get(i as usize) {
            todo.extend(targets(&e.kind));
        }
    }
    seen
}

fn action_name<'a>(f: &'a Flow, actor: u16, act: u16) -> Option<(&'a str, &'a str)> {
    let a: &crate::bfevfl::Actor = f.actors.get(actor as usize)?;
    Some((a.name.as_str(), a.actions.get(act as usize)?.as_str()))
}

fn grant_ok(f: &Flow, i: usize, name: &str) -> Res<u16> {
    let e: &Event = f.events.get(i).ok_or_else(|| E::Msg(format!("shelf hook: the shop flow has no event {i}, the game files differ")))?;
    let Kind::Action { next, actor, act, .. } = &e.kind else {
        return Err(E::Msg(format!("shelf hook: event {i} is not an action, the game files differ")));
    };
    if e.name != name || action_name(f, *actor, *act) != Some((STARTER, GRANT)) {
        return Err(E::Msg(format!("shelf hook: event {i} is not the shop grant {name}, the game files differ")));
    }
    Ok(*next)
}

pub fn apply(f: &mut Flow, hooks: &[&Hook]) -> Res<()> {
    if hooks.is_empty() {
        return Err(E::Format("shelf hook: no item needs the hook"));
    }
    if f.entries.iter().any(|e| e.name == ENTRY) {
        return Err(E::Format("shelf hook: the shop flow already holds the hook"));
    }
    for (i, name, next) in GRANTS {
        if grant_ok(f, i, name)? != next {
            return Err(E::Msg(format!("shelf hook: the grant {name} does not lead to event {next}, the game files differ")));
        }
    }
    let pad: u16 = f.entries.first().map(|e| e.pad).ok_or(E::Format("shelf hook: the shop flow has no entries"))?;
    let owners: Vec<Vec<usize>> = GRANTS.iter().map(|(g, _, _)| f.entries.iter().enumerate().filter(|(_, e)| reach(f, e.main).contains(&(*g as u16))).map(|(k, _)| k).collect()).collect();
    let (da, dq): (u16, u16) = query_slot(f, DATA, ASK)?;
    let mut rungs: Vec<u16> = Vec::new();
    for h in hooks {
        rungs.push(push(f, Kind::Switch { actor: da, query: dq, cases: Vec::new(), params: Some(ask_cont(h.name)) })?);
    }
    for (k, h) in hooks.iter().enumerate() {
        let first: u16 = idx(f.events.len())?;
        for (j, s) in h.steps.iter().enumerate() {
            let next: u16 = if j + 1 == h.steps.len() { NONE } else { first + idx(j + 1)? };
            let kind: Kind = step_kind(f, s, next)?;
            push(f, kind)?;
        }
        let mut cases: Vec<Case> = vec![Case { val: 0, ev: first }];
        if let Some(n) = rungs.get(k + 1) {
            cases.push(Case { val: 1, ev: *n });
        }
        if let Kind::Switch { cases: c, .. } = &mut f.events[rungs[k] as usize].kind {
            *c = cases;
        }
    }
    for ((g, _, next), who) in GRANTS.iter().zip(&owners) {
        let s: u16 = push(f, Kind::Sub { next: *next, params: None, flow: String::new(), entry: ENTRY.to_string() })?;
        if let Kind::Action { next: n, .. } = &mut f.events[*g].kind {
            *n = s;
        }
        for k in who {
            f.entries[*k].subs.push(s);
        }
    }
    f.entries.push(Entry { name: ENTRY.to_string(), subs: Vec::new(), vars: Vec::new(), main: rungs[0], pad });
    Ok(())
}

fn same_but_actions(a: &crate::bfevfl::Actor, b: &crate::bfevfl::Actor) -> bool {
    let mut x: crate::bfevfl::Actor = b.clone();
    x.actions.truncate(a.actions.len());
    x == *a && [POUCH, UI, WAITER].contains(&b.name.as_str())
}

pub fn audit(a: &Flow, b: &Flow) -> Vec<String> {
    let mut bad: Vec<String> = sound(b);
    if a.name != b.name || b.actors.len() != a.actors.len() || b.events.len() < a.events.len() || b.entries.len() != a.entries.len() + 1 {
        bad.push("the flow lost or gained actors, events or entries it should not".to_string());
        return bad;
    }
    for (i, (x, y)) in a.actors.iter().zip(&b.actors).enumerate() {
        let asks: bool = x.name == DATA && y.queries.starts_with(&x.queries) && {
            let mut z: crate::bfevfl::Actor = y.clone();
            z.queries.truncate(x.queries.len());
            z == *x
        };
        if x != y && !same_but_actions(x, y) && !asks {
            bad.push(format!("actor {i} {} changed in more than new pouch, interface or wait actions", x.name));
        }
    }
    for (i, (x, y)) in a.events.iter().zip(&b.events).enumerate() {
        if x == y {
            continue;
        }
        let spliced: bool = GRANTS.iter().any(|(g, _, _)| *g == i)
            && matches!((&x.kind, &y.kind), (Kind::Action { actor: a0, act: c0, params: p0, .. }, Kind::Action { actor: a1, act: c1, params: p1, .. }) if (a0, c0, p0) == (a1, c1, p1));
        if !spliced {
            bad.push(format!("event {i} {} changed", x.name));
        }
    }
    for (i, y) in b.events.iter().enumerate().skip(a.events.len()) {
        let fine: bool = match &y.kind {
            Kind::Action { actor, act, .. } => match action_name(b, *actor, *act) {
                Some((POUCH, ADD)) | Some((WAITER, WAIT)) => true,
                Some((UI, n)) => HOOKS.iter().flat_map(|h| h.steps.iter()).any(|s| matches!(s, Step::Ui(m, _) if *m == n)),
                _ => false,
            },
            Kind::Switch { actor, query, .. } => b.actors.get(*actor as usize).is_some_and(|x| x.name == DATA && x.queries.get(*query as usize).is_some_and(|q| q == ASK)),
            Kind::Sub { flow, entry, .. } => flow.is_empty() && entry == ENTRY,
            _ => false,
        };
        if !fine {
            bad.push(format!("new event {i} {} is not part of the shelf hook", y.name));
        }
    }
    for (x, y) in a.entries.iter().zip(&b.entries) {
        if y.subs.len() < x.subs.len() {
            bad.push(format!("entry {} lost a sub call", x.name));
            continue;
        }
        let mut z: Entry = y.clone();
        z.subs.truncate(x.subs.len());
        let added: bool = y.subs[x.subs.len()..].iter().all(|s| matches!(b.events.get(*s as usize).map(|e| &e.kind), Some(Kind::Sub { entry, .. }) if entry == ENTRY));
        if z != *x || !added {
            bad.push(format!("entry {} changed", x.name));
        }
    }
    if b.entries.last().is_none_or(|e| e.name != ENTRY) {
        bad.push(format!("the entry {ENTRY} is missing"));
    }
    bad
}

pub fn edit(raw: &[u8], names: &BS<String>) -> Res<Vec<u8>> {
    let hooks: Vec<&Hook> = names.iter().map(|n| by_name(n).ok_or_else(|| E::Msg(format!("shelf hook: {n} has no hook")))).collect::<Res<Vec<&Hook>>>()?;
    let orig: Bf = Bf::parse(raw)?;
    let mut bf: Bf = orig.clone();
    let before: &Flow = orig.flows.first().ok_or(E::Format("shelf hook: the shop flow file holds no flow"))?;
    let f: &mut Flow = bf.flows.first_mut().ok_or(E::Format("shelf hook: the shop flow file holds no flow"))?;
    apply(f, &hooks)?;
    if let Some(m) = audit(before, f).first() {
        return Err(E::Msg(format!("shelf hook: the edited shop flow fails its audit, {m}")));
    }
    let out: Vec<u8> = bf.write()?;
    if Bf::parse(&out)? != bf {
        return Err(E::Format("shelf hook: the written shop flow does not parse back to the edited model"));
    }
    Ok(out)
}

fn text(v: &Val) -> String {
    match v {
        Val::Str(s) | Val::Arg(s) => s.clone(),
        Val::Int(n) => n.to_string(),
        Val::Bool(x) => (*x != 0).to_string(),
        x => format!("{x:?}"),
    }
}

pub fn line(actor: &str, act: &str, c: &Option<Cont>) -> String {
    let mut ps: Vec<String> = c.iter().flat_map(|c| c.items.iter()).map(|(k, v)| format!("{k}={}", text(v))).collect();
    ps.sort();
    let mut out: String = format!("{actor}.{act}");
    for p in ps {
        out.push(' ');
        out.push_str(&p);
    }
    out
}

pub fn run_for(f: &Flow, name: &str) -> Res<Vec<String>> {
    let e: &Entry = f.entries.iter().find(|e| e.name == ENTRY).ok_or_else(|| E::Msg(format!("the shop flow has no {ENTRY}")))?;
    let mut at: u16 = e.main;
    let mut out: Vec<String> = Vec::new();
    for _ in 0..LIMIT {
        if at == NONE {
            return Ok(out);
        }
        let ev: &Event = f.events.get(at as usize).ok_or_else(|| E::Msg(format!("the hook leads to event {at} outside the flow")))?;
        at = match &ev.kind {
            Kind::Switch { actor, query, cases, params } => {
                let who: &str = f.actors.get(*actor as usize).map(|a| a.name.as_str()).unwrap_or("");
                let q: &str = f.actors.get(*actor as usize).and_then(|a| a.queries.get(*query as usize)).map(|s| s.as_str()).unwrap_or("");
                let data: Option<&Val> = params.as_ref().and_then(|c| c.get("GameDataName"));
                if who != DATA || q != ASK || data != Some(&Val::Str(NAME.to_string())) {
                    return Err(E::Msg(format!("hook event {} is not a test of {NAME}", ev.name)));
                }
                let hit: bool = params.as_ref().and_then(|c| c.get("Value")) == Some(&Val::Str(name.to_string()));
                let want: u32 = if hit { 0 } else { 1 };
                cases.iter().find(|c| c.val == want).map(|c| c.ev).unwrap_or(NONE)
            }
            Kind::Action { next, actor, act, params } => {
                let (who, what): (&str, &str) = action_name(f, *actor, *act).ok_or_else(|| E::Msg(format!("hook event {} names an unknown action", ev.name)))?;
                out.push(line(who, what, params));
                *next
            }
            _ => return Err(E::Msg(format!("hook event {} is neither a test nor an action", ev.name))),
        };
    }
    Err(E::Format("the shelf hook does not end"))
}

pub fn expected(h: &Hook) -> Vec<String> {
    h.steps
        .iter()
        .map(|s| match s {
            Step::Add(a, n) => line(POUCH, ADD, &Some(add_cont(a, *n))),
            Step::Ui(m, p) => line(UI, m, &p.map(|(k, v)| Cont { items: vec![(k.to_string(), b(v))] })),
            Step::Wait(n) => line(WAITER, WAIT, &Some(wait_cont(*n))),
        })
        .collect()
}

pub fn names(raw: &[u8]) -> Res<BS<String>> {
    let bf: Bf = Bf::parse(raw)?;
    let f: &Flow = bf.flows.first().ok_or(E::Format("the shop flow file holds no flow"))?;
    let Some(e) = f.entries.iter().find(|e| e.name == ENTRY) else {
        return Ok(BS::new());
    };
    let mut out: BS<String> = BS::new();
    for i in reach(f, e.main) {
        if let Some(Event { kind: Kind::Switch { params: Some(c), .. }, .. }) = f.events.get(i as usize) {
            if let Some(Val::Str(s)) = c.get("Value") {
                out.insert(s.clone());
            }
        }
    }
    Ok(out)
}

pub fn check(raw: &[u8], want: &BS<String>) -> Vec<String> {
    let mut bad: Vec<String> = Vec::new();
    let bf: Bf = match Bf::parse(raw) {
        Ok(b) => b,
        Err(e) => return vec![format!("the shop flow does not parse: {e}")],
    };
    let Some(f) = bf.flows.first() else {
        return vec!["the shop flow file holds no flow".to_string()];
    };
    bad.extend(sound(f));
    for (g, name, old) in GRANTS {
        let s: Option<u16> = match grant_ok(f, g, name) {
            Ok(s) => Some(s),
            Err(e) => {
                bad.push(e.to_string());
                None
            }
        };
        let ok: bool = s.and_then(|s| f.events.get(s as usize)).is_some_and(|e| matches!(&e.kind, Kind::Sub { next, flow, entry, .. } if *next == old && flow.is_empty() && entry == ENTRY));
        if !ok {
            bad.push(format!("the grant {name} does not call {ENTRY} and return to event {old}"));
        }
    }
    let have: BS<String> = names(raw).unwrap_or_default();
    for n in want {
        if !have.contains(n) {
            bad.push(format!("the hook has no rung for {n}"));
        }
    }
    for n in &have {
        let Some(h) = by_name(n) else {
            bad.push(format!("the hook tests {n}, which has no hook"));
            continue;
        };
        match run_for(f, n) {
            Ok(got) if got == expected(h) => {}
            Ok(got) => bad.push(format!("the hook for {n} runs {got:?}")),
            Err(e) => bad.push(e.to_string()),
        }
    }
    match run_for(f, "Item_Boomerang") {
        Ok(got) if got.is_empty() => {}
        Ok(got) => bad.push(format!("a plain shelf item runs {got:?}")),
        Err(e) => bad.push(e.to_string()),
    }
    bad
}

pub fn sized(rst: Option<u32>, stock: Option<u32>, old: usize, new: usize) -> bool {
    match (rst, stock) {
        (Some(r), Some(s)) => new <= old || r as u64 >= s as u64 + crate::rsizetable::align(new as u64) - crate::rsizetable::align(old as u64),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf as PB;

    use super::*;
    use crate::bfevfl::Actor;
    use crate::pregrant::pregrant;
    use crate::progressive::owners;

    fn actor(name: &str, acts: &[&str], qs: &[&str]) -> Actor {
        Actor { name: name.to_string(), sub: String::new(), arg: String::new(), actions: acts.iter().map(|s| s.to_string()).collect(), queries: qs.iter().map(|s| s.to_string()).collect(), params: None, ep: 0, cut: 0 }
    }

    fn fake() -> Flow {
        let actors: Vec<Actor> = vec![actor(STARTER, &[GRANT], &[]), actor(DATA, &[], &[ASK]), actor(POUCH, &[], &[]), actor(UI, &[], &[]), actor(WAITER, &[WAIT], &[])];
        let mut events: Vec<Event> = (0..470).map(|i| Event { name: format!("Event{i}"), kind: Kind::Action { next: NONE, actor: 4, act: 0, params: Some(wait_cont(1)) } }).collect();
        events[156] = Event { name: "Event281".to_string(), kind: Kind::Action { next: 277, actor: 0, act: 0, params: None } };
        events[461] = Event { name: "Event543".to_string(), kind: Kind::Action { next: 430, actor: 0, act: 0, params: None } };
        let entries: Vec<Entry> = vec![
            Entry { name: "NormalFlow".to_string(), subs: vec![], vars: vec![], main: 461, pad: 0 },
            Entry { name: "ShopItem_Buy_SecondMove".to_string(), subs: vec![], vars: vec![], main: 156, pad: 0 },
            Entry { name: "Other".to_string(), subs: vec![], vars: vec![], main: 3, pad: 0 },
        ];
        Flow { name: "DisplayShop".to_string(), actors, events, entries }
    }

    fn hooked(names: &[&str]) -> Flow {
        let mut f: Flow = fake();
        let hs: Vec<&Hook> = names.iter().map(|n| by_name(n).unwrap()).collect();
        apply(&mut f, &hs).unwrap();
        f
    }

    fn golden(h: &Hook) -> &'static str {
        match h.family {
            "Bow" => include_str!("golden/bow.txt"),
            "Slingshot" => include_str!("golden/slingshot.txt"),
            "BombBag" => include_str!("golden/bomb_bag.txt"),
            _ => include_str!("golden/magic.txt"),
        }
    }

    fn rendered(f: &Flow, h: &Hook) -> String {
        let mut out: String = format!("{} sells {} as {}\n", h.family, h.shelf, h.name);
        for l in run_for(f, h.name).unwrap() {
            out.push_str("  ");
            out.push_str(&l);
            out.push('\n');
        }
        out
    }

    fn one_class(family: &str) {
        let h: &Hook = by_family(family).unwrap();
        let f: Flow = hooked(&[h.name]);
        assert_eq!(rendered(&f, h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
        assert_eq!(run_for(&f, h.name).unwrap(), expected(h));
        assert!(run_for(&f, "Item_Boomerang").unwrap().is_empty());
        let t: &[&str] = tiers(h.family).unwrap();
        assert_eq!((h.shelf, h.name), (t[0], owners(h.family).unwrap()[0]));
        let last: Option<&str> = h.steps.iter().filter_map(|s| if let Step::Add(a, _) = s { Some(*a) } else { None }).next_back();
        assert_eq!(last, t.last().copied());
        for g in pregrant(h.name).iter().filter(|g| **g != h.name) {
            assert!(h.steps.iter().any(|s| matches!(s, Step::Add(a, _) if a == g)), "{family} misses the chest pre-grant {g}");
        }
        let mut both: Bf = Bf { ver: [0, 4, 0, 0], bom: 0xFEFF, shift: 3, pad: 0, name: "DisplayShop".to_string(), flows: vec![fake()], tls: Vec::new() };
        let raw: Vec<u8> = both.write().unwrap();
        let out: Vec<u8> = edit(&raw, &[h.name.to_string()].into_iter().collect()).unwrap();
        assert!(check(&out, &[h.name.to_string()].into_iter().collect()).is_empty());
        both = Bf::parse(&out).unwrap();
        assert_eq!(rendered(&both.flows[0], h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
    }

    #[test]
    fn a_bow_deal_adds_the_quiver_and_arrows_of_the_chest_then_the_top_quiver() {
        one_class("Bow");
    }

    #[test]
    fn a_slingshot_deal_adds_the_bullet_bag_and_seeds_of_the_chest_then_the_top_bag() {
        one_class("Slingshot");
    }

    #[test]
    fn a_bomb_bag_deal_adds_the_bombs_of_the_chest_then_the_top_bag() {
        one_class("BombBag");
    }

    #[test]
    fn a_magic_deal_adds_double_magic_and_the_great_fairy_gauge_steps() {
        one_class("Magic");
    }

    #[test]
    fn the_hook_splices_both_grants_and_keeps_the_rest_of_the_flow() {
        let before: Flow = fake();
        let f: Flow = hooked(&["Item_BombBag_Small", "Item_FairyBless_MP", "Weapon_Range_Bow", "Weapon_Range_SlingShot"]);
        assert!(audit(&before, &f).is_empty(), "{:?}", audit(&before, &f));
        for (g, _, old) in GRANTS {
            let Kind::Action { next, .. } = f.events[g].kind else { panic!() };
            assert!(matches!(&f.events[next as usize].kind, Kind::Sub { next: n, entry, .. } if *n == old && entry == ENTRY));
        }
        assert_eq!(f.entries[0].subs.len(), 1);
        assert_eq!(f.entries[1].subs.len(), 1);
        assert!(f.entries[2].subs.is_empty());
        assert_eq!(f.actors[2].actions, vec![ADD.to_string()]);
        assert_eq!(f.actors[4].actions, vec![WAIT.to_string()]);
        for h in &HOOKS {
            assert_eq!(run_for(&f, h.name).unwrap(), expected(h));
        }
        assert!(run_for(&f, "Weapon_Shield_Deku").unwrap().is_empty());
        let mut again: Flow = f.clone();
        assert!(apply(&mut again, &[&HOOKS[0]]).is_err());
    }

    #[test]
    fn a_flow_that_differs_is_refused() {
        let mut f: Flow = fake();
        f.events[461].name = "Event999".to_string();
        assert!(apply(&mut f, &[&HOOKS[0]]).unwrap_err().to_string().contains("game files differ"));
        let mut g: Flow = fake();
        if let Kind::Action { next, .. } = &mut g.events[156].kind {
            *next = 3;
        }
        assert!(apply(&mut g, &[&HOOKS[0]]).is_err());
        assert!(apply(&mut fake(), &[]).is_err());
    }

    #[test]
    fn tiers_of_a_hooked_family_map_to_its_shelf_item() {
        assert_eq!(shelf_item("Item_Quiver_Large"), "Item_Range_Bow");
        assert_eq!(shelf_item("Item_BulletBag_Medium"), "Item_Range_SlingShot");
        assert_eq!(shelf_item("Item_BombBag_Large"), "Item_BombBag_Small");
        assert_eq!(shelf_item("Item_FairyBless_MPDouble"), "Item_FairyBless_MP");
        assert_eq!(shelf_item("Item_Longshot"), "Item_Longshot");
        assert!(by_family("Hookshot").is_none());
        assert!(sized(Some(5000), Some(4000), 3000, 3100));
        assert!(!sized(Some(4000), Some(4000), 3000, 3100));
        assert!(!sized(None, Some(4000), 3000, 3100));
    }

    fn real() -> Option<PB> {
        let p: PB = std::env::var_os("OOTRR_ROMFS").map(PB::from).unwrap_or_else(|| PB::from("C:/o/nup dump/updates/131072/romfs"));
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(p)
        } else {
            eprintln!("skipped: no romfs at {}, set OOTRR_ROMFS", p.display());
            None
        }
    }

    #[test]
    fn the_shipped_shop_flow_takes_the_hook_and_reads_back() {
        let Some(p) = real() else {
            return;
        };
        let mut rf: crate::romfs::Romfs = crate::romfs::Romfs::open(&p).unwrap();
        let raw: Vec<u8> = rf.read_zs(FLOW).unwrap();
        let all: BS<String> = HOOKS.iter().map(|h| h.name.to_string()).collect();
        let out: Vec<u8> = edit(&raw, &all).unwrap();
        assert!(out.len() > raw.len());
        assert!(check(&out, &all).is_empty(), "{:?}", check(&out, &all));
        assert_eq!(names(&out).unwrap(), all);
        assert!(names(&raw).unwrap().is_empty());
        let f: Flow = Bf::parse(&out).unwrap().flows.remove(0);
        for (en, g) in [("NormalFlow", 461usize), ("ShopItem_Buy_SecondMove", 156)] {
            let e: &Entry = f.entries.iter().find(|e| e.name == en).unwrap();
            let Kind::Action { next, .. } = f.events[g].kind else { panic!() };
            assert!(e.subs.contains(&next), "{en}");
        }
        for h in &HOOKS {
            assert_eq!(rendered(&f, h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
        }
        let one: Vec<u8> = edit(&raw, &[HOOKS[2].name.to_string()].into_iter().collect()).unwrap();
        assert_eq!(names(&one).unwrap().len(), 1);
    }

    #[test]
    fn the_steps_copy_the_chest_pre_grant_counts_and_the_great_fairy_gauge() {
        let Some(p) = real() else {
            return;
        };
        let mut rf: crate::romfs::Romfs = crate::romfs::Romfs::open(&p).unwrap();
        let es: Vec<crate::sarc::Entry> = crate::sarc::parse(&rf.read_zs("Pack/AI.Global.Product.111.pack.zs").unwrap()).unwrap();
        let m: &crate::sarc::Entry = crate::sarc::get(&es, crate::pregrant::MODULE).unwrap();
        let t: String = crate::ainb::show::show(&crate::ainb::Ainb::parse(&m.data).unwrap());
        let mut adds: Vec<(String, i32)> = Vec::new();
        let mut num: Option<i32> = None;
        for l in t.lines().map(|l| l.trim()) {
            if let Some(r) = l.strip_prefix("in s32 Num = ") {
                num = r.split(' ').next().and_then(|x| x.parse().ok());
            }
            if let Some(r) = l.strip_prefix("in str Actor = \"Work/Actor/") {
                adds.push((r.split('.').next().unwrap().to_string(), num.take().unwrap()));
            }
        }
        for h in HOOKS.iter().filter(|h| h.family != "Magic") {
            for g in pregrant(h.name).iter().filter(|g| **g != h.name) {
                let n: i32 = adds.iter().find(|(a, _)| a == g).map(|x| x.1).unwrap();
                assert!(h.steps.contains(&Step::Add(g, n)), "{} {g} {n}", h.family);
            }
        }
        let gi: Flow = Bf::parse(&rf.read_zs("Event/EventFlow/Dm_SY_GetInEvent.bfevfl.zs").unwrap()).unwrap().flows.remove(0);
        let mut at: u16 = gi.events.iter().position(|e| e.name == "Event48").unwrap() as u16;
        let mut got: Vec<String> = Vec::new();
        while let Some(Event { kind: Kind::Action { next, actor, act, params }, .. }) = gi.events.get(at as usize) {
            let (who, what) = action_name(&gi, *actor, *act).unwrap();
            if what.contains("ActionGuide") || got.len() == 5 {
                break;
            }
            got.push(line(who, what, params));
            at = *next;
        }
        let want: Vec<String> = expected(by_family("Magic").unwrap()).into_iter().skip(1).collect();
        assert_eq!(got, want);
    }
}
