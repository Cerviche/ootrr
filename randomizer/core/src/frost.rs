pub mod chest;
pub mod fake;
pub mod ground;
pub mod sensor;
pub mod trigger;

use std::collections::BTreeMap as BM;
use std::fs;
use std::path::{Path as P, PathBuf as PB};

use crate::bfevfl::Bf;
use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::flow_locations::dict_id;
use crate::modpack::{Host, Pack};
use crate::msbt::{self, Msbt};
use crate::pickup_pack::actor::{pouch_file, POUCH_ACTOR};
use crate::pickup_pack::closure::{actor_file, work};
use crate::pickup_pack::rows::{self, ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO};
use crate::pickup_pack::write::start;
use crate::pickup_pack::{pack_of, sizes, PACK_DICT};
use crate::progressive::{ANALYZED_TABLE, SETTING_TABLE};
use crate::rng::Rng;
use crate::romfs::Romfs;
use crate::rsizetable::{self, Rst, TABLE_PATH};
use crate::sarc::{self, Entry};
use crate::settings::seed::SeedFile;
use crate::slice::data::Data;
use crate::slice::edit::Built;
use crate::staging::{BANC_DICT, MARGIN};
use crate::zs;
use crate::TITLE_ID;

pub const TRAP: &str = "Ice Trap";
pub const LOOKS: [&str; 24] = [
    "Ocarina",
    "Slingshot",
    "Boomerang",
    "Bow",
    "Bomb Bag",
    "Progressive Hookshot",
    "Megaton Hammer",
    "Lens of Truth",
    "Dins Fire",
    "Farores Wind",
    "Nayrus Love",
    "Fire Arrows",
    "Ice Arrows",
    "Light Arrows",
    "Iron Boots",
    "Hover Boots",
    "Goron Tunic",
    "Zora Tunic",
    "Mirror Shield",
    "Bottle",
    "Kokiri Sword",
    "Progressive Strength Upgrade",
    "Biggoron Sword",
    "Progressive Scale",
];
const NO_SONG: &str = "Scarecrow Song";
pub const SETTING: &str = "shuffle_ice_traps";
pub const COUNT: &str = "ice_trap_count";
pub const NUM: u16 = 204;
pub const COUNT_NUM: u16 = 205;
const GROUND_KINDS: [&str; 4] = ["freestanding", "heart", "bossheart", "key"];
const POUCH_MSG: &str = "ActorMsg/PouchContent.msbt";
const CHILD: &str = "Player_Child";
const CHILD_CONDITION: &str = "Component/ConditionParam/Player_Child.game__component__ConditionParam.bgyml";
const CHILD_LINK: &str = "Component/ELink/Player.engine__component__ELinkParam.bgyml";
const ADULT_USER: &str = "Player_Adult";
const CHILD_USER: &str = "OOTRR_Player_Child_Cold";
const SHELLS: usize = 16;
const BONE_BIT: u64 = 1 << 12;
const SCALE_BIT: u64 = 1 << 13;
const CHILD_SIZE: f32 = 0.5;
const GET_TEXT: &str = "FOOL!";
const FROZEN_CALL: &str = "Condition_Frozen";
const FULL_BODY_CALL: &str = "全身凍結";
const PARAM_AT: usize = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Chest,
    Ground,
    Shop(i64),
    Skull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trap {
    pub loc: String,
    pub place: Place,
    pub look: String,
    pub item: String,
    pub actor: String,
    pub fake: String,
    pub kit: String,
}

pub fn fake_name(actor: &str) -> String {
    format!("OOTRR_{actor}_Cold")
}

impl Trap {
    pub fn trigger_names(&self) -> Vec<String> {
        match self.place {
            Place::Chest => vec![self.fake.clone()],
            Place::Ground => vec![self.fake.clone(), ground::name_for(&self.fake)],
            Place::Shop(p) => vec![self.fake.clone(), crate::shelf::shelf_name(&self.actor, p as i32)],
            Place::Skull => vec![self.fake.clone(), self.kit.clone()],
        }
    }
}

fn spheres(w: &crate::world::World, place: &crate::world::Placement) -> BM<usize, usize> {
    let mut s: crate::search::Search = crate::search::Search::new(w);
    let mut out: BM<usize, usize> = BM::new();
    let mut n: usize = 0;
    let mut found: Vec<usize> = s.settle(place, false);
    while !found.is_empty() {
        for l in &found {
            out.entry(*l).or_insert(n);
            if let Some(i) = place.get(*l) {
                s.add(i, 1);
            }
        }
        n += 1;
        found = s.settle(place, false);
    }
    out
}

fn pool(d: &Data, seed: &SeedFile) -> Vec<String> {
    let set: Option<crate::settings::Settings> = seed.settings().ok().map(|l| l.settings);
    let is = |id: &str, v: &str| set.as_ref().and_then(|s| s.text(id).ok()).is_some_and(|x| x == v);
    let mut out: Vec<String> = LOOKS.iter().map(|x| x.to_string()).collect();
    for i in &d.items {
        let song: bool = i.kind == "Song" && i.name != NO_SONG && is("shuffle_songs", "anywhere");
        let reward: bool = i.kind == "DungeonReward" && is("shuffle_dungeon_rewards", "anywhere");
        if (song || reward) && i.actor.is_some() {
            out.push(i.name.clone());
        }
    }
    out.retain(|n| d.item(n).is_some_and(|i| i.actor.is_some()));
    out
}

fn place_of(d: &Data, seed: &SeedFile, loc: &str) -> Res<Place> {
    let kind: Option<&str> = d.cat.by_name(loc).map(|l| l.kind.as_str());
    Ok(if crate::shops::is_loc(loc) {
        Place::Shop(seed.shops.get(loc).map(|s| s.price).ok_or_else(|| E::Msg(format!("{loc} holds {TRAP} but has no shop entry")))?)
    } else if kind == Some("chest") {
        Place::Chest
    } else if kind.is_some_and(|k| GROUND_KINDS.contains(&k)) {
        Place::Ground
    } else if kind == Some("gs") {
        Place::Skull
    } else {
        return Err(E::Msg(format!("{loc} holds {TRAP}, which this build can only put in a chest, on the ground, in a shop or at a Gold Skulltula")));
    })
}

pub fn can_hold(d: &Data, loc: &str) -> bool {
    crate::shops::is_loc(loc) || d.cat.by_name(loc).is_some_and(|l| l.kind == "chest" || l.kind == "gs" || GROUND_KINDS.contains(&l.kind.as_str()))
}

fn tier_of(d: &Data, item: &str, held: usize) -> Option<(String, bool)> {
    let i = d.item(item)?;
    if i.tiers.is_empty() {
        return (held == 0).then(|| (i.actor.clone().unwrap_or_default(), false));
    }
    i.tiers.get(held).map(|t| (t.clone(), true))
}

fn fits(d: &Data, place: Place, item: &str, actor: &str, tiered: bool) -> bool {
    let tier: Option<&str> = tiered.then_some(actor);
    match place {
        Place::Chest => true,
        Place::Ground => crate::slice::key_spots::resolve(d, item, tier).is_ok(),
        Place::Skull => crate::slice::key_spots::resolve_kit(d, item, tier).is_ok_and(|p| p.hang_with().is_ok()),
        Place::Shop(_) => !tiered && d.item(item).is_some_and(|i| i.kind == "Item"),
    }
}

/// Chooses each trap's look: a key item the logic says the player cannot hold yet where the trap
/// is reached, and for an upgrade the next tier the player lacks there.
fn looks(d: &Data, seed: &SeedFile, places: &BM<String, Place>) -> Res<BM<String, (String, String, bool)>> {
    let (w, place): (crate::world::World, crate::world::Placement) = crate::slice::starts::rebuild(d, seed)?;
    let at: BM<usize, usize> = spheres(&w, &place);
    let names: Vec<String> = pool(d, seed);
    let held = |item: &str, here: usize| -> usize {
        let Some(it) = w.item(item) else { return 0 };
        if here == usize::MAX {
            return 0;
        }
        let start: usize = w.start.iter().filter(|(i, _)| *i == it).map(|(_, n)| *n as usize).sum();
        start + w.real.iter().filter(|l| place.get(**l) == Some(it) && at.get(*l).is_some_and(|s| *s < here)).count()
    };
    let skull = |l: &str| d.cat.by_name(l).is_some_and(|c| c.kind == "gs");
    let at_skulls: Vec<&String> = seed.placements.iter().filter(|(l, i)| *i != TRAP && skull(l)).map(|(_, i)| i).collect();
    let at_shops: Vec<&String> = seed.placements.iter().filter(|(l, i)| *i != TRAP && crate::shops::is_loc(l)).map(|(_, i)| i).collect();
    let mut out: BM<String, (String, String, bool)> = BM::new();
    for (loc, pl) in places {
        let here: usize = w.loc(loc).and_then(|l| at.get(&l).copied()).unwrap_or(usize::MAX);
        let taken = |n: &String| match pl {
            Place::Skull => at_skulls.contains(&n),
            Place::Shop(_) => at_shops.contains(&n),
            _ => false,
        };
        let pick = |late: bool| -> Vec<(String, String, bool)> {
            names
                .iter()
                .filter(|n| !taken(n))
                .filter_map(|n| tier_of(d, n, if late { held(n, here) } else { 0 }).map(|(a, t)| (n.clone(), a, t)))
                .filter(|(n, a, t)| fits(d, *pl, n, a, *t))
                .collect()
        };
        let mut c: Vec<(String, String, bool)> = pick(true);
        if c.is_empty() {
            c = pick(false);
        }
        if c.is_empty() {
            return Err(E::Msg(format!("{loc} holds {TRAP} but no key item can show there")));
        }
        let mut r: Rng = Rng::from_text(&format!("{}/{loc}/look", seed.seed));
        out.insert(loc.clone(), c.swap_remove(r.below(c.len() as u64) as usize));
    }
    Ok(out)
}

pub fn prepare(seed: &SeedFile) -> Res<(SeedFile, Vec<Trap>)> {
    if !seed.placements.values().any(|v| v == TRAP) {
        return Ok((seed.clone(), Vec::new()));
    }
    let d: Data = Data::load()?;
    let mut places: BM<String, Place> = BM::new();
    for (loc, item) in &seed.placements {
        if item == TRAP {
            places.insert(loc.clone(), place_of(&d, seed, loc)?);
        }
    }
    let chosen: BM<String, (String, String, bool)> = looks(&d, seed, &places)?;
    let mut out: SeedFile = seed.clone();
    let mut traps: Vec<Trap> = Vec::new();
    let mut short: Vec<String> = shortfall(&d, seed);
    for (loc, place) in places {
        let (look, actor, tiered): (String, String, bool) = chosen.get(&loc).cloned().ok_or_else(|| E::Msg(format!("{loc} has no look")))?;
        // A chest or shop keeps the look as its item, so it stands as that item would. An upgrade,
        // a ground spot and a skulltula keep a junk carrier instead, so the upgrade tier order of
        // the real copies is untouched, and the trap's own pickup is swapped in.
        let carry: bool = tiered || matches!(place, Place::Ground | Place::Skull);
        let pick: Option<usize> = short.iter().position(|x| !d.item(x).is_some_and(|i| i.adv)).or(if short.is_empty() { None } else { Some(0) });
        let held: Option<String> = pick.map(|k| short.remove(k));
        let stand: String = if carry { held.clone().unwrap_or_else(|| fake::RUPEE_ITEM.to_string()) } else { look.clone() };
        if let Some(h) = &held {
            *out.copies.entry(h.clone()).or_insert(0) -= 1;
        }
        *out.copies.entry(stand.clone()).or_insert(0) += 1;
        let fake: String = fake_name(&actor);
        let mut kit: String = String::new();
        match place {
            Place::Ground => {
                let g: String = ground::name_for(&fake);
                if tiered {
                    out.tiers.insert(g.clone(), actor.clone());
                }
                out.named.insert(g, look.clone());
            }
            Place::Skull => {
                let tier: Option<&str> = tiered.then_some(actor.as_str());
                let t: crate::slice::skulls::Table = crate::slice::skulls::table()?;
                kit = crate::slice::skulls::drop_actor(&t, &d, &look, tier)?;
                if crate::slice::skulls::pickup(&t, &d, &look).is_none() {
                    out.kits.insert(kit.clone(), look.clone());
                    if let Some(a) = tier {
                        out.tiers.insert(kit.clone(), a.to_string());
                    }
                }
                out.drops.insert(loc.clone(), kit.clone());
            }
            Place::Shop(_) => {
                if let Some(s) = out.shops.get_mut(&loc) {
                    s.item = look.clone();
                }
            }
            Place::Chest => {}
        }
        out.placements.insert(loc.clone(), stand);
        traps.push(Trap { loc, place, look: look.clone(), item: look, fake, actor, kit });
    }
    Ok((out, traps))
}

/// Swaps junk in the generated seed for traps: up to the set count, never at an excluded
/// location, only at a chest, a ground spot, a shuffled shop slot or a skulltula whose actor is known.
pub fn sprinkle(d: &Data, s: &crate::settings::Settings, seed: &str, f: &mut SeedFile, excluded: &[String]) -> usize {
    if !s.flag(SETTING).unwrap_or(false) {
        return 0;
    }
    let n: usize = s.int(COUNT).unwrap_or(0).clamp(0, 20) as usize;
    let contested: Vec<String> = crate::slice::skulls::shared().map(|t| t.contested.clone()).unwrap_or_default();
    let mut spots: Vec<String> = f
        .placements
        .iter()
        .filter(|(l, i)| {
            crate::slice::place_rules::junk(d, i)
                && !excluded.contains(l)
                && can_hold(d, l)
                && !crate::pot_slice::targets::is_loc(d, l)
                && !crate::slice::silver::is_spot(l)
                && (!crate::shops::is_loc(l) || f.shops.get(*l).is_some_and(|e| e.deal && e.item == **i))
                && d.cat.by_name(l).is_none_or(|c| c.kind != "gs" || (c.conf != crate::catalog::Conf::Pool && !contested.contains(l)))
        })
        .map(|(l, _)| l.clone())
        .collect();
    let mut r: Rng = Rng::from_text(&format!("{seed}/cold"));
    let mut out: usize = 0;
    while out < n && !spots.is_empty() {
        let l: String = spots.swap_remove(r.below(spots.len() as u64) as usize);
        f.placements.insert(l.clone(), TRAP.to_string());
        if let Some(e) = f.shops.get_mut(&l) {
            e.item = TRAP.to_string();
        }
        out += 1;
    }
    out
}

fn shortfall(d: &Data, seed: &SeedFile) -> Vec<String> {
    let mut need: BM<String, isize> = BM::new();
    for (loc, item) in &seed.placements {
        if crate::pot_slice::targets::is_loc(d, loc) || crate::shops::seed::skip(loc) || crate::slice::silver::is_spot(loc) {
            continue;
        }
        if let Some(v) = d.pool_item(loc) {
            *need.entry(v.to_string()).or_insert(0) += 1;
        }
        if item != TRAP && !crate::slice::silver::is_item(item) {
            *need.entry(item.clone()).or_insert(0) -= 1;
        }
    }
    let mut out: Vec<String> = Vec::new();
    for (k, n) in need {
        let net: isize = crate::pot_slice::targets::delta(d, seed, &k) + crate::shops::seed::delta(seed, &k) + crate::slice::silver::delta(seed, &k);
        out.extend(std::iter::repeat_n(k, (n + net).max(0) as usize));
    }
    out
}

struct Ctx<'a> {
    rf: Romfs,
    base: &'a PB,
    modded: bool,
}

struct File {
    path: String,
    raw: Vec<u8>,
    old: usize,
    plain: bool,
}

impl Ctx<'_> {
    fn read(&mut self, path: &str) -> Res<Vec<u8>> {
        start(&mut self.rf, Some(self.base), path)
    }

    fn file(&mut self, path: &str, edit: impl FnOnce(&[u8]) -> Res<Vec<u8>>) -> Res<File> {
        let first: Vec<u8> = self.read(path)?;
        let raw: Vec<u8> = edit(&first)?;
        let old: usize = if self.modded { first.len() } else { self.rf.read_zs(path)?.len() };
        Ok(File { path: path.to_string(), raw, old, plain: false })
    }

    fn plain(&mut self, path: &str, edit: impl FnOnce(&[u8]) -> Res<Vec<u8>>) -> Res<File> {
        let own: PB = path.split('/').fold(self.base.clone(), |a, s| a.join(s));
        let first: Vec<u8> = if own.is_file() { fs::read(own)? } else { self.rf.read(path)? };
        let raw: Vec<u8> = edit(&first)?;
        let old: usize = if self.modded { first.len() } else { self.rf.read(path)?.len() };
        Ok(File { path: path.to_string(), raw, old, plain: true })
    }

    fn pack(&mut self, name: &str) -> Res<Vec<Entry>> {
        sarc::parse(&self.read(&pack_of(name))?)
    }
}

fn dict(rf: &Romfs, path: &str) -> Res<Option<u32>> {
    if !rf.root.join(path).is_file() {
        return Ok(if path.starts_with("Pack/") { Some(PACK_DICT) } else { None });
    }
    let id: Option<u32> = dict_id(&rf.read(path)?);
    Ok(if path.starts_with("Banc/") { Some(id.unwrap_or(BANC_DICT)) } else { id })
}

fn add_actor(ts: &mut [Byml; 3], from: &str, name: &str, pouch: Option<&str>) -> Res<()> {
    let h: i32 = rows::heap(&ts[0], from)?;
    let mut a: Node = rows::actor_row(rows::need(&ts[0], from, ACTOR_INFO)?, name, h);
    if a.get(rows::ACTOR_NAME).is_some() {
        a.set(rows::ACTOR_NAME, Node::Str(name.to_string()));
    }
    let g: Node = rows::game_row(rows::need(&ts[1], from, GAME_ACTOR_INFO)?, rows::need(&ts[1], from, GAME_ACTOR_INFO)?, name);
    let i: usize = rows::insert(&mut ts[0], a)?;
    let j: usize = rows::insert(&mut ts[1], g)?;
    if i != j {
        return Err(E::Msg(format!("{name} sits at {i} in the actor table and at {j} in the game actor table")));
    }
    if let Some(p) = pouch {
        let q: Node = rows::pouch_row(rows::need(&ts[2], p, POUCH_INFO)?, name);
        rows::insert(&mut ts[2], q)?;
    }
    Ok(())
}

fn tables(c: &mut Ctx, fakes: &[(String, String)], ids: &BM<String, String>, grounds: &[(String, String)]) -> Res<Vec<File>> {
    let firsts: Vec<Vec<u8>> = [ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO].iter().map(|t| c.read(t)).collect::<Res<Vec<Vec<u8>>>>()?;
    let mut ts: [Byml; 3] = [Byml::parse(&firsts[0])?, Byml::parse(&firsts[1])?, Byml::parse(&firsts[2])?];
    add_actor(&mut ts, sensor::SUN, sensor::NAME, None)?;
    for (look, name) in fakes {
        let id: &str = ids.get(name).map(|x| x.as_str()).unwrap_or(look);
        let pouch: String = [id, look.as_str()].iter().find(|k| rows::find(&ts[2], k).is_some()).map(|k| k.to_string()).unwrap_or_else(|| fake::RUPEE_ROW.to_string());
        add_actor(&mut ts, look, name, Some(&pouch))?;
    }
    for (g, fake) in grounds {
        let list: &mut Vec<Node> = ts[0].root.as_arr_mut().ok_or(E::Format("the actor table is not a list"))?;
        let r: &mut Node = list.iter_mut().find(|r| rows::id(r) == Some(g.as_str())).ok_or_else(|| E::Msg(format!("the actor table has no {g} row")))?;
        r.set(rows::ACTOR_NAME, Node::Str(fake.clone()));
    }
    let mut out: Vec<File> = Vec::new();
    for (i, t) in [ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO].iter().enumerate() {
        let old: usize = if c.modded { firsts[i].len() } else { c.rf.read_zs(t)?.len() };
        out.push(File { path: t.to_string(), raw: ts[i].write()?, old, plain: false });
    }
    Ok(out)
}

fn text(raw: &[u8], fakes: &[(String, String)]) -> Res<Vec<u8>> {
    let mut es: Vec<Entry> = sarc::parse(raw)?;
    let b: Vec<u8> = sarc::get(&es, POUCH_MSG).ok_or(E::Format("a message archive has no PouchContent.msbt"))?.data.clone();
    let mut m: Msbt = msbt::parse(&b)?;
    for (look, name) in fakes {
        for sfx in ["_Name", "_Caption", "_GetCaption"] {
            let own: String = format!("{name}{sfx}");
            if m.find(&own).is_some() {
                continue;
            }
            if let Some(v) = m.raw(&format!("{look}{sfx}")).map(|x| x.to_vec()) {
                m.add_raw(&own, v)?;
            }
        }
        let shop: String = format!("{name}_ShopCaption");
        if m.find(&shop).is_none() {
            if let Some(v) = ["_ShopCaption", "_Caption", "_GetCaption"].iter().find_map(|sfx| m.raw(&format!("{look}{sfx}")).map(|x| x.to_vec())) {
                m.add_raw(&shop, v)?;
            }
        }
        m.put_markup(&format!("{name}_GetMessage"), GET_TEXT)?;
    }
    let w: Vec<u8> = m.write()?;
    msbt::parse(&w)?;
    sarc::set(&mut es, POUCH_MSG, w);
    sarc::write(&es)
}

fn full_body(raw: &[u8]) -> Res<Vec<u8>> {
    let mut l: crate::difficulty::elink::Link = crate::difficulty::elink::Link::parse(raw)?;
    let mut n: usize = 0;
    for u in 0..l.users.len() {
        let (Some(f), Some(g)) = (l.find_key(u, FROZEN_CALL), l.find_key(u, FULL_BODY_CALL)) else { continue };
        let pos: u64 = l.users[u].calls[g].param_pos();
        l.users[u].calls[f].raw[PARAM_AT..PARAM_AT + 8].copy_from_slice(&pos.to_le_bytes());
        n += 1;
    }
    if n == 0 {
        return Err(E::Format("the effect table has no frozen and full body calls in one user"));
    }
    child_user(&mut l)?;
    l.write()
}

fn child_user(l: &mut crate::difficulty::elink::Link) -> Res<()> {
    if l.user(CHILD_USER).is_some() {
        return Ok(());
    }
    let a: usize = l.user(ADULT_USER).ok_or(E::Format("the effect table has no adult Link user"))?;
    let scale: u32 = l.direct_f32(CHILD_SIZE).ok_or(E::Format("the effect table has no 0.7 scale value"))?;
    let mut u: crate::difficulty::elink::User = l.users[a].clone();
    for k in 1..=SHELLS {
        let c: usize = l.find_key(a, &format!("Condition_Frozen_{k:02}")).ok_or_else(|| E::Msg(format!("adult Link has no frozen shell {k}")))?;
        let mut p: crate::difficulty::elink::Param = l.param_at(u.calls[c].param_pos()).cloned().ok_or(E::Format("a frozen shell has no field record"))?;
        if p.mask & BONE_BIT == 0 || p.mask & SCALE_BIT != 0 {
            return Err(E::Msg(format!("frozen shell {k} is not sized by a bone")));
        }
        let at: usize = (p.mask & (BONE_BIT - 1)).count_ones() as usize;
        p.mask = (p.mask & !BONE_BIT) | SCALE_BIT;
        *p.vals.get_mut(at).ok_or(E::Format("a frozen shell record is short"))? = scale;
        let pos: u64 = l.add_param(p);
        u.calls[c].raw[PARAM_AT..PARAM_AT + 8].copy_from_slice(&pos.to_le_bytes());
    }
    let h: u32 = crate::rsizetable::hash(CHILD_USER);
    u.hash = h;
    let at: usize = l.hashes.partition_point(|x| *x < h);
    l.users.push(u);
    let slot: usize = l.users.len() - 1;
    l.hashes.insert(at, h);
    l.slots.insert(at, slot);
    Ok(())
}

fn child(raw: &[u8]) -> Res<Vec<u8>> {
    let mut es: Vec<Entry> = sarc::parse(raw)?;
    let e: &Entry = sarc::get(&es, CHILD_CONDITION).ok_or(E::Format("child Link has no condition file"))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    y.root.set("IsEnableFrozen", Node::Bool(true));
    sarc::set(&mut es, CHILD_CONDITION, y.write()?);
    let e: &Entry = sarc::get(&es, CHILD_LINK).ok_or(E::Format("child Link has no effect file"))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    y.root.set("UserName", Node::Str(CHILD_USER.to_string()));
    sarc::set(&mut es, CHILD_LINK, y.write()?);
    sarc::write(&es)
}

fn shelf(raw: &[u8], shelf: &str, fake: &str) -> Res<Vec<u8>> {
    let mut es: Vec<Entry> = sarc::parse(raw)?;
    let f: String = pouch_file(shelf);
    let e: &Entry = sarc::get(&es, &f).ok_or_else(|| E::Msg(format!("the shelf actor {shelf} has no pouch file")))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    y.root.set(POUCH_ACTOR, Node::Str(work(&actor_file(fake))));
    sarc::set(&mut es, &f, y.write()?);
    sarc::write(&es)
}

pub fn after(traps: &[Trap], romfs: &P, load: &P, mod_name: &str, host: Host, b: Built) -> Res<Built> {
    if traps.is_empty() {
        return Ok(b);
    }
    let d: Data = Data::load()?;
    let base: PB = b.root.join("romfs");
    let mut c: Ctx = Ctx { rf: Romfs::open(romfs)?, base: &base, modded: false };
    let mut rst: Rst = match Some(base.join(TABLE_PATH)).filter(|p| p.is_file()) {
        Some(p) => {
            c.modded = true;
            Rst::parse(&c.rf.zs.decompress(&fs::read(p)?)?)?
        }
        None => rsizetable::load(&mut c.rf)?,
    };
    let mut fakes: Vec<(String, String)> = traps.iter().map(|t| (t.actor.clone(), t.fake.clone())).collect();
    fakes.sort();
    fakes.dedup();
    let mut packs: Vec<(String, Vec<u8>, Vec<(String, usize)>, Vec<(String, usize)>)> = Vec::new();
    let sun: Vec<Entry> = c.pack(sensor::SUN)?;
    let frz: Vec<Entry> = c.pack(sensor::FREEZARD)?;
    let box4: Vec<Entry> = c.pack(sensor::TRAP_CHEST)?;
    let s: sensor::Made = sensor::make(&sun, &frz, &box4)?;
    packs.push((sensor::NAME.to_string(), s.pack, s.fresh, s.peers));
    let mut ids: BM<String, String> = BM::new();
    for (look, name) in &fakes {
        let lp: Vec<Entry> = c.pack(look)?;
        let f: fake::Fake = fake::make(&lp, look, name)?;
        ids.insert(f.name.clone(), f.id.clone());
        packs.push((f.name, f.pack, f.fresh, f.peers));
    }
    let mut grounds: Vec<(String, String)> = traps.iter().filter(|t| t.place == Place::Ground).map(|t| (ground::name_for(&t.fake), t.fake.clone())).collect();
    grounds.sort();
    grounds.dedup();
    let mut renamed: Vec<(String, String)> = grounds.clone();
    for t in traps {
        if let Place::Shop(p) = t.place {
            renamed.push((crate::shelf::shelf_name(&t.actor, p as i32), t.fake.clone()));
        }
        if t.place == Place::Skull {
            renamed.push((t.kit.clone(), t.fake.clone()));
        }
    }
    renamed.sort();
    renamed.dedup();
    let mut files: Vec<File> = tables(&mut c, &fakes, &ids, &renamed)?;
    for (g, fake) in &grounds {
        files.push(c.file(&pack_of(g), |r: &[u8]| shelf(r, g, fake))?);
    }
    let mut kits: Vec<(String, String)> = traps.iter().filter(|t| t.place == Place::Skull).map(|t| (t.kit.clone(), t.fake.clone())).collect();
    kits.sort();
    kits.dedup();
    for (k, fake) in &kits {
        files.push(c.file(&pack_of(k), |r: &[u8]| shelf(r, k, fake))?);
    }
    let mut by_file: BM<String, BM<u64, chest::Swap>> = BM::new();
    for t in traps {
        let swap: chest::Swap = match t.place {
            Place::Chest => chest::Swap::Content(t.fake.clone()),
            Place::Ground => chest::Swap::Actor(ground::name_for(&t.fake)),
            Place::Shop(_) | Place::Skull => continue,
        };
        let l = d.cat.by_name(&t.loc).ok_or_else(|| E::Msg(format!("{} is not in the location list", t.loc)))?;
        for r in &l.refs {
            for f in &r.files {
                by_file.entry(f.clone()).or_default().insert(r.hash, swap.clone());
            }
        }
    }
    for (f, hits) in &by_file {
        let mut hit: usize = 0;
        let file: File = c.file(f, |r: &[u8]| {
            let (out, n): (Vec<u8>, usize) = chest::banc(r, hits)?;
            hit = n;
            Ok(out)
        })?;
        if hit > 0 {
            files.push(file);
        }
    }
    for t in traps {
        if let Place::Shop(p) = t.place {
            let name: String = crate::shelf::shelf_name(&t.actor, p as i32);
            let fake: String = t.fake.clone();
            files.push(c.file(&pack_of(&name), |r: &[u8]| shelf(r, &name, &fake))?);
        }
    }
    let mut names: Vec<String> = traps.iter().flat_map(|t| t.trigger_names()).collect();
    names.sort();
    names.dedup();
    let fake_names: Vec<String> = fakes.iter().map(|f| f.1.clone()).collect();
    files.push(c.file(trigger::GET_FILE, |r: &[u8]| {
        let mut b: Bf = Bf::parse(r)?;
        trigger::hook_get(&mut b, &names)?;
        b.write()
    })?);
    for flow in trigger::PICKUPS {
        files.push(c.file(&format!("Event/EventFlow/{flow}.bfevfl.zs"), |r: &[u8]| {
            let mut b: Bf = Bf::parse(r)?;
            trigger::hook_pickup(&mut b, flow, &names)?;
            b.write()
        })?);
    }
    if traps.iter().any(|t| matches!(t.place, Place::Shop(_))) {
        files.push(c.file(trigger::SHOP_FILE, |r: &[u8]| {
            let mut b: Bf = Bf::parse(r)?;
            trigger::hook_shop(&mut b, &names)?;
            b.write()
        })?);
    }
    files.push(c.file(trigger::GIVE_FILE, |r: &[u8]| {
        let mut b: Bf = Bf::parse(r)?;
        trigger::add_event(&mut b, &fake_names)?;
        b.write()
    })?);
    files.push(c.file(SETTING_TABLE, trigger::setting)?);
    files.push(c.file(ANALYZED_TABLE, trigger::analyzed)?);
    files.push(c.file(crate::chest_slot::LIST, |r: &[u8]| {
        let mut y: Byml = Byml::parse(r)?;
        crate::ap::saved::add_bools(&mut y, &[trigger::FLAG.to_string()])?;
        y.write()
    })?);
    files.push(c.plain(trigger::MAIN, trigger::watch)?);
    let named: Vec<(String, String)> = fakes.iter().map(|(look, name)| (ids.get(name).cloned().unwrap_or_else(|| look.clone()), name.clone())).collect();
    for l in crate::hints::languages(&c.rf)? {
        files.push(c.file(&l, |r: &[u8]| text(r, &named))?);
    }
    files.push(c.file(&pack_of(CHILD), child)?);
    files.push(c.file(crate::difficulty::elink::FILE, full_body)?);
    let pk: Pack = Pack::create_for(load, TITLE_ID, mod_name, host)?;
    let id: u32 = dict_id(&c.rf.read(&pack_of(sensor::SUN))?).unwrap_or(PACK_DICT);
    for (name, raw, fresh, peers) in &packs {
        let path: String = pack_of(name);
        rst.set_size(&path, sizes::pack_size(raw.len())?)?;
        for (f, len) in fresh {
            let n: u32 = sizes::file_size(&rst, peers, f, *len)?;
            rst.set_size(f, n)?;
        }
        pk.write_romfs(&path, &zs::compress_raw(raw, Some(id))?)?;
    }
    for f in &files {
        if f.plain {
            pk.write_romfs(&f.path, &f.raw)?;
        } else {
            pk.write_romfs(&f.path, &zs::compress_raw(&f.raw, dict(&c.rf, &f.path)?)?)?;
        }
        if f.raw.len() > f.old {
            rst.grow(&f.path, f.old as u64, f.raw.len() as u64, MARGIN)?;
        }
    }
    pk.write_romfs(TABLE_PATH, &rst.to_zs()?)?;
    if pk.root != b.root {
        return Err(E::Msg(format!("the files went to {} and not to the seed mod {}", pk.root.display(), b.root.display())));
    }
    Ok(b)
}
