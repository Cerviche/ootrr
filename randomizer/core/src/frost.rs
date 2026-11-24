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
use crate::pickup_pack::tag::{Table, TAG};
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
pub const LOOKS: [&str; 14] = ["Boomerang", "Megaton Hammer", "Lens of Truth", "Iron Boots", "Hover Boots", "Mirror Shield", "Goron Tunic", "Zora Tunic", "Dins Fire", "Farores Wind", "Nayrus Love", "Fire Arrows", "Ice Arrows", "Light Arrows"];
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
const CHILD_SIZE: f32 = 0.65;
const GET_TEXT: &str = "FOOL!";
const FROZEN_CALL: &str = "Condition_Frozen";
const FULL_BODY_CALL: &str = "全身凍結";
const PARAM_AT: usize = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Chest,
    Ground,
    Shop(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trap {
    pub loc: String,
    pub place: Place,
    pub look: String,
    pub actor: String,
    pub fake: String,
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

fn looks(d: &Data, seed: &SeedFile) -> Res<BM<String, String>> {
    let (w, place): (crate::world::World, crate::world::Placement) = crate::slice::starts::rebuild(d, seed)?;
    let at: BM<usize, usize> = spheres(&w, &place);
    let first = |item: &str| -> usize {
        let Some(it) = w.item(item) else { return usize::MAX };
        if w.start.iter().any(|(i, n)| *i == it && *n > 0) {
            return 0;
        }
        w.real.iter().filter(|l| place.get(**l) == Some(it)).map(|l| at.get(l).copied().unwrap_or(usize::MAX)).min().unwrap_or(usize::MAX)
    };
    let mut out: BM<String, String> = BM::new();
    for (loc, item) in &seed.placements {
        if item != TRAP {
            continue;
        }
        let here: usize = w.loc(loc).and_then(|l| at.get(&l).copied()).unwrap_or(usize::MAX);
        let mut later: Vec<&str> = LOOKS.iter().copied().filter(|k| here == usize::MAX || first(k) > here).collect();
        if later.is_empty() {
            later = LOOKS.to_vec();
        }
        let mut r: Rng = Rng::from_text(&format!("{}/{loc}/look", seed.seed));
        out.insert(loc.clone(), later[r.below(later.len() as u64) as usize].to_string());
    }
    Ok(out)
}

pub fn prepare(seed: &SeedFile) -> Res<(SeedFile, Vec<Trap>)> {
    if !seed.placements.values().any(|v| v == TRAP) {
        return Ok((seed.clone(), Vec::new()));
    }
    let d: Data = Data::load()?;
    let chosen: BM<String, String> = looks(&d, seed)?;
    let mut out: SeedFile = seed.clone();
    let mut traps: Vec<Trap> = Vec::new();
    let mut short: Vec<String> = shortfall(&d, seed);
    for (loc, item) in &seed.placements {
        if item != TRAP {
            continue;
        }
        let kind: Option<&str> = d.cat.by_name(loc).map(|l| l.kind.as_str());
        let place: Place = if crate::shops::is_loc(loc) {
            Place::Shop(seed.shops.get(loc).map(|s| s.price).ok_or_else(|| E::Msg(format!("{loc} holds {TRAP} but has no shop entry")))?)
        } else if kind == Some("chest") {
            Place::Chest
        } else if kind.is_some_and(|k| GROUND_KINDS.contains(&k)) {
            Place::Ground
        } else {
            return Err(E::Msg(format!("{loc} holds {TRAP}, which this build can only put in a chest, on the ground or in a shop")));
        };
        let look: String = chosen.get(loc).cloned().ok_or_else(|| E::Msg(format!("{loc} has no look")))?;
        let actor: String = d.item(&look).and_then(|i| i.actor.clone()).ok_or_else(|| E::Msg(format!("{look} has no actor")))?;
        let pick: Option<usize> = short.iter().position(|x| !d.item(x).is_some_and(|i| i.adv)).or(if short.is_empty() { None } else { Some(0) });
        if let Some(k) = pick {
            let held: String = short.remove(k);
            *out.copies.entry(held).or_insert(0) -= 1;
        }
        *out.copies.entry(look.clone()).or_insert(0) += 1;
        out.placements.insert(loc.clone(), look.clone());
        if let (Place::Shop(_), Some(s)) = (place, out.shops.get_mut(loc)) {
            s.item = look.clone();
        }
        traps.push(Trap { loc: loc.clone(), place, look, fake: fake_name(&actor), actor });
    }
    Ok((out, traps))
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

fn tables(c: &mut Ctx, fakes: &[(String, String)], grounds: &[(String, String)]) -> Res<Vec<File>> {
    let firsts: Vec<Vec<u8>> = [ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO].iter().map(|t| c.read(t)).collect::<Res<Vec<Vec<u8>>>>()?;
    let mut ts: [Byml; 3] = [Byml::parse(&firsts[0])?, Byml::parse(&firsts[1])?, Byml::parse(&firsts[2])?];
    add_actor(&mut ts, sensor::SUN, sensor::NAME, None)?;
    for (look, name) in fakes {
        let pouch: String = if rows::find(&ts[2], look).is_some() { look.clone() } else { fake::RUPEE_ROW.to_string() };
        add_actor(&mut ts, look, name, Some(&pouch))?;
    }
    for (orig, g) in grounds {
        let pouch: &str = if rows::find(&ts[2], orig).is_some() { orig } else { fake::RUPEE_ROW };
        let pouch: String = pouch.to_string();
        add_actor(&mut ts, orig, g, Some(&pouch))?;
    }
    let mut out: Vec<File> = Vec::new();
    for (i, t) in [ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO].iter().enumerate() {
        let old: usize = if c.modded { firsts[i].len() } else { c.rf.read_zs(t)?.len() };
        out.push(File { path: t.to_string(), raw: ts[i].write()?, old, plain: false });
    }
    Ok(out)
}

fn tags(raw: &[u8], grounds: &[(String, String)]) -> Res<Vec<u8>> {
    let mut t: Table = Table::parse(raw)?;
    for (orig, g) in grounds {
        let own: Vec<String> = t.tags_of(orig);
        if t.find(g).is_none() && !own.is_empty() {
            t.insert(g, &own)?;
        }
    }
    t.write()
}

fn placed(c: &mut Ctx, d: &Data, loc: &str) -> Res<String> {
    let l = d.cat.by_name(loc).ok_or_else(|| E::Msg(format!("{loc} is not in the location list")))?;
    for r in &l.refs {
        for f in &r.files {
            let y: Byml = Byml::parse(&c.read(f)?)?;
            for a in y.root.get("Actors").and_then(|v| v.as_arr()).unwrap_or(&[]) {
                if crate::placement::hash_of(a) == Some(r.hash) {
                    if let Some(g) = a.get("Gyaml").and_then(|v| v.as_str()) {
                        return Ok(g.to_string());
                    }
                }
            }
        }
    }
    Err(E::Msg(format!("{loc} has no placed actor in its map files")))
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
    for (look, name) in &fakes {
        let lp: Vec<Entry> = c.pack(look)?;
        let f: fake::Fake = fake::make(&lp, look, name)?;
        packs.push((f.name, f.pack, f.fresh, f.peers));
    }
    let mut grounds: Vec<(String, String)> = Vec::new();
    for t in traps.iter().filter(|t| t.place == Place::Ground) {
        let g: String = ground::name_for(&t.fake);
        if grounds.iter().any(|x| x.1 == g) {
            continue;
        }
        let orig: String = placed(&mut c, &d, &t.loc)?;
        let op: Vec<Entry> = c.pack(&orig)?;
        let m: ground::Made = ground::make(&op, &orig, &t.fake)?;
        grounds.push((orig, m.name.clone()));
        packs.push((m.name, m.pack, m.fresh, m.peers));
    }
    let mut files: Vec<File> = tables(&mut c, &fakes, &grounds)?;
    if !grounds.is_empty() {
        files.push(c.file(TAG, |r: &[u8]| tags(r, &grounds))?);
    }
    let mut by_file: BM<String, BM<u64, chest::Swap>> = BM::new();
    for t in traps {
        let swap: chest::Swap = match t.place {
            Place::Chest => chest::Swap::Content(t.fake.clone()),
            Place::Ground => chest::Swap::Actor(ground::name_for(&t.fake)),
            Place::Shop(_) => continue,
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
    let named: Vec<(String, String)> = fakes.clone();
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
