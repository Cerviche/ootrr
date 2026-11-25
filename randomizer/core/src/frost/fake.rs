use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::pickup_pack::actor::{own_name_ref, pouch_file, DEAL_PARAM, STOCK_PARAM};
use crate::pickup_pack::closure::{actor_file, chain, local, norm, select, work, Chain, Pool, COMPONENTS, PARENT};
use crate::sarc::{self, Entry};

pub const RUPEE_ITEM: &str = "Rupee (1)";
pub const RUPEE_ROW: &str = "Obj_Rupee_Green";
const POUCH_REF: &str = "PouchContentRef";
const NAME_REF: &str = "ActorNameRef";

#[derive(Debug, Clone)]
pub struct Fake {
    pub name: String,
    pub look: String,
    pub id: String,
    pub pack: Vec<u8>,
    pub fresh: Vec<(String, usize)>,
    pub peers: Vec<(String, usize)>,
}

fn s(x: &str) -> Node {
    Node::Str(x.to_string())
}

const CATEGORY: &str = "Category";
const TAB: &str = "TabCategory";

/// What the stand-in's own pouch file says over the file of the item it looks like, which it
/// inherits for how its get is shown: a plain key item that is never pushed to the pouch. The
/// same as its pouch row (`crate::frost::inert`), since the two must agree, and the way the
/// dungeon item packs say it (`crate::pickup_pack::actor::deal_of`).
pub fn inert_params() -> Vec<(&'static str, Node)> {
    let plain: Node = Node::U32(crate::flags::hash_name(crate::frost::PLAIN));
    vec![
        (CATEGORY, plain.clone()),
        (TAB, plain),
        (DEAL_PARAM, Node::U32(crate::flags::hash_name(crate::pickup_pack::rows::QUIET_DEAL))),
        (STOCK_PARAM, Node::Bool(false)),
        (crate::pickup_pack::kit::MANY, Node::Bool(true)),
        (crate::pickup_pack::kit::AT_MAX, Node::Bool(true)),
    ]
}

fn pouch_of(c: &Chain, who: &str) -> Res<String> {
    c.comps.get(POUCH_REF).and_then(|v| norm(v)).ok_or_else(|| E::Msg(format!("{who} has no pouch file")))
}

pub fn make(look_pack: &[Entry], look: &str, name: &str) -> Res<Fake> {
    let mut p: Pool = Pool::default();
    p.add_pack(look_pack)?;
    let peers: Vec<(String, usize)> = p.files.iter().map(|e| (e.0.clone(), e.1.len())).collect();
    let lc: Chain = chain(&p, look)?;
    let look_pouch: String = pouch_of(&lc, look)?;
    let id: String = lc
        .comps
        .get(NAME_REF)
        .and_then(|v| v.strip_prefix("?ActorSystem/ActorName/"))
        .and_then(|v| v.strip_suffix(".engine__actor__ActorName.bgyml"))
        .unwrap_or(look)
        .to_string();
    let version: u16 = p.byml(&actor_file(look))?.version;
    let mut comps: Node = Node::Map(Vec::new());
    comps.set(NAME_REF, s(&own_name_ref(name)));
    comps.set(POUCH_REF, s(&local(&pouch_file(name))));
    let mut actor: Node = Node::Map(Vec::new());
    actor.set(PARENT, s(&work(&actor_file(look))));
    actor.set(COMPONENTS, comps);
    let mut pouch: Node = Node::Map(Vec::new());
    pouch.set(PARENT, s(&work(&look_pouch)));
    for (k, v) in inert_params() {
        pouch.set(k, v);
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
    Ok(Fake { name: name.to_string(), look: look.to_string(), id, pack: sarc::write(&es)?, fresh, peers })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the game's own files hold: KeyItemDefault's Category and TabCategory, and the
    /// SpecialDeal of the dungeon item rows.
    #[test]
    fn the_stand_in_says_plain_key_item_and_not_pushed_the_way_the_game_writes_them() {
        let p: Vec<(&'static str, Node)> = inert_params();
        let of = |k: &str| p.iter().find(|e| e.0 == k).map(|e| e.1.clone());
        assert_eq!(of(CATEGORY), Some(Node::U32(1_879_595_265)));
        assert_eq!(of(TAB), Some(Node::U32(1_879_595_265)));
        assert_eq!(of(DEAL_PARAM), Some(Node::U32(2_292_023_464)));
        assert_eq!(of(STOCK_PARAM), Some(Node::Bool(false)));
    }
}
