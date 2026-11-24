use crate::bfevfl::{Bf, Event, Flow, Kind};
use crate::dungeon_credit::{action_slot, set_bool};
use crate::error::{Error, Res};

pub const HOLDER: &str = "GameDataActor";
pub const SET: &str = "EventTriggerSetGameDataBool";
pub const PREFIX: &str = "OOTRR_AP_";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hook {
    pub file: &'static str,
    pub flow: &'static str,
    pub after: &'static str,
}

pub const SEED: Hook = Hook { file: crate::intro_skip::FILE, flow: crate::intro_skip::FLOW, after: "" };
pub const GOAL: Hook = Hook { file: crate::ganon_trials::FLOW_FILE, flow: crate::ganon_trials::FLOW, after: "Event37" };

pub fn seed_flag(id: &str) -> String {
    format!("OOTRR.Seed.{id}")
}

pub fn goal_flag(id: &str) -> String {
    format!("OOTRR.Goal.{id}")
}

fn next_of(k: &mut Kind) -> Option<&mut u16> {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => Some(next),
        _ => None,
    }
}

pub fn mark_after(f: &mut Flow, after: &str, flag: &str) -> Res<u16> {
    let i: usize = f.events.iter().position(|e| e.name == after).ok_or_else(|| Error::Msg(format!("markers: {} has no event {after}", f.name)))?;
    let (a, set): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let at: u16 = u16::try_from(f.events.len()).map_err(|_| Error::Format("markers: the flow is full"))?;
    let slot: &mut u16 = next_of(&mut f.events[i].kind).ok_or_else(|| Error::Msg(format!("markers: {after} in {} does not lead to one next event", f.name)))?;
    let then: u16 = *slot;
    *slot = at;
    f.events.push(Event { name: format!("{PREFIX}{at}"), kind: Kind::Action { next: then, actor: a, act: set, params: Some(set_bool(flag.to_string())) } });
    Ok(at)
}

pub fn mark_entries(f: &mut Flow, flag: &str) -> Res<u16> {
    let (a, set): (u16, u16) = action_slot(f, HOLDER, SET)?;
    let mut first: Option<u16> = None;
    for k in 0..f.entries.len() {
        let at: u16 = u16::try_from(f.events.len()).map_err(|_| Error::Format("markers: the flow is full"))?;
        let then: u16 = f.entries[k].main;
        f.events.push(Event { name: format!("{PREFIX}{at}"), kind: Kind::Action { next: then, actor: a, act: set, params: Some(set_bool(flag.to_string())) } });
        f.entries[k].main = at;
        first.get_or_insert(at);
    }
    first.ok_or_else(|| Error::Msg(format!("markers: {} has no entry", f.name)))
}

pub fn patch(b: &mut Bf, h: Hook, flag: &str) -> Res<u16> {
    let f: &mut Flow = b.flows.iter_mut().find(|x| x.name == h.flow).ok_or_else(|| Error::Msg(format!("markers: {} has no {} flow", h.file, h.flow)))?;
    let at: u16 = if h.after.is_empty() { mark_entries(f, flag)? } else { mark_after(f, h.after, flag)? };
    if !reached(f, at) {
        return Err(Error::Msg(format!("markers: the {flag} mark after {} in {} is not reachable from any entry of the flow, an earlier edit routes around it", h.after, h.flow)));
    }
    Ok(at)
}

fn targets(k: &Kind) -> Vec<u16> {
    match k {
        Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => vec![*next],
        Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter().map(|c| c.ev).collect(),
        Kind::Fork { join, forks } => forks.iter().copied().chain(std::iter::once(*join)).collect(),
        Kind::Label { .. } => Vec::new(),
    }
}

pub fn reached(f: &Flow, goal: u16) -> bool {
    let mut seen: Vec<bool> = vec![false; f.events.len()];
    let mut q: Vec<u16> = f.entries.iter().map(|e| e.main).collect();
    while let Some(i) = q.pop() {
        let Some(e) = f.events.get(i as usize) else { continue };
        if seen[i as usize] {
            continue;
        }
        seen[i as usize] = true;
        if i == goal {
            return true;
        }
        q.extend(targets(&e.kind));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::romfs::Romfs;

    fn edit_raw(raw: &[u8], h: Hook, flag: &str) -> Res<Vec<u8>> {
        let mut b: Bf = Bf::parse(raw)?;
        patch(&mut b, h, flag)?;
        let out: Vec<u8> = b.write()?;
        if Bf::parse(&out)? != b {
            return Err(Error::Format("markers: the patched flow does not parse back to the edited model"));
        }
        Ok(out)
    }
    use crate::bfevfl::Val;

    fn real() -> Option<Romfs> {
        let p = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(Romfs::open(&p).unwrap())
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    fn flag_of(f: &Flow, at: u16) -> Option<String> {
        let e: &Event = &f.events[at as usize];
        let Kind::Action { actor, act, params, .. } = &e.kind else { return None };
        let a = &f.actors[*actor as usize];
        if a.name != HOLDER || a.actions[*act as usize] != SET {
            return None;
        }
        params.as_ref()?.items.iter().find(|(k, _)| k == "GameDataName").and_then(|(_, v)| if let Val::Str(s) = v { Some(s.clone()) } else { None })
    }

    #[test]
    fn both_markers_sit_right_after_their_hook_and_keep_the_rest_of_the_chain() {
        let Some(mut rf) = real() else { return };
        for (h, flag) in [(GOAL, goal_flag("0123456789ab"))] {
            let raw: Vec<u8> = rf.read_zs(h.file).unwrap();
            let before: Bf = Bf::parse(&raw).unwrap();
            let f0: &Flow = before.flows.iter().find(|f| f.name == h.flow).unwrap();
            let i: usize = f0.events.iter().position(|e| e.name == h.after).unwrap();
            let Kind::Action { next: old, .. } = f0.events[i].kind else { panic!("{} is not an action", h.after) };
            let after: Bf = Bf::parse(&edit_raw(&raw, h, &flag).unwrap()).unwrap();
            let f: &Flow = after.flows.iter().find(|f| f.name == h.flow).unwrap();
            let Kind::Action { next: new, .. } = f.events[i].kind else { panic!() };
            assert_eq!(new as usize, f0.events.len());
            assert_eq!(flag_of(f, new), Some(flag.clone()));
            let Kind::Action { next: then, .. } = f.events[new as usize].kind else { panic!() };
            assert_eq!(then, old);
            assert_eq!(&f.events[..f0.events.len()].iter().enumerate().filter(|(j, _)| *j != i).map(|(_, e)| e.clone()).collect::<Vec<_>>(), &f0.events.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, e)| e.clone()).collect::<Vec<_>>());
        }
    }

    #[test]
    fn the_seed_mark_starts_every_opening_entry_and_leads_on_to_its_old_start() {
        let Some(mut rf) = real() else { return };
        let raw: Vec<u8> = rf.read_zs(SEED.file).unwrap();
        let before: Bf = Bf::parse(&raw).unwrap();
        let f0: &Flow = before.flows.iter().find(|f| f.name == SEED.flow).unwrap();
        let flag: String = seed_flag("0123456789ab");
        let after: Bf = Bf::parse(&edit_raw(&raw, SEED, &flag).unwrap()).unwrap();
        let f: &Flow = after.flows.iter().find(|f| f.name == SEED.flow).unwrap();
        assert!(!f.entries.is_empty());
        for (e, e0) in f.entries.iter().zip(&f0.entries) {
            assert_eq!(flag_of(f, e.main), Some(flag.clone()));
            let Kind::Action { next, .. } = f.events[e.main as usize].kind else { panic!() };
            assert_eq!(next, e0.main);
            assert!(reached(f, e.main));
        }
    }

    #[test]
    fn a_missing_hook_or_flow_is_an_error() {
        let Some(mut rf) = real() else { return };
        let raw: Vec<u8> = rf.read_zs(SEED.file).unwrap();
        assert!(edit_raw(&raw, Hook { after: "NoSuchEvent", ..SEED }, "X").is_err());
        assert!(edit_raw(&raw, Hook { flow: "NoSuchFlow", ..SEED }, "X").is_err());
    }
}
