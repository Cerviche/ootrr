use std::path::Path as P;

use crate::error::{Error, Res};
use crate::romfs::Romfs;
use crate::slice::data::{Data, Item};
use crate::staging::Out;

pub const STAND_IN: &str = "Recovery Heart";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foreign {
    pub label: String,
    pub item: String,
    pub player: String,
    pub progression: bool,
}

pub fn actor(label: &str) -> String {
    let _ = label;
    String::new()
}

pub fn register(d: &mut Data, f: &[Foreign]) -> Res<()> {
    let base: Item = d.item(STAND_IN).cloned().ok_or_else(|| Error::Msg(format!("ap item: the stand-in {STAND_IN} is not an item")))?;
    for x in f {
        if d.item_ix.contains_key(&x.label) {
            continue;
        }
        let mut it: Item = base.clone();
        it.name = x.label.clone();
        it.adv = false;
        d.item_ix.insert(it.name.clone(), d.items.len());
        d.items.push(it);
    }
    Ok(())
}

pub fn stage(rf: &mut Romfs, work: &P, outs: &mut Vec<Out>, f: &[Foreign]) -> Res<()> {
    let _ = (rf, work, outs, f);
    Ok(())
}
