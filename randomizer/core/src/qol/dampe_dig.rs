use crate::bfevfl::{Bf, Event, Flow, Kind, Val};
use crate::error::{Error, Res};

/// Dampé's gravedigging tour, in both copies of his flow (the base game's and the 1.1.0 one).
pub const FLOWS: [&str; 2] = ["Npc_Hylia_Dampe_001", "Npc_Hylia_Dampe_001.110"];
const NAME: &str = "Npc_Hylia_Dampe_001";
pub const DROPPED: &str = "GraveYard_DiggingHoleTour_Drop_HeartPiece";

pub const NOTE: &str = "Dampé's gravedigging tour gives its check on the first dig: before the dig lottery (Event219), a check of GraveYard_DiggingHoleTour_Drop_HeartPiece sends the dig to the Heart Piece branch (Event119) until the check is taken, and to the usual lottery after. Always on.";

fn named(f: &Flow, name: &str) -> Res<usize> {
    let hits: Vec<usize> = (0..f.events.len()).filter(|&i| f.events[i].name == name).collect();
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("first dig: expected one {name} in {NAME}, found {}", hits.len()))),
    }
}

pub fn patch(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == NAME).ok_or(Error::Format("first dig: the Dampé flow has no Npc_Hylia_Dampe_001 flow"))?;
    let idx = |i: usize| crate::dungeon_credit::idx(i);
    // Event119: has the Heart Piece dropped yet (0 yes: a rupee instead, 1 no: the Heart Piece).
    let heart: usize = named(f, "Event119")?;
    let asks: bool = f.events[heart].kind.params().and_then(|c| c.get("GameDataName")) == Some(&Val::Str(DROPPED.to_string()));
    let Kind::Switch { cases, .. } = &f.events[heart].kind else {
        return Err(Error::Format("first dig: Event119 is not a switch"));
    };
    if !asks || cases.len() != 2 {
        return Err(Error::Format("first dig: Event119 no longer asks whether the Heart Piece dropped"));
    }
    let lottery: usize = named(f, "Event219")?;
    let wait: usize = named(f, "Event93")?;
    if !matches!(f.events[wait].kind, Kind::Action { next, .. } if next as usize == lottery) {
        return Err(Error::Format("first dig: Event93 no longer leads to the dig lottery"));
    }
    // A copy of Event119 in front of the lottery: not dropped yet goes to the Heart Piece branch,
    // dropped goes to the lottery as before.
    let at: u16 = idx(f.events.len())?;
    let mut gate: Event = f.events[heart].clone();
    gate.name = "OOTRRFirstDig".to_string();
    if let Kind::Switch { cases, .. } = &mut gate.kind {
        for c in cases.iter_mut() {
            c.ev = if c.val == 0 { idx(lottery)? } else { idx(heart)? };
        }
    }
    f.events.push(gate);
    if let Kind::Action { next, .. } = &mut f.events[wait].kind {
        *next = at;
    }
    Ok(())
}
