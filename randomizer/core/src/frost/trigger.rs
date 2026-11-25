use crate::ainb::build::Spec;
use crate::ainb::graft::{root, set_root, SIMULTANEOUS};
use crate::ainb::Ainb;
use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Cont, Entry, Event, Flow, Kind, Val};
use crate::byml::{Byml, Node};
use crate::dungeon_credit::{action_slot, idx, push, set_bool, HOLDER, SET};
use crate::error::{Error as E, Res};

pub const FLAG: &str = "OOTRR.ColdPending";
pub const EVENT: &str = "OOTRR_ColdFreeze";
pub const GET_FILE: &str = "Event/EventFlow/GetPouchContent.bfevfl.zs";
pub const GET_FLOW: &str = "GetPouchContent";
pub const GIVE_FILE: &str = "Event/EventFlow/Dm_SY_GetInEvent.bfevfl.zs";
pub const GIVE_FLOW: &str = "Dm_SY_GetInEvent";
pub const FLOWCHART: &str = "Work/Event/EventFlow/Dm_SY_GetInEvent.evfc";
pub const MAIN: &str = "Sequence/MainUpdate.module.ainb";
const ADD: &str = "EventTriggerAddOrRemovePouchContent";
const SET_NAME: &str = "EventTriggerSetOwnNameToMessage";
pub const PICKUPS: [&str; 2] = ["Dm_SY_Get", "Dm_SY_Get_WithoutPlayer"];
pub const SHOP_FILE: &str = "Event/EventFlow/DisplayShop.bfevfl.zs";
const SHOP_ACTOR: &str = "Npc_EventStarter";
const SHOP_GRANT: &str = "EventTriggerGetItemAtShop";
const ON_SALE: &str = "Shop_SelectItemName";
const ASK: &str = "EventQueryIsGameDataString64";
const SET_TEXT: &str = "EventTriggerSetGameDataString64";
const TARGET: &str = "UI_MessageTargetActorName";
const CREATE: &str = "EventTriggerRequestCreateConditionSensor";
const DELETE: &str = "EventTriggerRequestDeleteConditionSensor";
const POUCH: &str = "PouchSystemActor";
const EXISTS: &str = "EventQueryIsExistInPouch";
const GAME: &str = "GameSystemActor";
const WAITER: &str = "EventSystemActor";
const WAIT: &str = "EventWait";
const SETTING_TEMPLATE: &str = "CrumblingGanonCastle_Arrive";
const ANALYZED_TEMPLATE: &str = "NaviLockOnHint";
const SUBS: &str = "SubFlowList";
const KIDS: &str = "ChildEventList";
const S32: usize = 0;
const BOOL: usize = 1;
const STR: usize = 3;
const SELECTOR: u16 = 7;

fn ask(key: &str, name: &str) -> Cont {
    Cont { items: vec![("GameDataName".to_string(), Val::Str(key.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(name.to_string()))] }
}

fn chain(f: &mut Flow, sites: &[usize], key: &str, names: &[String]) -> Res<()> {
    let (qa, qt): (u16, u16) = crate::start_state::query_slot(f, HOLDER, ASK)?;
    let (sa, ss): (u16, u16) = action_slot(f, HOLDER, SET)?;
    for at in sites {
        let then: u16 = match &f.events[*at].kind {
            Kind::Action { next, .. } => *next,
            _ => continue,
        };
        let mark: u16 = push(f, Kind::Action { next: then, actor: sa, act: ss, params: Some(set_bool(FLAG.to_string())) })?;
        let mut next: u16 = then;
        for n in names.iter().rev() {
            next = push(f, Kind::Switch { actor: qa, query: qt, cases: vec![Case { val: 0, ev: mark }, Case { val: 1, ev: next }], params: Some(ask(key, n)) })?;
        }
        if let Kind::Action { next: n, .. } = &mut f.events[*at].kind {
            *n = next;
        }
    }
    Ok(())
}

pub fn mark_after(f: &mut Flow, sites: &[usize]) -> Res<usize> {
    let (sa, ss): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let mut n: usize = 0;
    for at in sites {
        let then: u16 = match &f.events[*at].kind {
            Kind::Action { next, .. } | Kind::Sub { next, .. } => *next,
            _ => continue,
        };
        let mark: u16 = push(f, Kind::Action { next: then, actor: sa, act: ss, params: Some(set_bool(FLAG.to_string())) })?;
        match &mut f.events[*at].kind {
            Kind::Action { next, .. } | Kind::Sub { next, .. } => *next = mark,
            _ => {}
        }
        n += 1;
    }
    Ok(n)
}

pub fn set_name(f: &mut Flow, at: usize, event: &str, param: &str, fake: &str) -> Res<()> {
    let e: &mut Event = f.events.get_mut(at).ok_or_else(|| E::Msg(format!("event index {at} is out of range")))?;
    if e.name != event {
        return Err(E::Msg(format!("event {at} is named {} and not {event}, the game files differ", e.name)));
    }
    let c: &mut Cont = e.kind.params_mut().ok_or_else(|| E::Msg(format!("event {event} has no parameters")))?;
    for (k, v) in c.items.iter_mut() {
        if k == param {
            if let Val::Str(s) | Val::Actor(s, _) = v {
                *s = fake.to_string();
                return Ok(());
            }
        }
    }
    Err(E::Msg(format!("{event} has no name parameter {param}")))
}

fn sites_of(f: &Flow, action: &str, named: bool) -> Vec<usize> {
    f.events
        .iter()
        .enumerate()
        .filter(|(_, e)| match &e.kind {
            Kind::Action { actor, act, params, .. } => {
                f.actors.get(*actor as usize).and_then(|a| a.actions.get(*act as usize)).is_some_and(|x| x == action)
                    && (!named || params.as_ref().is_some_and(|c| matches!(c.get("IsSetMessageTargetActorName"), Some(Val::Bool(v)) if *v != 0)))
            }
            _ => false,
        })
        .map(|(i, _)| i)
        .collect()
}

pub fn hook_get(b: &mut Bf, names: &[String]) -> Res<usize> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == GET_FLOW).ok_or(E::Format("the get flow has no flow of its name"))?;
    let sites: Vec<usize> = sites_of(f, ADD, true);
    if sites.is_empty() {
        return Err(E::Format("the get flow has no pouch add that names its actor"));
    }
    chain(f, &sites, TARGET, names)?;
    Ok(sites.len())
}

pub fn hook_give(b: &mut Bf, names: &[String]) -> Res<usize> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == GIVE_FLOW).ok_or(E::Format("the scene get flow has no flow of its name"))?;
    let sites: Vec<usize> = sites_of(f, ADD, true);
    chain(f, &sites, TARGET, names)?;
    Ok(sites.len())
}

fn is_wait(f: &Flow, at: u16) -> bool {
    match f.events.get(usize::from(at)).map(|e| &e.kind) {
        Some(Kind::Action { actor, act, .. }) => f.actors.get(usize::from(*actor)).is_some_and(|a| a.name == WAITER && a.actions.get(usize::from(*act)).is_some_and(|x| x == WAIT)),
        _ => false,
    }
}

/// The event after which the picked name can be asked: the pickup names itself with an
/// EventTrigger action, which the game only carries out some frames later, so a question asked
/// right behind it still reads the name of the pickup before. Returns the wait that follows the
/// naming event (crate::pickup_tiers already puts one there), or a new one.
fn settled(f: &mut Flow, at: usize) -> Res<usize> {
    let then: u16 = match &f.events[at].kind {
        Kind::Action { next, .. } => *next,
        _ => return Err(E::Format("the naming event of a pickup is not an action")),
    };
    if is_wait(f, then) {
        return Ok(usize::from(then));
    }
    let (wa, wx): (u16, u16) = action_slot(f, WAITER, WAIT)?;
    let c: Cont = Cont { items: vec![("IsWaitEnd".to_string(), Val::Bool(1)), ("Frames".to_string(), Val::Int(crate::pickup_tiers::SETTLE))] };
    let wait: u16 = push(f, Kind::Action { next: then, actor: wa, act: wx, params: Some(c) })?;
    if let Kind::Action { next, .. } = &mut f.events[at].kind {
        *next = wait;
    }
    Ok(usize::from(wait))
}

/// Whether every question about the picked name that directly follows an event is behind a wait,
/// never right behind the event that sets the name.
pub fn asks_wait(f: &Flow) -> bool {
    let asks = |at: u16| -> bool {
        match f.events.get(usize::from(at)).map(|e| &e.kind) {
            Some(Kind::Switch { actor, query, params, .. }) => {
                f.actors.get(usize::from(*actor)).is_some_and(|a| a.queries.get(usize::from(*query)).is_some_and(|q| q == ASK)) && params.as_ref().is_some_and(|c| matches!(c.get("GameDataName"), Some(Val::Str(k)) if k == TARGET))
            }
            _ => false,
        }
    };
    sites_of(f, SET_NAME, false).into_iter().all(|i| match &f.events[i].kind {
        Kind::Action { next, .. } => !asks(*next),
        _ => true,
    })
}

pub fn hook_pickup(b: &mut Bf, flow: &str, names: &[String]) -> Res<usize> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == flow).ok_or_else(|| E::Msg(format!("{flow} has no flow of its name")))?;
    hook_named(f, names)
}

pub fn hook_named(f: &mut Flow, names: &[String]) -> Res<usize> {
    // crate::pickup_tiers names the pickup a second time on its way to the window; by then the
    // first naming has been asked about, so only the game's own naming events are hooked.
    let named: Vec<usize> = sites_of(f, SET_NAME, false).into_iter().filter(|i| !f.events[*i].name.starts_with(crate::pickup_tiers::PREFIX)).collect();
    if named.is_empty() {
        return Err(E::Msg(format!("{} has no pickup that names itself", f.name)));
    }
    let mut sites: Vec<usize> = Vec::new();
    for at in &named {
        sites.push(settled(f, *at)?);
    }
    chain(f, &sites, TARGET, names)?;
    Ok(sites.len())
}

pub fn hook_shop(b: &mut Bf, names: &[String]) -> Res<usize> {
    let f: &mut Flow = b.flows.first_mut().ok_or(E::Format("the shop flow file holds no flow"))?;
    let sites: Vec<usize> = f
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| match &e.kind {
            Kind::Action { actor, act, .. } => f.actors.get(*actor as usize).is_some_and(|a| a.name == SHOP_ACTOR && a.actions.get(*act as usize).is_some_and(|x| x == SHOP_GRANT)),
            _ => false,
        })
        .map(|(i, _)| i)
        .collect();
    if sites.is_empty() {
        return Err(E::Format("the shop flow has no purchase grant"));
    }
    chain(f, &sites, ON_SALE, names)?;
    Ok(sites.len())
}

pub fn add_event(b: &mut Bf, fakes: &[String]) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == GIVE_FLOW).ok_or(E::Format("the give flow has no flow of its name"))?;
    if f.entries.iter().any(|e| e.name == EVENT) {
        return Ok(());
    }
    let (g, mk): (u16, u16) = action_slot(f, GAME, CREATE)?;
    let (_, rm): (u16, u16) = action_slot(f, GAME, DELETE)?;
    let (pa, take): (u16, u16) = action_slot(f, POUCH, ADD)?;
    let (qa, has): (u16, u16) = crate::start_state::query_slot(f, POUCH, EXISTS)?;
    let clear: u16 = idx(f.events.len())?;
    f.events.push(Event { name: format!("{EVENT}_Clear"), kind: Kind::Action { next: clear + 1, actor: g, act: rm, params: None } });
    f.events.push(Event { name: format!("{EVENT}_Make"), kind: Kind::Action { next: NONE, actor: g, act: mk, params: Some(Cont { items: vec![("ActorName".to_string(), Val::Str(crate::frost::sensor::NAME.to_string()))] }) } });
    let mut next: u16 = clear;
    for n in fakes.iter().rev() {
        let out: Cont = Cont {
            items: vec![
                ("Actor".to_string(), Val::Str(n.clone())),
                ("Num".to_string(), Val::Int(-1)),
                ("IsSetMessageTargetActorName".to_string(), Val::Bool(0)),
                ("IsEquipDIrect".to_string(), Val::Bool(0)),
                ("IsGetEvent".to_string(), Val::Bool(0)),
            ],
        };
        let gone: u16 = push(f, Kind::Action { next, actor: pa, act: take, params: Some(out) })?;
        let ask: Cont = Cont { items: vec![("Actor".to_string(), Val::Str(n.clone())), ("Num".to_string(), Val::Int(1))] };
        next = push(f, Kind::Switch { actor: qa, query: has, cases: vec![Case { val: 0, ev: gone }, Case { val: 1, ev: next }], params: Some(ask) })?;
    }
    let (ha, st): (u16, u16) = action_slot(f, HOLDER, SET_TEXT)?;
    let blank: Cont = Cont { items: vec![("GameDataName".to_string(), Val::Str(TARGET.to_string())), ("Index".to_string(), Val::Int(-1)), ("Value".to_string(), Val::Str(String::new()))] };
    let next: u16 = push(f, Kind::Action { next, actor: ha, act: st, params: Some(blank) })?;
    f.entries.push(Entry { name: EVENT.to_string(), subs: Vec::new(), vars: Vec::new(), main: next, pad: 0 });
    Ok(())
}

fn text<'a>(n: &'a Node, k: &str) -> &'a str {
    n.get(k).and_then(|v| v.as_str()).unwrap_or("")
}

fn row(y: &Byml, id: &str) -> Res<Node> {
    y.root.as_arr().and_then(|a| a.iter().find(|r| text(r, "__RowId") == id)).cloned().ok_or_else(|| E::Msg(format!("the table has no {id} row")))
}

fn insert(y: &mut Byml, r: Node) -> Res<()> {
    let id: String = text(&r, "__RowId").to_string();
    let list: &mut Vec<Node> = y.root.as_arr_mut().ok_or(E::Format("the table root is not a list"))?;
    if list.iter().any(|x| text(x, "__RowId") == id) {
        return Ok(());
    }
    let at: usize = list.partition_point(|x| text(x, "__RowId") < id.as_str());
    list.insert(at, r);
    Ok(())
}

pub fn setting(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let mut n: Node = row(&y, SETTING_TEMPLATE)?;
    n.set("__RowId", Node::Str(EVENT.to_string()));
    n.set("EntryPointName", Node::Str(EVENT.to_string()));
    n.set("FlowchartFileName", Node::Str(FLOWCHART.to_string()));
    insert(&mut y, n)?;
    y.write()
}

pub fn analyzed(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let mut n: Node = row(&y, ANALYZED_TEMPLATE)?;
    n.set("__RowId", Node::Str(EVENT.to_string()));
    n.set(SUBS, Node::Arr(Vec::new()));
    n.remove(KIDS);
    insert(&mut y, n)?;
    y.write()
}

fn s32(v: i32) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

fn b(v: bool) -> Vec<u8> {
    (v as u32).to_le_bytes().to_vec()
}

fn simultaneous<'a>(seed: &'a str) -> Spec<'a> {
    Spec::composite(SIMULTANEOUS, seed).prop(S32, "EndPolicy", s32(0)).prop(S32, "ResultPolicy", s32(0))
}

pub fn watch(raw: &[u8]) -> Res<Vec<u8>> {
    let mut a: Ainb = Ainb::parse(raw)?;
    let old: usize = root(&a).ok_or(E::Format("MainUpdate has no root"))?;
    let flag: Vec<u8> = a.pool.add(FLAG).to_le_bytes().to_vec();
    let ev: Vec<u8> = a.pool.add(EVENT).to_le_bytes().to_vec();
    let q: usize = a.add_node(
        &Spec::query("SeqQueryGetNonStructGameDataBool", "ootrr-cold#query")
            .idle(S32, "GameDataIndex", s32(-1))
            .constant(BOOL, "IsDirect", b(true))
            .constant(STR, "GameDataFullName", flag.clone())
            .out(BOOL, "Value"),
    )?;
    let req: usize = a.add_node(&Spec::node("SeqOneShotRequestEvent", "ootrr-cold#request").prop(BOOL, "ExecuteEveryFrame", b(false)).idle(BOOL, "IsBancParamValue", b(false)).constant(STR, "EventName", ev))?;
    let clear: usize = a.add_node(
        &Spec::node("SeqOneShotSetNonStructGameDataBool", "ootrr-cold#clear")
            .prop(BOOL, "ExecuteEveryFrame", b(false))
            .idle(S32, "GameDataIndex", s32(-1))
            .constant(BOOL, "Value", b(false))
            .constant(STR, "GameDataFullName", flag),
    )?;
    let idle: usize = a.add_node(&Spec::node("SeqExecuteGenericDummyAction", "ootrr-cold#idle"))?;
    let both: usize = a.add_node(&simultaneous("ootrr-cold#both").kid(req, "").kid(clear, ""))?;
    let mut sel: Spec = Spec::composite(SELECTOR, "ootrr-cold#select")
        .prop(S32, "CalculateTiming", s32(0))
        .prop(BOOL, "ChildFrameSync", b(false))
        .prop(BOOL, "InputValue", b(false))
        .prop(BOOL, "IsNoSelectWhenChildBusy", b(false))
        .linked(BOOL, "Input", q, 0)
        .kid(both, "True")
        .kid(idle, "False");
    sel.link_extra = 8;
    let s: usize = a.add_node(&sel)?;
    let top: usize = a.add_node(&simultaneous("ootrr-cold#root").kid(old, "").kid(s, ""))?;
    set_root(&mut a, top)?;
    let bad: Vec<String> = crate::ainb::audit(&a);
    if !bad.is_empty() {
        return Err(E::Msg(format!("the edited MainUpdate has problems: {}", bad.join("; "))));
    }
    a.write()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfevfl::Actor;

    const GUEST: &str = "PouchContentGuestPerformer";

    fn actor(name: &str, acts: &[&str], queries: &[&str]) -> Actor {
        Actor { name: name.to_string(), sub: String::new(), arg: String::new(), actions: acts.iter().map(|s| s.to_string()).collect(), queries: queries.iter().map(|s| s.to_string()).collect(), params: None, ep: 0, cut: 0 }
    }

    fn act(name: &str, actor: u16, act: u16, next: u16) -> Event {
        Event { name: name.to_string(), kind: Kind::Action { next, actor, act, params: None } }
    }

    /// The game's pickup flow: the pickup names itself, then the rest (here one event).
    fn pickup() -> Flow {
        Flow { name: "Dm_SY_Get".to_string(), actors: vec![actor(GUEST, &[SET_NAME, "EventTriggerAddOwnToPouch"], &[])], events: vec![act("Event38", 0, 0, 1), act("Event35", 0, 1, NONE)], entries: Vec::new() }
    }

    fn frames(f: &Flow, at: u16) -> Option<i32> {
        match &f.events[usize::from(at)].kind {
            Kind::Action { params: Some(c), .. } if is_wait(f, at) => match c.get("Frames") {
                Some(Val::Int(n)) => Some(*n),
                _ => None,
            },
            _ => None,
        }
    }

    fn next(f: &Flow, at: u16) -> u16 {
        match &f.events[usize::from(at)].kind {
            Kind::Action { next, .. } => *next,
            _ => NONE,
        }
    }

    /// The game only carries the naming out some frames later: asked in the same frame, a trap on
    /// the ground read the name of the pickup before it, did not freeze, and left its stand-in.
    #[test]
    fn a_pickup_is_asked_for_its_name_only_after_a_wait() {
        let mut f: Flow = pickup();
        assert_eq!(hook_named(&mut f, &["OOTRR_Item_Hookshot_Cold".to_string()]).unwrap(), 1);
        let wait: u16 = next(&f, 0);
        assert_eq!(frames(&f, wait), Some(crate::pickup_tiers::SETTLE));
        let ask: u16 = next(&f, wait);
        let Kind::Switch { cases, params, .. } = &f.events[usize::from(ask)].kind else { panic!("the wait does not lead to the question") };
        assert_eq!(params.as_ref().and_then(|c| c.get("Value")), Some(&Val::Str("OOTRR_Item_Hookshot_Cold".to_string())));
        let (hit, miss): (u16, u16) = (cases[0].ev, cases[1].ev);
        assert_eq!(miss, 1);
        assert_eq!(next(&f, hit), 1);
        assert!(asks_wait(&f));
    }

    #[test]
    fn a_wait_that_is_already_there_is_used_and_a_second_naming_is_left_alone() {
        let mut f: Flow = pickup();
        f.actors.push(actor(WAITER, &[WAIT], &[]));
        f.events[0] = act("Event38", 0, 0, 2);
        f.events.push(Event { name: "OOTRR_PickTier2".to_string(), kind: Kind::Action { next: 3, actor: 1, act: 0, params: Some(Cont { items: vec![("Frames".to_string(), Val::Int(5))] }) } });
        f.events.push(act("OOTRR_PickTier3", 0, 0, 1));
        let n: usize = f.events.len();
        assert_eq!(hook_named(&mut f, &["OOTRR_Item_Hookshot_Cold".to_string()]).unwrap(), 1);
        assert_eq!(next(&f, 0), 2);
        assert!(f.events[n..].iter().all(|e| !matches!(&e.kind, Kind::Action { params: Some(c), .. } if c.get("Frames").is_some())), "a second wait was added");
        assert!(matches!(f.events[usize::from(next(&f, 2))].kind, Kind::Switch { .. }));
        assert_eq!(next(&f, 3), 1);
        assert!(asks_wait(&f));
    }
}
