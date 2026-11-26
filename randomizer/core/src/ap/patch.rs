use std::collections::{BTreeMap as BM, BTreeSet as BS};

use crate::error::{Error as E, Res};
use crate::json::Json;
use crate::logic_items::ItemId;
use crate::search::{explore, Search};
use crate::settings::seed::SeedFile;
use crate::settings::Settings;
use crate::slice::data::Data;
use crate::slice::generate::{prepare, Prep};
use crate::world::{LocId, Placement, World};

use super::item::Foreign;
use super::mailbox::Record;

pub const FORMAT: &str = "ootrr-ap-patch";
pub const VERSION: i64 = 1;
pub const EXT: &str = "apootrr";
pub const MAX_TEXT: usize = 512;
pub const MAX_ROWS: usize = 200_000;
pub const LABEL: &str = "AP: ";

#[derive(Debug, Clone, PartialEq)]
pub struct Remote {
    pub key: String,
    pub item: String,
    pub player: String,
    pub location: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub preset: String,
    pub settings: String,
    pub seed: String,
    pub seed_name: String,
    pub slot: String,
    pub seed_key: String,
    pub placements: BM<String, String>,
    pub foreign: BM<String, Foreign>,
    pub remote: Vec<Remote>,
    pub start: Vec<(String, String)>,
    pub location_ids: BM<String, i64>,
}

fn bad(m: &str) -> E {
    E::Msg(format!("archipelago patch: {m}"))
}

fn text(j: &Json, k: &str) -> Res<String> {
    let s: &str = j.get(k).and_then(|v| v.as_str()).ok_or_else(|| bad(&format!("{k} is missing or not text")))?;
    if s.is_empty() || s.len() > MAX_TEXT || s.chars().any(|c| c.is_control()) {
        return Err(bad(&format!("{k} is empty, too long or holds control characters")));
    }
    Ok(s.to_string())
}

fn obj<'a>(j: &'a Json, k: &str) -> Res<&'a [(String, Json)]> {
    let o: &[(String, Json)] = j.get(k).and_then(|v| v.as_obj()).ok_or_else(|| bad(&format!("{k} is missing or not an object")))?;
    if o.len() > MAX_ROWS {
        return Err(bad(&format!("{k} has too many entries")));
    }
    Ok(o)
}

fn arr<'a>(j: &'a Json, k: &str) -> Res<&'a [Json]> {
    let a: &[Json] = j.get(k).and_then(|v| v.as_arr()).ok_or_else(|| bad(&format!("{k} is missing or not a list")))?;
    if a.len() > MAX_ROWS {
        return Err(bad(&format!("{k} has too many entries")));
    }
    Ok(a)
}

fn key_ok(k: &str) -> Res<()> {
    if k.len() != 12 || !k.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        return Err(bad(&format!("the key {k:?} is not 12 lowercase hex digits")));
    }
    Ok(())
}

impl Patch {
    pub fn parse(t: &str) -> Res<Patch> {
        let j: Json = crate::json::parse(t)?;
        if j.get("format").and_then(|v| v.as_str()) != Some(FORMAT) {
            return Err(bad("this is not an OOTRR Archipelago patch"));
        }
        let v: i64 = j.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
        if v != VERSION {
            return Err(bad(&format!("version {v} is not supported, this build reads version {VERSION}")));
        }
        if j.get("game").and_then(|v| v.as_str()) != Some(super::export::GAME) {
            return Err(bad("the patch is for another game"));
        }
        let mut placements: BM<String, String> = BM::new();
        for (k, v) in obj(&j, "placements")? {
            let item: &str = v.as_str().ok_or_else(|| bad(&format!("the placement of {k} is not text")))?;
            if k.len() > MAX_TEXT || item.is_empty() || item.len() > MAX_TEXT || item.chars().any(|c| c.is_control()) {
                return Err(bad(&format!("the placement of {k} is not usable")));
            }
            placements.insert(k.clone(), item.to_string());
        }
        let mut foreign: BM<String, Foreign> = BM::new();
        for (k, v) in obj(&j, "foreign")? {
            let label: &str = placements.get(k).ok_or_else(|| bad(&format!("{k} holds another player's item but has no placement")))?;
            if !label.starts_with(LABEL) {
                return Err(bad(&format!("the placement of {k} does not name another player's item")));
            }
            foreign.insert(
                k.clone(),
                Foreign { label: label.to_string(), item: text(v, "item")?, player: text(v, "player")?, progression: v.get("progression").and_then(|x| x.as_bool()).unwrap_or(false) },
            );
        }
        for (k, v) in &placements {
            if v.starts_with(LABEL) && !foreign.contains_key(k) {
                return Err(bad(&format!("{k} names another player's item that the patch does not describe")));
            }
        }
        let mut remote: Vec<Remote> = Vec::new();
        let mut keys: BS<String> = BS::new();
        for r in arr(&j, "remote")? {
            let key: String = text(r, "key")?;
            key_ok(&key)?;
            if !keys.insert(key.clone()) {
                return Err(bad(&format!("the key {key} is used twice")));
            }
            remote.push(Remote { key, item: text(r, "item")?, player: text(r, "player")?, location: text(r, "location")? });
        }
        let mut start: Vec<(String, String)> = Vec::new();
        for r in arr(&j, "start")? {
            let key: String = text(r, "key")?;
            key_ok(&key)?;
            if !keys.insert(key.clone()) {
                return Err(bad(&format!("the key {key} is used twice")));
            }
            start.push((key, text(r, "item")?));
        }
        let mut location_ids: BM<String, i64> = BM::new();
        for (k, v) in obj(&j, "location_ids")? {
            location_ids.insert(k.clone(), v.as_i64().ok_or_else(|| bad(&format!("the id of {k} is not a number")))?);
        }
        let seed_key: String = text(&j, "seed_key")?;
        key_ok(&seed_key)?;
        Ok(Patch {
            preset: text(&j, "preset")?,
            settings: text(&j, "settings")?,
            seed: text(&j, "seed")?,
            seed_name: text(&j, "seed_name")?,
            slot: text(&j, "slot")?,
            seed_key,
            placements,
            foreign,
            remote,
            start,
            location_ids,
        })
    }

    pub fn records(&self, d: &Data) -> Res<Vec<Record>> {
        let mut out: Vec<Record> = Vec::with_capacity(self.remote.len() + self.start.len());
        for r in &self.remote {
            out.push(Record::of(d, &r.key, &r.item).map_err(|e| bad(&format!("{} from {} at {}: {e}", r.item, r.player, r.location)))?);
        }
        for (k, item) in &self.start {
            out.push(Record::of(d, k, item).map_err(|e| bad(&format!("the starting item {item}: {e}")))?);
        }
        Ok(out)
    }

    pub fn foreign_list(&self) -> Vec<Foreign> {
        let mut seen: BS<String> = BS::new();
        self.foreign.values().filter(|f| seen.insert(f.label.clone())).cloned().collect()
    }
}

pub struct Checked {
    pub seed: SeedFile,
    pub settings: Settings,
    pub won: bool,
    pub notes: Vec<String>,
}

fn held(w: &World, names: &[&str]) -> Res<Vec<ItemId>> {
    names.iter().map(|n| w.item(n).ok_or_else(|| bad(&format!("{n} is not an item of this world")))).collect()
}

pub fn check(d: &Data, p: &Patch) -> Res<Checked> {
    let loaded = Settings::decode(&p.settings)?;
    let s: Settings = loaded.settings;
    if crate::slice::no_logic::enabled(&s)? {
        return Err(bad("No Logic is not supported in Archipelago yet"));
    }
    if s.text("item_pool")? != "balanced" {
        return Err(bad("Scarce and Minimal item pools are not supported in Archipelago yet"));
    }
    let Prep { w, vanilla, pk, gates, lay, .. } = prepare(d, &s, &p.seed, &[])?;
    if gates.is_some() {
        return Err(bad("entrance shuffle is not supported yet"));
    }
    let free: BS<&str> = pk.free.iter().map(|l| w.locs[*l].name.as_str()).collect();
    let given: BS<&str> = p.placements.keys().map(|k| k.as_str()).collect();
    if free != given {
        let extra: Vec<&&str> = given.difference(&free).take(5).collect();
        let missing: Vec<&&str> = free.difference(&given).take(5).collect();
        return Err(bad(&format!(
            "the patch does not match this build's shuffled locations for its settings (not shuffled here: {extra:?}, missing from the patch: {missing:?}). Update OOTRR or the apworld so both are the same version"
        )));
    }
    let mut place: Placement = Placement::new(w.locs.len());
    for l in w.real.iter().filter(|l| !pk.free.contains(l)) {
        if let Some(i) = vanilla.get(*l) {
            place.put(*l, i);
        }
    }
    let mut own: BM<String, String> = BM::new();
    for (loc, item) in &p.placements {
        let l: LocId = w.loc(loc).ok_or_else(|| bad(&format!("{loc} is not a location of this world")))?;
        if p.foreign.contains_key(loc) {
            own.insert(loc.clone(), item.clone());
            continue;
        }
        let i: ItemId = w.item(item).ok_or_else(|| bad(&format!("{item} at {loc} is not an item of this world")))?;
        place.put(l, i);
        own.insert(loc.clone(), item.clone());
    }
    let mut arriving: Vec<&str> = p.remote.iter().map(|r| r.item.as_str()).collect();
    arriving.extend(p.start.iter().map(|(_, i)| i.as_str()));
    let extra: Vec<ItemId> = held(&w, &arriving)?;
    let sr: Search = explore(&w, &place, &extra);
    let won: bool = sr.won();
    let mut notes: Vec<String> = Vec::new();
    if !won {
        notes.push("OOTRR's own search cannot beat this world even with every item from the other worlds. The apworld and this build disagree on the logic.".to_string());
    }
    let mut seed: SeedFile = SeedFile::new(&p.seed, &s, &d.scan.game, own)?;
    if let Some(mut l) = lay {
        crate::shops::layout::fill_deals(&mut l, &seed.placements)?;
        seed.shops = crate::shops::seed::entries(&l);
    }
    Ok(Checked { seed, settings: s, won, notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &str = r#"{"format":"ootrr-ap-patch","version":1,"game":"OOTRR","preset":"default","settings":"x","seed":"archipelago","seed_name":"1","player":1,"slot":"Link","seed_key":"0123456789ab",
        "placements":{"KF Midos Top Left Chest":"AP: 0123456789ab","KF Midos Top Right Chest":"Slingshot"},
        "foreign":{"KF Midos Top Left Chest":{"item":"Key","player":"Quester","game":"APQuest","progression":true}},
        "remote":[{"key":"aaaaaaaaaaaa","item":"Boomerang","player":"Quester","location":"Top Left Chest"}],
        "start":[{"key":"bbbbbbbbbbbb","item":"Deku Shield"}],"location_ids":{"KF Midos Top Left Chest":12}}"#;

    #[test]
    fn a_patch_reads_back_its_parts_and_refuses_bad_ones() {
        let p: Patch = Patch::parse(SMALL).unwrap();
        assert_eq!(p.foreign["KF Midos Top Left Chest"].label, "AP: 0123456789ab");
        assert!(p.foreign["KF Midos Top Left Chest"].progression);
        assert_eq!(p.remote[0].key, "aaaaaaaaaaaa");
        assert_eq!(p.start, vec![("bbbbbbbbbbbb".to_string(), "Deku Shield".to_string())]);
        let d: Data = Data::load().unwrap();
        let r: Vec<Record> = p.records(&d).unwrap();
        assert_eq!(r.len(), 2);
        assert!(Patch::parse(&SMALL.replace("ootrr-ap-patch", "other")).is_err());
        assert!(Patch::parse(&SMALL.replace("\"version\":1", "\"version\":9")).is_err());
        assert!(Patch::parse(&SMALL.replace("bbbbbbbbbbbb", "aaaaaaaaaaaa")).is_err());
        assert!(Patch::parse(&SMALL.replace("\"Slingshot\"", "\"AP: Slingshot (X)\"")).is_err());
    }

    fn solo(s: &Settings, g: &crate::slice::generate::Generated) -> Patch {
        Patch {
            preset: "custom".to_string(),
            settings: s.encode().unwrap(),
            seed: super::super::export::SEED.to_string(),
            seed_name: "1".to_string(),
            slot: "Link".to_string(),
            seed_key: "0123456789ab".to_string(),
            placements: g.seed_file.placements.clone(),
            foreign: BM::new(),
            remote: Vec::new(),
            start: Vec::new(),
            location_ids: BM::new(),
        }
    }

    fn shop_settings(level: &str) -> Settings {
        let mut s: Settings = Settings::default();
        s.set("shuffle_shops", crate::settings::model::Value::s(level)).unwrap();
        s
    }

    #[test]
    fn a_shopsanity_slot_gets_the_generators_shop_table_and_passes_the_shop_check() {
        use crate::slice::generate::{generate_data, GenOpts};
        let d: Data = Data::load().unwrap();
        for level in ["deals_1", "shuffle"] {
            let s: Settings = shop_settings(level);
            let g = generate_data(&d, &s, super::super::export::SEED, &GenOpts::default()).unwrap();
            assert_eq!(g.seed_file.shops.len(), 64, "{level}");
            let c: Checked = check(&d, &solo(&s, &g)).unwrap();
            assert_eq!(c.seed.shops, g.seed_file.shops, "{level}");
            let o = crate::slice::support::plan(&s).unwrap().shops;
            crate::shops::seed::check(&o, &c.seed).unwrap();
        }
    }

    #[test]
    fn a_default_slot_has_no_shop_table() {
        use crate::slice::generate::{generate_data, GenOpts};
        let d: Data = Data::load().unwrap();
        let s: Settings = Settings::default();
        let g = generate_data(&d, &s, super::super::export::SEED, &GenOpts::default()).unwrap();
        assert!(check(&d, &solo(&s, &g)).unwrap().seed.shops.is_empty());
    }
}
