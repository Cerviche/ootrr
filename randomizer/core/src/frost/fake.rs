use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::pickup_pack::actor::{own_name_ref, pouch_file};
use crate::pickup_pack::closure::{actor_file, chain, local, norm, select, work, Chain, Pool, COMPONENTS, PARENT};
use crate::sarc::{self, Entry};

pub const RUPEE: &str = "TBox_Rupee_Green";
pub const RUPEE_ITEM: &str = "Rupee (1)";
pub const RUPEE_ROW: &str = "Obj_Rupee_Green";
const GET_TYPE: &str = "GetType";
const POUCH_REF: &str = "PouchContentRef";
const NAME_REF: &str = "ActorNameRef";

#[derive(Debug, Clone)]
pub struct Fake {
    pub name: String,
    pub look: String,
    pub pack: Vec<u8>,
    pub fresh: Vec<(String, usize)>,
    pub peers: Vec<(String, usize)>,
}

fn s(x: &str) -> Node {
    Node::Str(x.to_string())
}

fn pouch_of(c: &Chain, who: &str) -> Res<String> {
    c.comps.get(POUCH_REF).and_then(|v| norm(v)).ok_or_else(|| E::Msg(format!("{who} has no pouch file")))
}

fn get_type(p: &Pool, mut at: String) -> Res<Option<Node>> {
    for _ in 0..8 {
        let Some(_) = p.get(&at) else { return Ok(None) };
        let y: Byml = p.byml(&at)?;
        if let Some(v) = y.root.get(GET_TYPE) {
            return Ok(Some(v.clone()));
        }
        match y.root.get(PARENT).and_then(|v| v.as_str()).and_then(norm) {
            Some(up) => at = up,
            None => return Ok(None),
        }
    }
    Ok(None)
}

pub fn make(look_pack: &[Entry], rupee_pack: &[Entry], look: &str, name: &str) -> Res<Fake> {
    let mut p: Pool = Pool::default();
    p.add_pack(rupee_pack)?;
    p.add_pack(look_pack)?;
    let peers: Vec<(String, usize)> = p.files.iter().map(|e| (e.0.clone(), e.1.len())).collect();
    let lc: Chain = chain(&p, look)?;
    let rc: Chain = chain(&p, RUPEE)?;
    let rupee_pouch: String = pouch_of(&rc, RUPEE)?;
    let kind: Option<Node> = get_type(&p, pouch_of(&lc, look)?)?;
    let version: u16 = p.byml(&actor_file(look))?.version;
    let mut comps: Node = Node::Map(Vec::new());
    comps.set(NAME_REF, s(&own_name_ref(name)));
    comps.set(POUCH_REF, s(&local(&pouch_file(name))));
    let mut actor: Node = Node::Map(Vec::new());
    actor.set(PARENT, s(&work(&actor_file(look))));
    actor.set(COMPONENTS, comps);
    let mut pouch: Node = Node::Map(Vec::new());
    pouch.set(PARENT, s(&work(&rupee_pouch)));
    if let Some(k) = kind {
        pouch.set(GET_TYPE, k);
    }
    let a: Vec<u8> = Byml { root: actor, version, big: false }.write()?;
    let q: Vec<u8> = Byml { root: pouch, version, big: false }.write()?;
    let fresh: Vec<(String, usize)> = vec![(actor_file(name), a.len()), (pouch_file(name), q.len())];
    p.add(&actor_file(name), &a)?;
    p.add(&pouch_file(name), &q)?;
    let files: Vec<String> = select(&p, name)?;
    for (f, _) in &fresh {
        if !files.contains(f) {
            return Err(E::Msg(format!("the actor {name} does not reach its own file {f}")));
        }
    }
    let es: Vec<Entry> = files.iter().map(|f| Entry { name: f.clone(), data: p.get(f).unwrap_or(&[]).to_vec() }).collect();
    Ok(Fake { name: name.to_string(), look: look.to_string(), pack: sarc::write(&es)?, fresh, peers })
}
