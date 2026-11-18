use super::flows::{cont, finish, flow, marked, push, query, DATA};
use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Event, Flow, Kind, Val};
use crate::error::{Error, Res};

pub const CATCH_FILE: &str = "Event/EventFlow/Dm_SY_InsertBottle.bfevfl.zs";
pub const CATCH_FLOW: &str = "Dm_SY_InsertBottle";
pub const CATCH_ENTRIES: [&str; 1] = ["EntryPoint0"];
pub const GIVE_FILE: &str = "Event/EventFlow/Dm_SY_GetInEvent.bfevfl.zs";
pub const GIVE_FLOW: &str = "Dm_SY_GetInEvent";
pub const GIVE_ENTRIES: [&str; 5] = ["GetInEventInBottle", "GetInEventInEquipedBottle", "GetInEventInBottle_2DShop", "GetInEventInBottle_FindOutCucco", "GetInEventInBottle_BlueBottleShop"];
pub const SHOP_FILE: &str = "Event/EventFlow/DisplayShop.bfevfl.zs";
pub const SHOP_FLOW: &str = "DisplayShop";

pub const RED_POTION: i32 = 0;
pub const GREEN_POTION: i32 = 1;
pub const BLUE_POTION: i32 = 2;
pub const BLUE_FIRE: i32 = 3;
pub const FISH: i32 = 4;
pub const BIG_POE: i32 = 6;
pub const POE: i32 = 7;
pub const BUG: i32 = 8;
pub const FAIRY: i32 = 9;
pub const MILK: i32 = 10;
pub const HALF_MILK: i32 = 11;

pub const SELLERS: [(&str, &str, i32); 2] =
    [("Event/EventFlow/Minigame_FindOutCucco.bfevfl.zs", "Minigame_FindOutCucco", MILK), ("Event/EventFlow/Npc_Hylia_053_Adult.bfevfl.zs", "Npc_Hylia_053_Adult", BLUE_POTION)];
pub const GOODS: [(&str, i32); 11] = [
    ("Item_Bottle_RedPotion", RED_POTION),
    ("Item_Bottle_GreenPotion", GREEN_POTION),
    ("Item_Bottle_BluePotion", BLUE_POTION),
    ("Item_Bottle_BlueFire", BLUE_FIRE),
    ("Item_Bottle_Fish", FISH),
    ("Item_Bottle_BigPoe", BIG_POE),
    ("Item_Bottle_Poe", POE),
    ("Item_Bottle_Bug", BUG),
    ("Item_Bottle_Fairy", FAIRY),
    ("Item_Bottle_Milk", MILK),
    ("Item_Bottle_HalfMilk", HALF_MILK),
];

pub const POUCH: &str = "PouchSystemActor";
pub const HOLDS: &str = "EventQueryIsExistBottleTypeInPouch";
pub const ANY_EMPTY: &str = "EventQueryIsAnyBottleEmpty";
pub const IS_TEXT: &str = "EventQueryIsGameDataString64";
pub const PICKED: &str = "Shop_SelectItemName";
pub const ITEM: &str = "BottleItem";
pub const YES: u32 = 0;
pub const NO: u32 = 1;
pub const MARK: &str = "OOTRRNoDupe";

pub type Good<'a> = (Option<&'a str>, i32);

pub fn goods() -> Vec<Good<'static>> {
    GOODS.iter().map(|(n, i)| (Some(*n), *i)).collect()
}

fn passes(f: &Flow) -> bool {
    f.events.iter().any(|e| e.kind.params().is_some_and(|c| c.items.iter().any(|(_, v)| *v == Val::Arg(ITEM.to_string()))))
}

pub fn entries(raw: &[u8], name: &str, list: &[&str]) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    let f: &mut Flow = flow(&mut b, name)?;
    if marked(f, MARK) {
        return Err(Error::Format("difficulty: a bottle filling flow is already edited"));
    }
    if !passes(f) {
        return Err(Error::Msg(format!("difficulty: the flow {name} never passes on which bottle content it fills in")));
    }
    let (pa, pq): (u16, u16) = query(f, POUCH, HOLDS)?;
    for want in list {
        let i: usize = f.entries.iter().position(|e| e.name == *want).ok_or_else(|| Error::Msg(format!("difficulty: the flow {name} has no entry {want}")))?;
        let main: u16 = f.entries[i].main;
        if main == NONE {
            return Err(Error::Msg(format!("difficulty: the entry {want} of {name} is empty")));
        }
        let gate: u16 = push(f, format!("{MARK}Entry{i}"), Kind::Switch { actor: pa, query: pq, cases: vec![Case { val: NO, ev: main }], params: cont(vec![(ITEM, Val::Arg(ITEM.to_string()))]) })?;
        f.entries[i].main = gate;
    }
    finish(&b)
}

fn empty_check(f: &Flow, e: &Event) -> Option<(u16, u16)> {
    let Kind::Switch { actor, query, cases, .. } = &e.kind else {
        return None;
    };
    if !f.actors.get(*actor as usize).is_some_and(|a| a.name == POUCH && a.queries.get(*query as usize).is_some_and(|s| s == ANY_EMPTY)) {
        return None;
    }
    let free: u16 = cases.iter().find(|c| c.val == YES)?.ev;
    let held: u16 = cases.iter().find(|c| c.val == NO)?.ev;
    Some((free, held))
}

pub fn fronts(raw: &[u8], name: &str, goods: &[Good]) -> Res<Vec<u8>> {
    let mut b: Bf = Bf::parse(raw)?;
    let f: &mut Flow = flow(&mut b, name)?;
    if marked(f, MARK) {
        return Err(Error::Format("difficulty: a bottle seller flow is already edited"));
    }
    let at: Vec<(usize, u16, u16)> = (0..f.events.len()).filter_map(|i| empty_check(f, &f.events[i]).map(|(free, held)| (i, free, held))).collect();
    if at.is_empty() || goods.is_empty() {
        return Err(Error::Msg(format!("difficulty: the flow {name} never asks for an empty bottle")));
    }
    let (pa, pq): (u16, u16) = query(f, POUCH, HOLDS)?;
    for (i, free, held) in at {
        let mut next: u16 = free;
        for (j, (good, item)) in goods.iter().enumerate().rev() {
            let has: u16 = push(
                f,
                format!("{MARK}Has{i}_{j}"),
                Kind::Switch { actor: pa, query: pq, cases: vec![Case { val: YES, ev: held }, Case { val: NO, ev: free }], params: cont(vec![(ITEM, Val::Int(*item))]) },
            )?;
            next = match good {
                None => has,
                Some(n) => {
                    let (da, dq): (u16, u16) = query(f, DATA, IS_TEXT)?;
                    push(
                        f,
                        format!("{MARK}Is{i}_{j}"),
                        Kind::Switch {
                            actor: da,
                            query: dq,
                            cases: vec![Case { val: YES, ev: has }, Case { val: NO, ev: next }],
                            params: cont(vec![("GameDataName", Val::Str(PICKED.to_string())), ("Index", Val::Int(-1)), ("Value", Val::Str(n.to_string()))]),
                        },
                    )?
                }
            };
        }
        if let Kind::Switch { cases, .. } = &mut f.events[i].kind {
            for c in cases.iter_mut().filter(|c| c.val == YES) {
                c.ev = next;
            }
        }
    }
    finish(&b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfevfl::{Actor, Entry};

    fn sys(name: &str, actions: &[&str], queries: &[&str]) -> Actor {
        Actor {
            name: name.to_string(),
            sub: String::new(),
            arg: String::new(),
            actions: actions.iter().map(|s| s.to_string()).collect(),
            queries: queries.iter().map(|s| s.to_string()).collect(),
            params: None,
            ep: 0xFFFF,
            cut: 1,
        }
    }

    fn file(name: &str, actors: Vec<Actor>, events: Vec<Event>, mains: &[(&str, u16)]) -> Vec<u8> {
        let entries: Vec<Entry> = mains.iter().map(|(n, m)| Entry { name: n.to_string(), subs: Vec::new(), vars: Vec::new(), main: *m, pad: 0 }).collect();
        Bf { ver: [0, 4, 0, 0], bom: 0xFEFF, shift: 3, pad: 0, name: name.to_string(), flows: vec![Flow { name: name.to_string(), actors, events, entries }], tls: Vec::new() }.write().unwrap()
    }

    fn act(name: &str, next: u16, items: Vec<(&str, Val)>) -> Event {
        Event { name: name.to_string(), kind: Kind::Action { next, actor: 0, act: 0, params: cont(items) } }
    }

    fn switch(f: &Flow, i: u16) -> (&str, &str, &Vec<Case>, Option<&Val>, Option<&Val>) {
        let Kind::Switch { actor, query, cases, params } = &f.events[i as usize].kind else { panic!("event {i} is not a switch") };
        let a: &Actor = &f.actors[*actor as usize];
        (a.name.as_str(), a.queries[*query as usize].as_str(), cases, params.as_ref().and_then(|c| c.get(ITEM)), params.as_ref().and_then(|c| c.get("Value")))
    }

    #[test]
    fn a_filling_entry_only_runs_when_the_content_is_not_held() {
        let push_it: Event = act("Push", NONE, vec![("Item", Val::Arg(ITEM.into()))]);
        let raw: Vec<u8> = file("Fill", vec![sys(POUCH, &["EventTriggerPushToEmptyBottle"], &[])], vec![push_it, act("Other", NONE, vec![])], &[("In", 0), ("Plain", 1)]);
        let out: Vec<u8> = entries(&raw, "Fill", &["In"]).unwrap();
        let b: Bf = Bf::parse(&out).unwrap();
        let f: &Flow = &b.flows[0];
        assert_eq!(f.events.len(), 3);
        assert_eq!((f.entries[0].main, f.entries[1].main), (2, 1), "an entry that fills no bottle is left alone");
        let (actor, q, cases, item, _) = switch(f, 2);
        assert_eq!((actor, q), (POUCH, HOLDS));
        assert_eq!(cases, &vec![Case { val: NO, ev: 0 }], "holding the content ends the event");
        assert_eq!(item, Some(&Val::Arg(ITEM.into())));
        assert!(entries(&out, "Fill", &["In"]).is_err());
        assert!(entries(&raw, "Fill", &["Missing"]).is_err());
        assert!(entries(&raw, "Other", &["In"]).is_err());
        let blind: Vec<u8> = file("Fill", vec![sys(POUCH, &["Use"], &[])], vec![act("Other", NONE, vec![])], &[("In", 0)]);
        assert!(entries(&blind, "Fill", &["In"]).is_err());
    }

    fn asks(free: u16, held: u16) -> Event {
        Event { name: "Ask".into(), kind: Kind::Switch { actor: 0, query: 0, cases: vec![Case { val: YES, ev: free }, Case { val: NO, ev: held }], params: None } }
    }

    #[test]
    fn a_seller_treats_held_content_like_no_empty_bottle() {
        let raw: Vec<u8> = file("Seller", vec![sys(POUCH, &[], &[ANY_EMPTY])], vec![act("Sell", NONE, vec![]), act("Refuse", NONE, vec![]), asks(0, 1)], &[("Talk", 2)]);
        let out: Vec<u8> = fronts(&raw, "Seller", &[(None, MILK)]).unwrap();
        let b: Bf = Bf::parse(&out).unwrap();
        let f: &Flow = &b.flows[0];
        assert_eq!(f.events.len(), 4);
        assert_eq!(f.entries[0].main, 2);
        let (_, q, cases, _, _) = switch(f, 2);
        assert_eq!(q, ANY_EMPTY);
        assert_eq!(cases, &vec![Case { val: YES, ev: 3 }, Case { val: NO, ev: 1 }]);
        let (actor, q, cases, item, _) = switch(f, 3);
        assert_eq!((actor, q), (POUCH, HOLDS));
        assert_eq!(cases, &vec![Case { val: YES, ev: 1 }, Case { val: NO, ev: 0 }]);
        assert_eq!(item, Some(&Val::Int(MILK)));
        assert!(fronts(&out, "Seller", &[(None, MILK)]).is_err());
        let plain: Vec<u8> = file("Seller", vec![sys(POUCH, &["Use"], &[])], vec![act("Sell", NONE, vec![])], &[("Talk", 0)]);
        assert!(fronts(&plain, "Seller", &[(None, MILK)]).is_err());
    }

    #[test]
    fn a_shop_checks_the_picked_good_by_name() {
        let raw: Vec<u8> = file("Shop", vec![sys(POUCH, &[], &[ANY_EMPTY])], vec![act("Sell", NONE, vec![]), act("Refuse", NONE, vec![]), asks(0, 1)], &[("Talk", 2)]);
        let b: Bf = Bf::parse(&fronts(&raw, "Shop", &goods()).unwrap()).unwrap();
        let f: &Flow = &b.flows[0];
        assert_eq!(f.events.len(), 3 + 2 * GOODS.len());
        assert_eq!(f.actors.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec![POUCH, DATA]);
        let (_, _, cases, _, _) = switch(f, 2);
        let mut at: u16 = cases[0].ev;
        for (name, item) in GOODS {
            let (actor, q, cases, _, value) = switch(f, at);
            assert_eq!((actor, q), (DATA, IS_TEXT));
            assert_eq!(value, Some(&Val::Str(name.to_string())));
            let (has, next): (u16, u16) = (cases[0].ev, cases[1].ev);
            assert_eq!((cases[0].val, cases[1].val), (YES, NO));
            let (actor, q, cases, held, _) = switch(f, has);
            assert_eq!((actor, q), (POUCH, HOLDS));
            assert_eq!(held, Some(&Val::Int(item)));
            assert_eq!(cases, &vec![Case { val: YES, ev: 1 }, Case { val: NO, ev: 0 }]);
            at = next;
        }
        assert_eq!(at, 0, "a good that is no bottle content is sold as before");
    }

    #[test]
    fn the_contents_are_distinct_and_leave_out_the_letter() {
        let mut ids: Vec<i32> = GOODS.iter().map(|(_, i)| *i).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), GOODS.len());
        assert!(!ids.contains(&5) && !ids.contains(&-1));
        assert!(GOODS.iter().all(|(n, _)| n.starts_with("Item_Bottle_") && *n != "Item_Bottle_Letter"));
    }
}
