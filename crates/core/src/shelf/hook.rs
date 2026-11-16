//! The shelf hook: every progressive family sold on a special deal.
//!
//! Both shop grants are replaced by a test of the bought item. A hooked item then asks the
//! same ownership questions as a progressive chest, before anything is handed over: a player
//! who owns none of the family gets the shop grant of the first tier and what the chest or the
//! Great Fairy adds with it, a player who owns a tier gets the next tier instead of the grant.
//! Tier items are added with the item-get flag, as a chest adds them: the shop grant alone gave
//! the magic bar of the Gift of Improved Spin but not the spin.
//! Every path goes on to the event the grant led to. Other items reach an untouched copy of the
//! grant.

use std::collections::BTreeSet as BS;

use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Cont, Event, Flow, Kind, Val};
use crate::dungeon_credit::{action_slot, sound};
use crate::error::{Error as E, Res};
use crate::progressive::{owners, tiers};

pub const FLOW: &str = "Event/EventFlow/DisplayShop.bfevfl.zs";
pub const NAME: &str = "Shop_SelectItemName";
pub const GRANTS: [(usize, &str, u16); 2] = [(156, "Event281", 277), (461, "Event543", 430)];
/// How `run_for` writes the shop grant of the first tier.
pub const GIVE: &str = "shop grant";

const GRANT: &str = "EventTriggerGetItemAtShop";
const STARTER: &str = "Npc_EventStarter";
const DATA: &str = "GameDataActor";
const ASK: &str = "EventQueryIsGameDataString64";
const OWNED: &str = "EventQueryCheckIsGet";
const POUCH: &str = "PouchSystemActor";
const ADD: &str = "EventTriggerAddOrRemovePouchContent";
const UI: &str = "UISystemActor";
const WAITER: &str = "EventSystemActor";
const WAIT: &str = "EventWait";
const LIMIT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Adds an amount the way the chest pre-grant does, without the item-get flag.
    Add(&'static str, i32),
    /// Adds one tier item the way a chest hands it over, with the item-get flag set.
    Get(&'static str),
    Ui(&'static str, Option<(&'static str, bool)>),
    Wait(i32),
}

/// A hooked family: `base` gives the first tier, `ups[i]` gives tier `i + 1`. With `grant` the
/// first tier also runs the shop grant before `base`; without it the shelf shows another tier,
/// whose grant must not run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hook {
    pub family: &'static str,
    pub shelf: &'static str,
    pub name: &'static str,
    pub grant: bool,
    pub base: &'static [Step],
    pub ups: &'static [&'static [Step]],
}

const SHOWN: Option<(&str, bool)> = Some(("IsDisplayHeartGauge", true));
const WAITED: Option<(&str, bool)> = Some(("IsWaitEnd", true));

/// The families sold as gear deals under their own setting. Every other hooked family is an
/// upgrade deal.
pub const GEAR: [&str; 4] = ["Bow", "Slingshot", "BombBag", "Magic"];

pub const HOOKS: [Hook; 11] = [
    Hook {
        family: "Bow",
        shelf: "Item_Range_Bow",
        name: "Weapon_Range_Bow",
        grant: true,
        base: &[Step::Get("Item_Range_Bow"), Step::Add("Item_Quiver_Small", 1), Step::Add("Item_Arrow_Normal", 30)],
        ups: &[&[Step::Get("Item_Quiver_Medium")], &[Step::Get("Item_Quiver_Large")]],
    },
    Hook {
        family: "Slingshot",
        shelf: "Item_Range_SlingShot",
        name: "Weapon_Range_SlingShot",
        grant: true,
        base: &[Step::Get("Item_Range_SlingShot"), Step::Add("Item_BulletBag_Small", 1), Step::Add("Item_DekuSeed", 30)],
        ups: &[&[Step::Get("Item_BulletBag_Medium")], &[Step::Get("Item_BulletBag_Large")]],
    },
    Hook {
        family: "BombBag",
        shelf: "Item_BombBag_Small",
        name: "Item_BombBag_Small",
        grant: true,
        base: &[Step::Get("Item_BombBag_Small"), Step::Add("Item_Bomb", 20)],
        ups: &[&[Step::Get("Item_BombBag_Medium")], &[Step::Get("Item_BombBag_Large")]],
    },
    Hook {
        family: "Magic",
        shelf: "Item_FairyBless_MP",
        name: "Item_FairyBless_MP",
        grant: true,
        base: &[
            Step::Get("Item_FairyBless_MP"),
            Step::Ui("EventTriggerReserveBornMPGauge", None),
            Step::Ui("EventTriggerSetDisplayHeartGauge", SHOWN),
            Step::Wait(100),
            Step::Ui("EventBornMPGauge", WAITED),
            Step::Wait(30),
        ],
        ups: &[&[
            Step::Get("Item_FairyBless_MPDouble"),
            Step::Ui("EventTriggerReserveBornMPGaugeDouble", None),
            Step::Ui("EventTriggerSetDisplayHeartGauge", SHOWN),
            Step::Wait(100),
            Step::Ui("EventBornMPGaugeDouble", WAITED),
            Step::Wait(30),
        ]],
    },
    Hook { family: "Hookshot", shelf: "Item_Hookshot", name: "Item_Hookshot", grant: true, base: &[Step::Get("Item_Hookshot")], ups: &[&[Step::Get("Item_Longshot")]] },
    // The Goron's Bracelet cannot stand on a shelf: Item_Gauntlet_Goron has no shop name text and
    // Armor_Child_Gauntlet_Goron has no animation. The shelf shows the Silver Gauntlets instead
    // and skips their grant, so the purchase gives the tier the player needs.
    Hook {
        family: "Strength",
        shelf: "Item_Gauntlet_Silver",
        name: "Armor_Adult_Gauntlet_Silver",
        grant: false,
        base: &[Step::Get("Item_Gauntlet_Goron")],
        ups: &[&[Step::Get("Item_Gauntlet_Silver")], &[Step::Get("Item_Gauntlet_Gold")]],
    },
    Hook { family: "Wallet", shelf: "Item_Wallet_Medium", name: "Item_Wallet_Medium", grant: true, base: &[Step::Get("Item_Wallet_Medium")], ups: &[&[Step::Get("Item_Wallet_Large")]] },
    Hook { family: "Scale", shelf: "Item_Scale_Silver", name: "Item_Scale_Silver", grant: true, base: &[Step::Get("Item_Scale_Silver")], ups: &[&[Step::Get("Item_Scale_Gold")]] },
    Hook { family: "NutBag", shelf: "Item_NutBag_Medium", name: "Item_NutBag_Medium", grant: true, base: &[Step::Get("Item_NutBag_Medium")], ups: &[&[Step::Get("Item_NutBag_Large")]] },
    Hook { family: "StickBag", shelf: "Item_StickBag_Medium", name: "Item_StickBag_Medium", grant: true, base: &[Step::Get("Item_StickBag_Medium")], ups: &[&[Step::Get("Item_StickBag_Large")]] },
    Hook { family: "Ocarina", shelf: "Item_Ocarina_Fairy", name: "Item_Ocarina_Fairy", grant: true, base: &[Step::Get("Item_Ocarina_Fairy")], ups: &[&[Step::Get("Item_Ocarina_Time")]] },
];

pub fn gear(family: &str) -> bool {
    GEAR.contains(&family)
}

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

fn add_cont(actor: &str, n: i32, get: bool) -> Cont {
    Cont {
        items: vec![
            ("Num".to_string(), Val::Int(n)),
            ("IsEquipDIrect".to_string(), b(false)),
            ("IsSetMessageTargetActorName".to_string(), b(false)),
            ("Actor".to_string(), Val::Str(actor.to_string())),
            ("IsGetEvent".to_string(), b(get)),
        ],
    }
}

fn ask_cont(name: &str) -> Cont {
    Cont { items: vec![("GameDataName".to_string(), Val::Str(NAME.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(name.to_string()))] }
}

fn wait_cont(n: i32) -> Cont {
    Cont { items: vec![("IsWaitEnd".to_string(), b(true)), ("Frames".to_string(), Val::Int(n))] }
}

fn ui_cont(p: Option<(&str, bool)>) -> Option<Cont> {
    p.map(|(k, v)| Cont { items: vec![(k.to_string(), b(v))] })
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
            Kind::Action { next, actor, act, params: Some(add_cont(a, *n, false)) }
        }
        Step::Get(a) => {
            let (actor, act): (u16, u16) = action_slot(f, POUCH, ADD)?;
            Kind::Action { next, actor, act, params: Some(add_cont(a, 1, true)) }
        }
        Step::Ui(name, p) => {
            let (actor, act): (u16, u16) = action_slot(f, UI, name)?;
            Kind::Action { next, actor, act, params: ui_cont(*p) }
        }
        Step::Wait(n) => {
            let (actor, act): (u16, u16) = action_slot(f, WAITER, WAIT)?;
            Kind::Action { next, actor, act, params: Some(wait_cont(*n)) }
        }
    })
}

/// Pushes the steps as one chain that ends at `end` and returns where it starts.
fn steps_chain(f: &mut Flow, steps: &[Step], end: u16) -> Res<u16> {
    let mut at: u16 = end;
    for s in steps.iter().rev() {
        let k: Kind = step_kind(f, s, at)?;
        at = push(f, k)?;
    }
    Ok(at)
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

fn action_name<'a>(f: &'a Flow, actor: u16, act: u16) -> Option<(&'a str, &'a str)> {
    let a: &crate::bfevfl::Actor = f.actors.get(actor as usize)?;
    Some((a.name.as_str(), a.actions.get(act as usize)?.as_str()))
}

fn query_name<'a>(f: &'a Flow, actor: u16, query: u16) -> Option<(&'a str, &'a str)> {
    let a: &crate::bfevfl::Actor = f.actors.get(actor as usize)?;
    Some((a.name.as_str(), a.queries.get(query as usize)?.as_str()))
}

fn is_grant(f: &Flow, k: &Kind) -> bool {
    matches!(k, Kind::Action { actor, act, .. } if action_name(f, *actor, *act) == Some((STARTER, GRANT)))
}

fn grant_ok(f: &Flow, i: usize, name: &str) -> Res<u16> {
    let e: &Event = f.events.get(i).ok_or_else(|| E::Msg(format!("shelf hook: the shop flow has no event {i}, the game files differ")))?;
    let Kind::Action { next, .. } = &e.kind else {
        return Err(E::Msg(format!("shelf hook: event {i} is not an action, the game files differ")));
    };
    if e.name != name || !is_grant(f, &e.kind) {
        return Err(E::Msg(format!("shelf hook: event {i} is not the shop grant {name}, the game files differ")));
    }
    Ok(*next)
}

fn with_next(k: &Kind, to: u16) -> Kind {
    let mut out: Kind = k.clone();
    if let Kind::Action { next, .. } = &mut out {
        *next = to;
    }
    out
}

/// The item a hook rung tests, when the event is one.
fn is_rung(f: &Flow, k: &Kind) -> Option<String> {
    let Kind::Switch { actor, query, params: Some(c), .. } = k else {
        return None;
    };
    if query_name(f, *actor, *query) != Some((DATA, ASK)) || c.get("GameDataName") != Some(&Val::Str(NAME.to_string())) {
        return None;
    }
    match c.get("Value") {
        Some(Val::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

pub fn apply(f: &mut Flow, hooks: &[&Hook]) -> Res<()> {
    if hooks.is_empty() {
        return Err(E::Format("shelf hook: no item needs the hook"));
    }
    for (i, name, next) in GRANTS {
        if f.events.get(i).is_some_and(|e| is_rung(f, &e.kind).is_some()) {
            return Err(E::Format("shelf hook: the shop flow already holds the hook"));
        }
        if grant_ok(f, i, name)? != next {
            return Err(E::Msg(format!("shelf hook: the grant {name} does not lead to event {next}, the game files differ")));
        }
    }
    for h in hooks {
        if h.ups.len() + 1 != tiers(h.family)?.len() {
            return Err(E::Msg(format!("shelf hook: {} does not give every tier of its family", h.family)));
        }
    }
    let (da, dq): (u16, u16) = query_slot(f, DATA, ASK)?;
    let (oa, oq): (u16, u16) = crate::progressive::ensure(f, DATA, OWNED)?;
    for (g, _, next) in GRANTS {
        let grant: Kind = f.events[g].kind.clone();
        let plain: u16 = push(f, grant.clone())?;
        let mut roots: Vec<u16> = Vec::new();
        for h in hooks {
            let mut gives: Vec<u16> = Vec::new();
            let after: u16 = steps_chain(f, h.base, next)?;
            gives.push(if h.grant { push(f, with_next(&grant, after))? } else { after });
            for up in h.ups {
                gives.push(steps_chain(f, up, next)?);
            }
            let root: Kind = crate::progressive::chain(f, oa, oq, h.family, owners(h.family)?, &gives)?;
            roots.push(push(f, root)?);
        }
        let mut miss: u16 = plain;
        for k in (0..hooks.len()).rev() {
            let rung: Kind = Kind::Switch { actor: da, query: dq, cases: vec![Case { val: 0, ev: roots[k] }, Case { val: 1, ev: miss }], params: Some(ask_cont(hooks[k].name)) };
            if k == 0 {
                f.events[g].kind = rung;
            } else {
                miss = push(f, rung)?;
            }
        }
    }
    Ok(())
}

fn same_but_actions(a: &crate::bfevfl::Actor, b: &crate::bfevfl::Actor) -> bool {
    let mut x: crate::bfevfl::Actor = b.clone();
    x.actions.truncate(a.actions.len());
    x == *a && [POUCH, UI, WAITER].contains(&b.name.as_str())
}

fn hook_ui(n: &str) -> bool {
    HOOKS.iter().flat_map(|h| h.base.iter().chain(h.ups.iter().flat_map(|u| u.iter()))).any(|s| matches!(s, Step::Ui(m, _) if *m == n))
}

pub fn audit(a: &Flow, b: &Flow) -> Vec<String> {
    let mut bad: Vec<String> = sound(b);
    if a.name != b.name || b.actors.len() != a.actors.len() || b.events.len() < a.events.len() || b.entries != a.entries {
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
            bad.push(format!("actor {i} {} changed in more than new pouch, interface, wait or ownership calls", x.name));
        }
    }
    for (i, (x, y)) in a.events.iter().zip(&b.events).enumerate() {
        if x == y {
            continue;
        }
        let spliced: bool = GRANTS.iter().any(|(g, _, _)| *g == i) && x.name == y.name && is_rung(b, &y.kind).is_some();
        if !spliced {
            bad.push(format!("event {i} {} changed", x.name));
        }
    }
    let grants: Vec<&Kind> = GRANTS.iter().filter_map(|(g, _, _)| a.events.get(*g).map(|e| &e.kind)).collect();
    for (i, y) in b.events.iter().enumerate().skip(a.events.len()) {
        let fine: bool = match &y.kind {
            Kind::Action { actor, act, params, .. } => match action_name(b, *actor, *act) {
                Some((POUCH, ADD)) | Some((WAITER, WAIT)) => true,
                Some((UI, n)) => hook_ui(n),
                Some((STARTER, GRANT)) => grants.iter().any(|g| matches!(g, Kind::Action { params: p, .. } if p == params)),
                _ => false,
            },
            Kind::Switch { actor, query, .. } => matches!(query_name(b, *actor, *query), Some((DATA, ASK)) | Some((DATA, OWNED))),
            _ => false,
        };
        if !fine {
            bad.push(format!("new event {i} {} is not part of the shelf hook", y.name));
        }
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

/// Runs the hooked grant `grant` (an index into `GRANTS`) for a purchase of `name` by a player
/// who owns the actors in `owned`, and returns what it does up to the event the grant led to.
pub fn run_for(f: &Flow, grant: usize, name: &str, owned: &[&str]) -> Res<Vec<String>> {
    let (g, _, end): (usize, &str, u16) = *GRANTS.get(grant).ok_or(E::Format("the shop flow has two grants"))?;
    let mut at: u16 = idx(g)?;
    let mut out: Vec<String> = Vec::new();
    for _ in 0..LIMIT {
        if at == end {
            return Ok(out);
        }
        let ev: &Event = f.events.get(at as usize).ok_or_else(|| E::Msg(format!("the hook leads to event {at} outside the flow")))?;
        at = match &ev.kind {
            Kind::Switch { actor, query, cases, params } => {
                let want: u32 = match (query_name(f, *actor, *query), params.as_ref()) {
                    (Some((DATA, ASK)), Some(c)) if c.get("GameDataName") == Some(&Val::Str(NAME.to_string())) => u32::from(c.get("Value") != Some(&Val::Str(name.to_string()))),
                    (Some((DATA, OWNED)), Some(c)) => u32::from(!matches!(c.get("ActorName"), Some(Val::Str(s)) if owned.contains(&s.as_str()))),
                    _ => return Err(E::Msg(format!("hook event {} is not a test of {NAME} or of ownership", ev.name))),
                };
                cases.iter().find(|c| c.val == want).map(|c| c.ev).unwrap_or(NONE)
            }
            Kind::Action { next, actor, act, params } => {
                let (who, what): (&str, &str) = action_name(f, *actor, *act).ok_or_else(|| E::Msg(format!("hook event {} names an unknown action", ev.name)))?;
                out.push(if (who, what) == (STARTER, GRANT) { GIVE.to_string() } else { line(who, what, params) });
                *next
            }
            _ => return Err(E::Msg(format!("hook event {} is neither a test nor an action", ev.name))),
        };
    }
    Err(E::Format("the shelf hook does not reach the event the grant led to"))
}

pub fn step_lines(steps: &[Step]) -> Vec<String> {
    steps
        .iter()
        .map(|s| match s {
            Step::Add(a, n) => line(POUCH, ADD, &Some(add_cont(a, *n, false))),
            Step::Get(a) => line(POUCH, ADD, &Some(add_cont(a, 1, true))),
            Step::Ui(m, p) => line(UI, m, &ui_cont(*p)),
            Step::Wait(n) => line(WAITER, WAIT, &Some(wait_cont(*n))),
        })
        .collect()
}

/// What a purchase does for a player who owns the first `k` tiers of the family. Owning every
/// tier repeats the top one, as a progressive chest does.
pub fn expected(h: &Hook, k: usize) -> Vec<String> {
    if k == 0 {
        return h.grant.then(|| GIVE.to_string()).into_iter().chain(step_lines(h.base)).collect();
    }
    step_lines(h.ups[k.min(h.ups.len()) - 1])
}

pub fn names(raw: &[u8]) -> Res<BS<String>> {
    let bf: Bf = Bf::parse(raw)?;
    let f: &Flow = bf.flows.first().ok_or(E::Format("the shop flow file holds no flow"))?;
    let mut out: BS<String> = BS::new();
    for (g, _, _) in GRANTS {
        let mut at: u16 = idx(g)?;
        for _ in 0..LIMIT {
            let Some(e) = f.events.get(at as usize) else {
                break;
            };
            let Some(n) = is_rung(f, &e.kind) else {
                break;
            };
            out.insert(n);
            let Kind::Switch { cases, .. } = &e.kind else {
                break;
            };
            at = cases.iter().find(|c| c.val == 1).map(|c| c.ev).unwrap_or(NONE);
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
    for (g, name, _) in GRANTS {
        if f.events.get(g).is_none_or(|e| e.name != name || is_rung(f, &e.kind).is_none()) {
            bad.push(format!("the grant {name} is not replaced by the hook"));
        }
    }
    let have: BS<String> = names(raw).unwrap_or_default();
    for n in want {
        if !have.contains(n) {
            bad.push(format!("the hook has no rung for {n}"));
        }
    }
    for gi in 0..GRANTS.len() {
        for n in &have {
            let Some(h) = by_name(n) else {
                bad.push(format!("the hook tests {n}, which has no hook"));
                continue;
            };
            let o: &[&str] = owners(h.family).unwrap_or(&[]);
            for k in 0..=o.len() {
                match run_for(f, gi, n, &o[..k]) {
                    Ok(got) if got == expected(h, k) => {}
                    Ok(got) => bad.push(format!("the hook for {n} with {k} tiers owned runs {got:?}")),
                    Err(e) => bad.push(e.to_string()),
                }
            }
        }
        match run_for(f, gi, "Item_Boomerang", &[]) {
            Ok(got) if got == [GIVE] => {}
            Ok(got) => bad.push(format!("a plain shelf item runs {got:?}")),
            Err(e) => bad.push(e.to_string()),
        }
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
    use crate::bfevfl::{Actor, Entry};
    use crate::pregrant::pregrant;

    fn actor(name: &str, acts: &[&str], qs: &[&str]) -> Actor {
        Actor {
            name: name.to_string(),
            sub: String::new(),
            arg: String::new(),
            actions: acts.iter().map(|s| s.to_string()).collect(),
            queries: qs.iter().map(|s| s.to_string()).collect(),
            params: None,
            ep: 0,
            cut: 0,
        }
    }

    fn fake() -> Flow {
        let actors: Vec<Actor> = vec![actor(STARTER, &[GRANT], &[]), actor(DATA, &[], &[ASK]), actor(POUCH, &[], &[]), actor(UI, &[], &[]), actor(WAITER, &[WAIT], &[])];
        let mut events: Vec<Event> = (0..470).map(|i| Event { name: format!("Event{i}"), kind: Kind::Action { next: NONE, actor: 4, act: 0, params: Some(wait_cont(1)) } }).collect();
        let grant: Option<Cont> = Some(Cont { items: vec![("IsGetEvent".to_string(), b(false))] });
        events[156] = Event { name: "Event281".to_string(), kind: Kind::Action { next: 277, actor: 0, act: 0, params: grant.clone() } };
        events[461] = Event { name: "Event543".to_string(), kind: Kind::Action { next: 430, actor: 0, act: 0, params: grant } };
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

    const GOLDEN: &str = include_str!("golden/hooks.txt");

    fn golden(h: &Hook) -> String {
        let all: String = GOLDEN.replace("\r\n", "\n");
        let mut out: String = String::new();
        let mut on: bool = false;
        for l in all.lines() {
            if !l.starts_with(' ') && l.contains(" sells ") {
                on = l.starts_with(&format!("{} sells ", h.family));
            }
            if on {
                out.push_str(l);
                out.push('\n');
            }
        }
        out
    }

    fn rendered(f: &Flow, h: &Hook) -> String {
        let o: &[&str] = owners(h.family).unwrap();
        let mut out: String = format!("{} sells {} as {}\n", h.family, h.shelf, h.name);
        for k in 0..=o.len() {
            let owned: String = if k == 0 { "nothing".to_string() } else { o[..k].join(", ") };
            out.push_str(&format!("owning {owned}\n"));
            for l in run_for(f, 0, h.name, &o[..k]).unwrap() {
                out.push_str("  ");
                out.push_str(&l);
                out.push('\n');
            }
        }
        out
    }

    fn every_state_runs_as_expected(f: &Flow, h: &Hook) {
        let o: &[&str] = owners(h.family).unwrap();
        for gi in 0..GRANTS.len() {
            for k in 0..=o.len() {
                assert_eq!(run_for(f, gi, h.name, &o[..k]).unwrap(), expected(h, k), "{} grant {gi} owning {k}", h.family);
            }
        }
    }

    fn one_class(family: &str) {
        let h: &Hook = by_family(family).unwrap();
        let f: Flow = hooked(&[h.name]);
        assert_eq!(rendered(&f, h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
        every_state_runs_as_expected(&f, h);
        for gi in 0..GRANTS.len() {
            assert_eq!(run_for(&f, gi, "Item_Boomerang", &[]).unwrap(), vec![GIVE.to_string()]);
        }
        let t: &[&str] = tiers(h.family).unwrap();
        assert!(t.contains(&h.shelf), "{family}");
        assert_eq!(h.grant, h.shelf == t[0], "{family}");
        assert_eq!(h.ups.len() + 1, t.len());
        for (i, up) in h.ups.iter().enumerate() {
            assert_eq!(up.first(), Some(&Step::Get(t[i + 1])), "{family} tier {}", i + 1);
        }
        assert_eq!(h.base.first(), Some(&Step::Get(t[0])), "{family}");
        assert!(h.base.iter().all(|s| !matches!(s, Step::Add(a, _) | Step::Get(a) if t[1..].contains(a))), "{family} gives a later tier with the first one");
        for g in pregrant(h.name) {
            assert!(h.base.iter().any(|s| matches!(s, Step::Add(a, _) | Step::Get(a) if a == g)), "{family} misses the chest pre-grant {g}");
        }
        let both: Bf = Bf { ver: [0, 4, 0, 0], bom: 0xFEFF, shift: 3, pad: 0, name: "DisplayShop".to_string(), flows: vec![fake()], tls: Vec::new() };
        let raw: Vec<u8> = both.write().unwrap();
        let out: Vec<u8> = edit(&raw, &[h.name.to_string()].into_iter().collect()).unwrap();
        assert!(check(&out, &[h.name.to_string()].into_iter().collect()).is_empty());
        let back: Bf = Bf::parse(&out).unwrap();
        assert_eq!(rendered(&back.flows[0], h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
    }

    #[test]
    fn a_bow_deal_gives_the_bow_with_its_quiver_and_arrows_or_the_next_quiver() {
        one_class("Bow");
    }

    #[test]
    fn a_slingshot_deal_gives_the_slingshot_with_its_bag_and_seeds_or_the_next_bag() {
        one_class("Slingshot");
    }

    #[test]
    fn a_bomb_bag_deal_gives_the_bag_with_its_bombs_or_the_next_bag() {
        one_class("BombBag");
    }

    #[test]
    fn a_magic_deal_gives_the_first_bar_or_doubles_it() {
        one_class("Magic");
    }

    #[test]
    fn every_upgrade_deal_gives_its_base_or_the_next_tier() {
        for f in ["Hookshot", "Strength", "Wallet", "Scale", "NutBag", "StickBag", "Ocarina"] {
            assert!(!gear(f));
            one_class(f);
        }
        assert!(GEAR.iter().all(|f| by_family(f).is_some()));
        assert_eq!(HOOKS.len(), crate::progressive::FAMILIES.len());
    }

    #[test]
    fn the_hook_replaces_both_grants_and_keeps_the_rest_of_the_flow() {
        let before: Flow = fake();
        let all: [&str; 4] = ["Item_BombBag_Small", "Item_FairyBless_MP", "Weapon_Range_Bow", "Weapon_Range_SlingShot"];
        let f: Flow = hooked(&all);
        assert!(audit(&before, &f).is_empty(), "{:?}", audit(&before, &f));
        for (g, name, _) in GRANTS {
            assert_eq!(f.events[g].name, name);
            assert!(is_rung(&f, &f.events[g].kind).is_some());
        }
        assert_eq!(f.entries, before.entries);
        assert_eq!(f.actors[1].queries, vec![ASK.to_string(), OWNED.to_string()]);
        for h in HOOKS.iter().filter(|h| all.contains(&h.name)) {
            every_state_runs_as_expected(&f, h);
        }
        for gi in 0..GRANTS.len() {
            assert_eq!(run_for(&f, gi, "Weapon_Shield_Deku", &[]).unwrap(), vec![GIVE.to_string()]);
            assert_eq!(run_for(&f, gi, "Item_Hookshot", &[]).unwrap(), vec![GIVE.to_string()]);
        }
        let mut again: Flow = f.clone();
        assert!(apply(&mut again, &[&HOOKS[0]]).is_err());
        let raw: Vec<u8> = Bf { ver: [0, 4, 0, 0], bom: 0xFEFF, shift: 3, pad: 0, name: "DisplayShop".to_string(), flows: vec![fake()], tls: Vec::new() }.write().unwrap();
        let want: BS<String> = all.iter().map(|s| s.to_string()).collect();
        let out: Vec<u8> = edit(&raw, &want).unwrap();
        assert_eq!(names(&out).unwrap(), want);
        assert!(names(&raw).unwrap().is_empty());
        assert!(check(&out, &want).is_empty(), "{:?}", check(&out, &want));
        assert!(!check(&raw, &want).is_empty());
    }

    #[test]
    fn a_magic_deal_adds_the_first_gift_itself_and_hands_it_to_a_player_who_only_holds_double_magic() {
        let h: &Hook = by_family("Magic").unwrap();
        assert_eq!(h.base.first(), Some(&Step::Get("Item_FairyBless_MP")));
        let f: Flow = hooked(&[h.name]);
        for gi in 0..GRANTS.len() {
            assert_eq!(run_for(&f, gi, h.name, &["Item_FairyBless_MPDouble"]).unwrap(), expected(h, 0));
        }
        let b: &Hook = by_family("BombBag").unwrap();
        assert_eq!(b.base.first(), Some(&Step::Get("Item_BombBag_Small")));
    }

    #[test]
    fn a_magic_deal_never_hands_out_double_magic_to_a_player_without_magic() {
        let h: &Hook = by_family("Magic").unwrap();
        let f: Flow = hooked(&[h.name]);
        for gi in 0..GRANTS.len() {
            let first: Vec<String> = run_for(&f, gi, h.name, &[]).unwrap();
            assert!(first.iter().all(|l| !l.contains("MPDouble") && !l.contains("GaugeDouble")), "{first:?}");
            let second: Vec<String> = run_for(&f, gi, h.name, &["Item_FairyBless_MP"]).unwrap();
            assert!(!second.contains(&GIVE.to_string()));
            assert!(second[0].contains("Actor=Item_FairyBless_MPDouble"), "{second:?}");
        }
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
        assert_eq!(shelf_item("Item_Longshot"), "Item_Hookshot");
        assert_eq!(shelf_item("Item_Gauntlet_Gold"), "Item_Gauntlet_Silver");
        assert_eq!(shelf_item("Item_Gauntlet_Goron"), "Item_Gauntlet_Silver");
        assert_eq!(shelf_item("Item_Boomerang"), "Item_Boomerang");
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
        let before: Flow = Bf::parse(&raw).unwrap().flows.remove(0);
        let f: Flow = Bf::parse(&out).unwrap().flows.remove(0);
        assert_eq!(f.entries, before.entries);
        for h in &HOOKS {
            assert_eq!(rendered(&f, h).replace("\r\n", "\n"), golden(h).replace("\r\n", "\n"));
        }
        let one: Vec<u8> = edit(&raw, &[HOOKS[2].name.to_string()].into_iter().collect()).unwrap();
        assert_eq!(names(&one).unwrap().len(), 1);
    }

    #[test]
    fn the_steps_copy_the_chest_pre_grant_counts_and_the_great_fairy_gauges() {
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
            for g in pregrant(h.name) {
                let n: i32 = adds.iter().find(|(a, _)| a == g).map(|x| x.1).unwrap();
                assert!(h.base.contains(&Step::Add(g, n)) || (n == 1 && h.base.contains(&Step::Get(g))), "{} {g} {n}", h.family);
            }
        }
        let gi: Flow = Bf::parse(&rf.read_zs("Event/EventFlow/Dm_SY_GetInEvent.bfevfl.zs").unwrap()).unwrap().flows.remove(0);
        let walk = |from: &str, most: usize| -> Vec<String> {
            let mut at: u16 = gi.events.iter().position(|e| e.name == from).unwrap() as u16;
            let mut got: Vec<String> = Vec::new();
            while let Some(Event { kind: Kind::Action { next, actor, act, params }, .. }) = gi.events.get(at as usize) {
                let (who, what) = action_name(&gi, *actor, *act).unwrap();
                if what.contains("ActionGuide") || got.len() == most {
                    break;
                }
                got.push(line(who, what, params));
                at = *next;
            }
            got
        };
        let magic: &Hook = by_family("Magic").unwrap();
        let double: Vec<String> = step_lines(magic.ups[0]).into_iter().skip(1).collect();
        assert_eq!(walk("Event48", 5), double);
        let single: Vec<String> = walk("Event115", 3).into_iter().chain(walk("Event116", 2)).collect();
        assert_eq!(single, step_lines(&magic.base[1..]));
    }
}
