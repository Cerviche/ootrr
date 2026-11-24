use std::collections::BTreeMap as BM;

use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::placement::hash_of;

const DROP: &str = "Drop__DropActor";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Swap {
    Content(String),
    Actor(String),
}

pub fn banc(raw: &[u8], hits: &BM<u64, Swap>) -> Res<(Vec<u8>, usize)> {
    let mut y: Byml = Byml::parse(raw)?;
    let mut n: usize = 0;
    let list: &mut Vec<Node> = y.root.get_mut("Actors").and_then(|v| v.as_arr_mut()).ok_or(E::Format("a map file has no actor list"))?;
    for a in list.iter_mut() {
        let Some(h) = hash_of(a) else { continue };
        match hits.get(&h) {
            Some(Swap::Content(fake)) => {
                let d: &mut Node = a.get_mut("Dynamic").ok_or(E::Format("a chest has no dynamic parameters"))?;
                if d.get(DROP).is_none() {
                    return Err(E::Msg(format!("actor {h} is not a chest with a content")));
                }
                d.set(DROP, Node::Str(fake.clone()));
                n += 1;
            }
            Some(Swap::Actor(name)) => {
                a.set("Gyaml", Node::Str(name.clone()));
                n += 1;
            }
            None => {}
        }
    }
    Ok((y.write()?, n))
}
