use std::collections::BTreeMap as BM;
use std::path::Path as P;

use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::flow_locations::dict_id;
use crate::msbt::{self, Msbt, Tok};
use crate::pickup_pack::closure::actor_file;
use crate::pickup_pack::rows::{self, ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO};
use crate::pickup_pack::{pack_of, PACK_DICT};
use crate::romfs::Romfs;
use crate::sarc::{self, Entry};
use crate::slice::data::{Data, Item};
use crate::staging::{edit_staged, Out};
use crate::zs;

pub const PREFIX: &str = "Item_AP_";
pub const TEMPLATE: &str = "Item_Medal_Forest";
pub const MESSAGES: &str = "ActorMsg/PouchContent.msbt";
pub const KIND: &str = "Item";
pub const MAX_ACTOR: usize = crate::ap::mailbox::MAX_ACTOR;
pub const TABLES: [&str; 3] = [ACTOR_INFO, GAME_ACTOR_INFO, POUCH_INFO];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foreign {
    pub label: String,
    pub item: String,
    pub player: String,
    pub progression: bool,
}

pub fn actor(label: &str) -> String {
    let h: String = crate::settings::digest::sha256_hex(label.as_bytes());
    format!("{PREFIX}{}", &h[..12])
}

pub fn is_actor(name: &str) -> bool {
    name.len() == PREFIX.len() + 12 && name.starts_with(PREFIX) && name[PREFIX.len()..].bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

pub fn foreign_actor(d: &Data, item: &str) -> Option<String> {
    let i: &Item = d.item(item)?;
    let a: &str = i.actor.as_deref()?;
    (is_actor(a) && actor(item) == a).then(|| a.to_string())
}

fn unique(f: &[Foreign]) -> Res<Vec<&Foreign>> {
    let mut by: BM<String, &Foreign> = BM::new();
    for x in f {
        if x.label.is_empty() || x.item.is_empty() || x.player.is_empty() {
            return Err(E::Msg(format!("ap item: {:?} needs a label, an item and a player", x.label)));
        }
        let a: String = actor(&x.label);
        if a.len() > MAX_ACTOR {
            return Err(E::Msg(format!("ap item: {a} is longer than {MAX_ACTOR} characters, the item get would crash")));
        }
        match by.get(&a) {
            Some(y) if y.label != x.label => return Err(E::Msg(format!("ap item: {:?} and {:?} hash to the same actor {a}", y.label, x.label))),
            Some(y) if *y != x => return Err(E::Msg(format!("ap item: {:?} is given twice with different contents", x.label))),
            _ => {
                by.insert(a, x);
            }
        }
    }
    Ok(by.into_values().collect())
}

pub fn register(d: &mut Data, f: &[Foreign]) -> Res<()> {
    for x in unique(f)? {
        let a: String = actor(&x.label);
        if let Some(i) = d.item(&x.label) {
            if i.actor.as_deref() == Some(a.as_str()) {
                continue;
            }
            return Err(E::Msg(format!("ap item: {:?} is already an OOTRR item, a foreign label cannot share its name", x.label)));
        }
        d.item_ix.insert(x.label.clone(), d.items.len());
        d.items.push(Item {
            name: x.label.clone(),
            kind: KIND.to_string(),
            adv: x.progression,
            actor: Some(a.clone()),
            conf: "exact".to_string(),
            dungeon: None,
            tiers: Vec::new(),
            world: Vec::new(),
        });
        if !d.flow.accept.contains(&a) {
            d.flow.accept.push(a);
        }
    }
    Ok(())
}

pub fn texts(f: &Foreign, lang: &str) -> [(&'static str, String); 4] {
    let (item, player): (&str, &str) = (&f.item, &f.player);
    if lang.ends_with("fr") {
        return [
            ("Name", format!("{item} ({player})")),
            ("Caption", format!("Objet Archipelago de {player}.")),
            ("GetMessage", format!("Vous avez trouvé « {item} » pour {player} !")),
            ("GetCaption", "Il sera envoyé dans son monde.".to_string()),
        ];
    }
    [
        ("Name", format!("{item} ({player})")),
        ("Caption", format!("Archipelago item of {player}.")),
        ("GetMessage", format!("You found \"{item}\" for {player}!")),
        ("GetCaption", "It will be sent to their world.".to_string()),
    ]
}

pub fn lang_of(path: &str) -> &str {
    let file: &str = path.rsplit('/').next().unwrap_or(path);
    file.split('.').next().unwrap_or(file)
}

pub fn add_texts(raw: &[u8], lang: &str, f: &[&Foreign]) -> Res<Vec<u8>> {
    let mut es: Vec<Entry> = sarc::parse(raw)?;
    let b: Vec<u8> = sarc::get(&es, MESSAGES).ok_or_else(|| E::Msg(format!("ap item: the {lang} messages have no {MESSAGES}")))?.data.clone();
    let mut m: Msbt = msbt::parse(&b)?;
    if m.find(&format!("{TEMPLATE}_GetMessage")).is_none() {
        return Err(E::Msg(format!("ap item: the {lang} messages have no {TEMPLATE} get text")));
    }
    for x in f {
        let a: String = actor(&x.label);
        for (sfx, t) in texts(x, lang) {
            m.put_tokens(&format!("{a}_{sfx}"), &[Tok::Text(t)])?;
        }
    }
    let w: Vec<u8> = m.write()?;
    msbt::parse(&w)?;
    sarc::set(&mut es, MESSAGES, w);
    sarc::write(&es)
}

fn s(x: &str) -> Node {
    Node::Str(x.to_string())
}

pub fn name_ref(a: &str) -> String {
    format!("?ActorSystem/ActorName/{a}.engine__actor__ActorName.bgyml")
}

pub fn pack(tpl: &[Entry], a: &str) -> Res<Vec<Entry>> {
    let from: String = actor_file(TEMPLATE);
    let e: &Entry = sarc::get(tpl, &from).ok_or_else(|| E::Msg(format!("ap item: the {TEMPLATE} pack has no {from}")))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    let c: &mut Node = y.root.get_mut("Components").ok_or_else(|| E::Msg(format!("ap item: {from} has no Components")))?;
    if c.get("ActorNameRef").is_none() {
        return Err(E::Msg(format!("ap item: {from} has no ActorNameRef to rename")));
    }
    c.set("ActorNameRef", s(&name_ref(a)));
    let mut out: Vec<Entry> = tpl.iter().filter(|x| x.name != from).cloned().collect();
    out.push(Entry { name: actor_file(a), data: y.write()? });
    Ok(out)
}

fn read_work(rf: &mut Romfs, work: &P, path: &str) -> Res<Vec<u8>> {
    let local = path.split('/').fold(work.to_path_buf(), |a, s| a.join(s));
    let z: Vec<u8> = if local.is_file() { std::fs::read(&local)? } else { rf.read(path)? };
    rf.zs.decompress(&z)
}

pub fn stage(rf: &mut Romfs, work: &P, outs: &mut Vec<Out>, f: &[Foreign]) -> Res<()> {
    let all: Vec<&Foreign> = unique(f)?;
    if all.is_empty() {
        return Ok(());
    }
    let tp: String = pack_of(TEMPLATE);
    let tpl: Vec<Entry> = sarc::parse(&read_work(rf, work, &tp)?)?;
    let id: u32 = dict_id(&rf.read(&tp)?).unwrap_or(PACK_DICT);
    for x in &all {
        let a: String = actor(&x.label);
        let p: String = pack_of(&a);
        if rf.root.join(&p).is_file() || outs.iter().any(|o| o.path == p) {
            return Err(E::Msg(format!("ap item: {p} exists already")));
        }
        let raw: Vec<u8> = sarc::write(&pack(&tpl, &a)?)?;
        outs.push(Out { path: p, z: zs::compress_raw(&raw, Some(id))?, old: 0, new: raw.len() });
    }
    for (i, t) in TABLES.iter().enumerate() {
        edit_staged(rf, outs, work, t, true, |raw| {
            let mut y: Byml = Byml::parse(raw)?;
            for x in &all {
                add_row(&mut y, i, &actor(&x.label))?;
            }
            y.write()
        })?;
    }
    for l in crate::hints::languages(rf)? {
        let lang: String = lang_of(&l).to_string();
        edit_staged(rf, outs, work, &l, true, |raw| add_texts(raw, &lang, &all))?;
    }
    Ok(())
}

pub fn add_row(t: &mut Byml, i: usize, a: &str) -> Res<()> {
    let name: &str = TABLES[i];
    if rows::find(t, a).is_some() {
        return Err(E::Msg(format!("ap item: {a} already has a row in {name}")));
    }
    let mut r: Node = rows::pouch_row(rows::need(t, TEMPLATE, name)?, a);
    if i == 0 {
        r.set(rows::ACTOR_NAME, s(a));
    }
    rows::insert(t, r)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfevfl::{Bf, Kind};
    use crate::settings::seed::SeedFile;
    use crate::settings::Settings;
    use crate::slice::generate::{generate_data, GenOpts};
    use std::path::PathBuf as PB;

    fn fx(label: &str, item: &str, player: &str) -> Foreign {
        Foreign { label: label.to_string(), item: item.to_string(), player: player.to_string(), progression: true }
    }

    #[test]
    fn the_actor_is_twenty_characters_of_the_label_hash() {
        let a: String = actor("AP: Hookshot (Bob)");
        assert_eq!(a.len(), 20);
        assert!(a.len() <= MAX_ACTOR);
        assert!(is_actor(&a));
        assert_eq!(a, actor("AP: Hookshot (Bob)"));
        assert_ne!(a, actor("AP: Hookshot (Ami)"));
        assert_eq!(actor("abc"), "Item_AP_ba7816bf8f01");
        assert!(!is_actor("Item_AP_BA7816BF8F01"));
        assert!(!is_actor("Item_Medal_Forest"));
    }

    #[test]
    fn texts_name_the_item_and_its_player() {
        let f: Foreign = fx("AP: Hookshot (Bob)", "Hookshot", "Bob");
        let en = texts(&f, "USen");
        assert_eq!(en[0], ("Name", "Hookshot (Bob)".to_string()));
        assert_eq!(en[2], ("GetMessage", "You found \"Hookshot\" for Bob!".to_string()));
        assert_eq!(en[3].1, "It will be sent to their world.");
        assert!(texts(&f, "EUfr")[2].1.starts_with("Vous avez trouvé"));
        assert_eq!(texts(&f, "JPja"), en);
        assert_eq!(lang_of("Mals/EUfr.Product.110.sarc.zs"), "EUfr");
    }

    #[test]
    fn register_adds_one_item_per_label_and_refuses_clashes() {
        let mut d: Data = Data::load().unwrap();
        let n: usize = d.items.len();
        let f: Vec<Foreign> = vec![fx("AP: Hookshot (Bob)", "Hookshot", "Bob"), fx("AP: Bow (Ami)", "Bow", "Ami"), fx("AP: Bow (Ami)", "Bow", "Ami")];
        register(&mut d, &f).unwrap();
        assert_eq!(d.items.len(), n + 2);
        let i: &Item = d.item("AP: Bow (Ami)").unwrap();
        assert_eq!(i.actor.as_deref(), Some(actor("AP: Bow (Ami)").as_str()));
        assert!(i.tiers.is_empty() && i.dungeon.is_none());
        assert!(d.flow.usable(&actor("AP: Bow (Ami)")));
        assert_eq!(foreign_actor(&d, "AP: Bow (Ami)"), Some(actor("AP: Bow (Ami)")));
        assert_eq!(foreign_actor(&d, "Progressive Hookshot"), None);
        register(&mut d, &f).unwrap();
        assert_eq!(d.items.len(), n + 2);
        assert!(register(&mut d, &[fx("Progressive Hookshot", "x", "y")]).is_err());
        assert!(register(&mut d, &[fx("AP: Bow (Ami)", "Bow", "Ami"), fx("AP: Bow (Ami)", "Bow", "Bea")]).is_err());
        assert!(register(&mut d, &[fx("AP: x", "", "Ami")]).is_err());
    }

    fn root() -> Option<PB> {
        let p: PB = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(p)
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    fn once(seed: &SeedFile, item: &str) -> bool {
        seed.placements.values().filter(|v| *v == item).count() == 1
    }

    fn has_str(n: &Node, want: &str) -> bool {
        match n {
            Node::Str(x) => x == want,
            Node::Arr(v) => v.iter().any(|x| has_str(x, want)),
            Node::Map(v) => v.iter().any(|(_, x)| has_str(x, want)),
            _ => false,
        }
    }

    fn mod_file(rf: &mut Romfs, romfs: &P, path: &str) -> Vec<u8> {
        let z: Vec<u8> = std::fs::read(path.split('/').fold(romfs.to_path_buf(), |a, s| a.join(s))).unwrap_or_else(|_| panic!("{path} missing from the mod"));
        if path.ends_with(".zs") { rf.zs.decompress(&z).unwrap() } else { z }
    }

    fn walk(dir: &P, base: &P, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p: PB = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                out.push(p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }

    fn pick(d: &Data, seed: &SeedFile) -> (String, String) {
        let wrapped = crate::slice::flow_wrap::wraps(d, seed);
        let chest: String = seed
            .placements
            .iter()
            .filter(|(l, _)| d.cat.by_name(l).is_some_and(|c| c.kind == "chest" && c.refs.iter().all(|r| !d.events.keep.contains(&r.hash))) && d.state_of(l) == Some(crate::slice::data::State::Eligible))
            .filter(|(_, i)| once(seed, i) && d.family(i).is_none())
            .map(|(l, _)| l.clone())
            .next()
            .unwrap();
        let npc: String = seed
            .placements
            .iter()
            .filter(|(l, i)| crate::slice::edit::is_sale(d, l) && d.cat.by_name(l).is_none() && d.flow_ok(l) && !wrapped.contains_key(*l) && !crate::flow_extra::scripted(l) && once(seed, i) && d.family(i).is_none())
            .map(|(l, _)| l.clone())
            .next()
            .unwrap();
        (chest, npc)
    }

    #[test]
    #[ignore]
    fn a_label_builds_at_a_chest_and_at_an_npc() {
        let Some(r) = root() else { return };
        let mut d: Data = Data::load().unwrap();
        let g = generate_data(&d, &Settings::default(), "ap item build", &GenOpts::default()).unwrap();
        let mut seed: SeedFile = g.seed_file.clone();
        let (chest, npc): (String, String) = pick(&d, &seed);
        let f: Vec<Foreign> = vec![fx("AP: Hookshot (Bob)", "Hookshot", "Bob"), Foreign { progression: false, ..fx("AP: Rupees (Ami)", "Rupees", "Ami") }];
        seed.placements.insert(chest.clone(), f[0].label.clone());
        seed.placements.insert(npc.clone(), f[1].label.clone());
        register(&mut d, &f).unwrap();
        let order = crate::slice::tier_order::fixed_actors(&d, &seed).unwrap();
        let wants = crate::slice::seed_packs::need(&d, &seed, &order.assign).unwrap();
        let extra: crate::pickup_persist::Extra = if wants.is_empty() { Default::default() } else { crate::slice::seed_packs::pool(&r, crate::modpack::Host::Eden, &wants).unwrap() };
        let load: PB = PB::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ap_item_mods");
        let _ = std::fs::remove_dir_all(&load);
        let mut staged: usize = 0;
        let mut hook = |rf: &mut Romfs, work: &P, outs: &mut Vec<Out>| -> Res<()> {
            let n: usize = outs.len();
            stage(rf, work, outs, &f)?;
            staged = outs.len() - n;
            Ok(())
        };
        let b = crate::slice::edit::build_data(d.clone(), &seed, &r, &load, "ootrr-ap-item", crate::modpack::Host::Eden, &extra, Some(&mut hook)).unwrap();
        assert!(staged >= 2 + 3);
        let romfs: PB = b.root.join("romfs");
        let mut rf: Romfs = Romfs::open(&r).unwrap();
        let rst: crate::rsizetable::Rst = crate::rsizetable::Rst::parse(&mod_file(&mut rf, &romfs, crate::rsizetable::TABLE_PATH)).unwrap();
        let ts: Vec<Byml> = TABLES.iter().map(|t| Byml::parse(&mod_file(&mut rf, &romfs, t)).unwrap()).collect();
        for (i, t) in ts.iter().enumerate() {
            assert!(rows::sorted(t).unwrap(), "{}", TABLES[i]);
        }
        let langs: Vec<String> = crate::hints::languages(&rf).unwrap();
        assert_eq!(langs.len(), 16);
        for x in &f {
            let a: String = actor(&x.label);
            let p: String = pack_of(&a);
            let es: Vec<Entry> = sarc::parse(&mod_file(&mut rf, &romfs, &p)).unwrap();
            let ap: Byml = Byml::parse(&sarc::get(&es, &actor_file(&a)).unwrap().data).unwrap();
            assert_eq!(ap.root.get("Components").and_then(|c| c.get("ActorNameRef")).and_then(|v| v.as_str()), Some(name_ref(&a).as_str()));
            assert!(sarc::get(&es, &actor_file(TEMPLATE)).is_none());
            for e in &es {
                assert!(rst.get(&e.name).is_some(), "{} has no size", e.name);
                if e.name.ends_with(".bgyml") {
                    Byml::parse(&e.data).unwrap();
                }
            }
            assert!(rst.get(&p).is_some());
            for (i, t) in ts.iter().enumerate() {
                let row: &Node = rows::find(t, &a).unwrap_or_else(|| panic!("{a} has no row in {}", TABLES[i]));
                let mut same: Node = row.clone();
                same.set(rows::ROW_ID, s(TEMPLATE));
                if i == 0 {
                    assert_eq!(row.get(rows::ACTOR_NAME).and_then(|v| v.as_str()), Some(a.as_str()));
                    same.set(rows::ACTOR_NAME, s(TEMPLATE));
                }
                assert_eq!(Some(&same), rows::find(t, TEMPLATE));
            }
            for l in &langs {
                let es: Vec<Entry> = sarc::parse(&mod_file(&mut rf, &romfs, l)).unwrap();
                let m: Msbt = msbt::parse(&sarc::get(&es, MESSAGES).unwrap().data).unwrap();
                for (sfx, t) in texts(x, lang_of(l)) {
                    assert_eq!(m.text(&format!("{a}_{sfx}")).unwrap(), vec![Tok::Text(t)], "{l} {sfx}");
                }
            }
        }
        let a0: String = actor(&f[0].label);
        let line = b.chests.iter().find(|c| c.name == chest).unwrap_or_else(|| panic!("{chest} is not among the edited chests"));
        assert_eq!(line.new_actor, a0);
        let c = d.cat.by_name(&chest).unwrap();
        for at in &c.refs {
            for file in &at.files {
                let y: Byml = Byml::parse(&mod_file(&mut rf, &romfs, file)).unwrap();
                let hit: &Node = y.root.get("Actors").and_then(|v| v.as_arr()).unwrap().iter().find(|n| matches!(n.get("Hash"), Some(Node::U64(h)) if *h == at.hash)).unwrap();
                assert!(has_str(hit, &a0), "{chest} in {file} does not drop {a0}");
            }
        }
        let a1: String = actor(&f[1].label);
        let l = d.flow.by_name(&npc).unwrap();
        let mut gave: bool = false;
        for e in &l.edits {
            let b: Bf = Bf::parse(&mod_file(&mut rf, &romfs, &crate::flow_locations::path_of(&e.file))).unwrap();
            gave |= b.flows.iter().flat_map(|fl| fl.events.iter()).any(|ev| match &ev.kind {
                Kind::Action { params, .. } | Kind::Sub { params, .. } => params.as_ref().is_some_and(|p| p.items.iter().any(|(_, v)| v.strings().contains(&a1.as_str()))),
                _ => false,
            });
        }
        assert!(gave, "{npc} does not give {a1}");
        let mut all: Vec<String> = Vec::new();
        walk(&romfs, &romfs, &mut all);
        let mut parsed: usize = 0;
        for p in &all {
            if p.ends_with(".byml.zs") || p.ends_with(".bgyml") {
                Byml::parse(&mod_file(&mut rf, &romfs, p)).unwrap_or_else(|e| panic!("{p}: {e:?}"));
                parsed += 1;
            } else if p.ends_with(".bfevfl.zs") {
                Bf::parse(&mod_file(&mut rf, &romfs, p)).unwrap_or_else(|e| panic!("{p}: {e:?}"));
                parsed += 1;
            }
        }
        assert!(parsed > 10, "{parsed}");
        eprintln!("chest {chest}, npc {npc}, {} files in the mod, {parsed} parsed back", all.len());
        let _ = std::fs::remove_dir_all(&load);
    }
}
