use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::pickup_pack::actor::{own_name_ref, pouch_file, POUCH_ACTOR};
use crate::pickup_pack::closure::{actor_file, chain, local, norm, work, Chain, Pool, COMPONENTS, PARENT};
use crate::sarc::{self, Entry};

const POUCH_REF: &str = "PouchContentRef";
const NAME_REF: &str = "ActorNameRef";

pub struct Made {
    pub name: String,
    pub pack: Vec<u8>,
    pub fresh: Vec<(String, usize)>,
    pub peers: Vec<(String, usize)>,
}

fn s(x: &str) -> Node {
    Node::Str(x.to_string())
}

pub fn name_for(fake: &str) -> String {
    format!("{fake}_Obj")
}

pub fn make(orig_pack: &[Entry], orig: &str, fake: &str) -> Res<Made> {
    let name: String = name_for(fake);
    let mut p: Pool = Pool::default();
    p.add_pack(orig_pack)?;
    let peers: Vec<(String, usize)> = p.files.iter().map(|e| (e.0.clone(), e.1.len())).collect();
    let oc: Chain = chain(&p, orig)?;
    let orig_pouch: String = oc.comps.get(POUCH_REF).and_then(|v| norm(v)).ok_or_else(|| E::Msg(format!("{orig} has no pouch file")))?;
    let version: u16 = p.byml(&actor_file(orig))?.version;
    let mut comps: Node = Node::Map(Vec::new());
    comps.set(NAME_REF, s(&own_name_ref(fake)));
    comps.set(POUCH_REF, s(&local(&pouch_file(&name))));
    let mut actor: Node = Node::Map(Vec::new());
    actor.set(PARENT, s(&work(&actor_file(orig))));
    actor.set(COMPONENTS, comps);
    let mut pouch: Node = Node::Map(Vec::new());
    pouch.set(PARENT, s(&work(&orig_pouch)));
    pouch.set(POUCH_ACTOR, s(&work(&actor_file(fake))));
    let a: Vec<u8> = Byml { root: actor, version, big: false }.write()?;
    let q: Vec<u8> = Byml { root: pouch, version, big: false }.write()?;
    let fresh: Vec<(String, usize)> = vec![(actor_file(&name), a.len()), (pouch_file(&name), q.len())];
    let mut es: Vec<Entry> = orig_pack.to_vec();
    es.push(Entry { name: actor_file(&name), data: a });
    es.push(Entry { name: pouch_file(&name), data: q });
    Ok(Made { name, pack: sarc::write(&es)?, fresh, peers })
}
