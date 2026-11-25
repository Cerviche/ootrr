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
/// The longest name of an actor the mod adds. Longer new names were seen to crash the item get.
pub const MAX_ACTOR: usize = 25;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Plain,
    Logo,
}

pub const SPIN_PACK: &str = "Item_Wallet_Medium";
pub const SPIN_AS: &str = "?AS/Item_Common.root.asb";
pub const SPIN_INFO: &str = "Component/ASInfo/Item_GetDemo.engine__component__ASInfo.bgyml";
pub const SPIN_ANIM: &str = "Item_AP_Spin";
pub const MODEL_INFO_TPL: &str = "Component/ModelInfo/Item_Medal_Forest.engine__component__ModelInfo.bgyml";

pub fn model_info_file() -> String {
    format!("Component/ModelInfo/{}.engine__component__ModelInfo.bgyml", logo::PROJECT)
}

pub struct Parts {
    pub look: Look,
    pub extra: Vec<Entry>,
    pub comps: Vec<(&'static str, String)>,
}

pub fn plain() -> Parts {
    Parts { look: Look::Plain, extra: Vec::new(), comps: Vec::new() }
}

pub fn logo_parts(tpl: &[Entry], spin: &[Entry]) -> Res<Parts> {
    let info: &Entry = sarc::get(spin, SPIN_INFO).ok_or_else(|| E::Msg(format!("ap item: the {SPIN_PACK} pack has no {SPIN_INFO}")))?;
    let m: &Entry = sarc::get(tpl, MODEL_INFO_TPL).ok_or_else(|| E::Msg(format!("ap item: the {TEMPLATE} pack has no {MODEL_INFO_TPL}")))?;
    let mut y: Byml = Byml::parse(&m.data)?;
    for (k, v) in [("FmdbName", logo::FMDB), ("ModelProjectName", logo::PROJECT)] {
        if !y.root.set(k, s(v)) {
            return Err(E::Msg(format!("ap item: {MODEL_INFO_TPL} has no {k}")));
        }
    }
    let (anim_ref, anim_file, anim): (String, String, Vec<u8>) = crate::pickup_pack::model::anim(SPIN_ANIM, y.version, None)?;
    let mi: String = model_info_file();
    Ok(Parts {
        look: Look::Logo,
        extra: vec![info.clone(), Entry { name: anim_file, data: anim }, Entry { name: mi.clone(), data: y.write()? }],
        comps: vec![("ModelInfoRef", format!("?{mi}")), ("ASRef", SPIN_AS.to_string()), ("ASInfoRef", format!("?{SPIN_INFO}")), ("AnimationRef", anim_ref)],
    })
}

pub fn pack(tpl: &[Entry], a: &str, parts: &Parts) -> Res<Vec<Entry>> {
    let from: String = actor_file(TEMPLATE);
    let e: &Entry = sarc::get(tpl, &from).ok_or_else(|| E::Msg(format!("ap item: the {TEMPLATE} pack has no {from}")))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    let c: &mut Node = y.root.get_mut("Components").ok_or_else(|| E::Msg(format!("ap item: {from} has no Components")))?;
    if c.get("ActorNameRef").is_none() {
        return Err(E::Msg(format!("ap item: {from} has no ActorNameRef to rename")));
    }
    c.set("ActorNameRef", s(&name_ref(a)));
    for (k, v) in &parts.comps {
        c.set(k, s(v));
    }
    let mut out: Vec<Entry> = tpl.iter().filter(|x| x.name != from && !parts.extra.iter().any(|n| n.name == x.name)).cloned().collect();
    out.extend(parts.extra.iter().cloned());
    out.push(Entry { name: actor_file(a), data: y.write()? });
    Ok(out)
}

fn read_work(rf: &mut Romfs, work: &P, path: &str) -> Res<Vec<u8>> {
    let local = path.split('/').fold(work.to_path_buf(), |a, s| a.join(s));
    let z: Vec<u8> = if local.is_file() { std::fs::read(&local)? } else { rf.read(path)? };
    rf.zs.decompress(&z)
}

pub fn stage(rf: &mut Romfs, work: &P, outs: &mut Vec<Out>, f: &[Foreign]) -> Res<()> {
    stage_look(rf, work, outs, f, Look::Logo).map(|_| ())
}

fn logo_files(rf: &mut Romfs, work: &P, tpl: &[Entry]) -> Res<(Parts, Vec<Out>)> {
    let spin: Vec<Entry> = sarc::parse(&read_work(rf, work, &pack_of(SPIN_PACK))?)?;
    let parts: Parts = logo_parts(tpl, &spin)?;
    let made: Vec<(String, Vec<u8>, String)> = logo::files(&mut |p: &str| read_work(rf, work, p))?;
    let mut outs: Vec<Out> = Vec::new();
    for (p, raw, src) in made {
        let id: Option<u32> = dict_id(&rf.read(&src)?);
        outs.push(Out { path: p, z: zs::compress_raw(&raw, id)?, old: 0, new: raw.len() });
    }
    Ok((parts, outs))
}

pub fn stage_look(rf: &mut Romfs, work: &P, outs: &mut Vec<Out>, f: &[Foreign], want: Look) -> Res<Look> {
    let all: Vec<&Foreign> = unique(f)?;
    if all.is_empty() {
        return Ok(want);
    }
    let tp: String = pack_of(TEMPLATE);
    let tpl: Vec<Entry> = sarc::parse(&read_work(rf, work, &tp)?)?;
    let id: u32 = dict_id(&rf.read(&tp)?).unwrap_or(PACK_DICT);
    let (parts, model): (Parts, Vec<Out>) = match want {
        Look::Plain => (plain(), Vec::new()),
        Look::Logo => logo_files(rf, work, &tpl).unwrap_or_else(|_| (plain(), Vec::new())),
    };
    for o in model {
        if !outs.iter().any(|x| x.path == o.path) {
            outs.push(o);
        }
    }
    for x in &all {
        let a: String = actor(&x.label);
        let p: String = pack_of(&a);
        if rf.root.join(&p).is_file() || outs.iter().any(|o| o.path == p) {
            return Err(E::Msg(format!("ap item: {p} exists already")));
        }
        let raw: Vec<u8> = sarc::write(&pack(&tpl, &a, &parts)?)?;
        outs.push(Out { path: p, z: zs::compress_raw(&raw, Some(id))?, old: 0, new: raw.len() });
    }
    let look: Look = parts.look;
    for (i, t) in TABLES.iter().enumerate() {
        edit_staged(rf, outs, work, t, true, |raw| {
            let mut y: Byml = Byml::parse(raw)?;
            for x in &all {
                add_row(&mut y, i, &actor(&x.label), look)?;
            }
            y.write()
        })?;
    }
    for l in crate::hints::languages(rf)? {
        let lang: String = lang_of(&l).to_string();
        edit_staged(rf, outs, work, &l, true, |raw| add_texts(raw, &lang, &all))?;
    }
    Ok(look)
}

fn heap_of(r: &Node) -> Option<i64> {
    match r.get(rows::HEAP) {
        Some(Node::I32(v)) => Some(*v as i64),
        Some(Node::U32(v)) => Some(*v as i64),
        _ => None,
    }
}

pub fn add_row(t: &mut Byml, i: usize, a: &str, look: Look) -> Res<()> {
    let name: &str = TABLES[i];
    if rows::find(t, a).is_some() {
        return Err(E::Msg(format!("ap item: {a} already has a row in {name}")));
    }
    let mut r: Node = rows::pouch_row(rows::need(t, TEMPLATE, name)?, a);
    if i == 0 {
        r.set(rows::ACTOR_NAME, s(a));
        if look == Look::Logo {
            let m: &Node = rows::need(t, logo::DONOR, name)?;
            for k in ["FmdbName", "ModelAabbMax", "ModelAabbMin"] {
                if let Some(v) = m.get(k) {
                    r.set(k, v.clone());
                }
            }
            r.set("ModelProjectName", s(logo::PROJECT));
            r.set("AnimationResources", crate::pickup_pack::model::shared_list());
            if let (Some(h), Some(mh)) = (heap_of(&r), heap_of(m))
                && mh > h
            {
                let v: Node = r.get(rows::HEAP).cloned().unwrap_or(Node::I32(0));
                r.set(rows::HEAP, if matches!(v, Node::U32(_)) { Node::U32(mh as u32) } else { Node::I32(mh as i32) });
            }
        }
    }
    rows::insert(t, r)?;
    Ok(())
}

pub fn size_peer(path: &str) -> Option<String> {
    logo::source_of(path)
}

pub mod logo {
    use crate::error::{Error as E, Res};

    pub const DONOR: &str = "Item_Medal_Fire";
    pub const FMDB: &str = "Item_Medal_Fire";
    pub const PROJECT: &str = "Item_Medal_Arch";
    pub const MODEL: &str = "Model/Item_Medal_Fire.Item_Medal_Fire.bfres.zs";
    pub const OLD_TEX: &str = "Mt_Item_Medal_Fire";
    pub const NEW_TEX: &str = "Mt_Item_Medal_Arch";
    pub const ALB_SOURCE: &str = "Tex/Mt_Upper_Enemy_Poe_Alb.bntx.zs";
    pub const SUFFIXES: [&str; 6] = ["Alb", "AO", "Emc", "Mtl", "Nrm", "Rgh"];
    pub const HIDDEN: [&str; 3] = ["Glass_Back__Mt_Medal_Glass_Back", "Glass__Mt_Medal_Glass", "Glaze__Mt_Medal_Glaze"];
    pub const RELIEF: (&str, f64, f64) = ("Medal_Base__Mt_Item_Medal_Fire", 0.0095, 0.105);
    pub const SHEET: usize = 576;
    pub const FACES: [(f64, f64); 2] = [(153.0, 152.0), (377.0, 375.0)];
    pub const FACE_RADIUS: f64 = 134.0;
    pub const RIM: f64 = 0.86;
    pub const CREAM: [u8; 3] = [246, 239, 222];
    pub const GOLD: [u8; 3] = [214, 168, 74];
    pub const GOLD_DARK: [u8; 3] = [150, 104, 38];
    pub const FLAT_NORMAL: [u8; 4] = [128, 128, 128, 128];
    pub const METAL: u8 = 255;
    pub const MATTE: u8 = 0;
    pub const ROUGHNESS: u8 = 96;
    pub const OCCLUSION: u8 = 255;
    pub const LOGO: [((f64, f64), [u8; 3]); 6] = [
        ((255.5, 102.0), [201, 118, 130]),
        ((110.0, 183.0), [238, 227, 145]),
        ((400.5, 183.0), [117, 194, 117]),
        ((111.0, 325.0), [118, 126, 189]),
        ((400.0, 325.0), [202, 148, 194]),
        ((255.5, 410.0), [217, 160, 125]),
    ];
    pub const LOGO_RADIUS: f64 = 100.0;
    pub const LOGO_HALO: f64 = 125.0;
    pub const BC1: u32 = 0x1a;
    pub const BC4: u32 = 0x1d;

    fn bad(m: String) -> E {
        E::Msg(format!("ap logo: {m}"))
    }

    pub fn new_model() -> String {
        format!("Model/{PROJECT}.{FMDB}.bfres.zs")
    }

    pub fn tex_file(sfx: &str) -> String {
        format!("Tex/{NEW_TEX}_{sfx}.bntx.zs")
    }

    pub fn source_of(path: &str) -> Option<String> {
        if path == new_model() {
            return Some(MODEL.to_string());
        }
        let sfx: &str = SUFFIXES.iter().copied().find(|x| tex_file(x) == path)?;
        Some(if sfx == "Alb" { ALB_SOURCE.to_string() } else { format!("Tex/{OLD_TEX}_{sfx}.bntx.zs") })
    }

    pub fn logo_colour(x: f64, y: f64) -> Option<[u8; 3]> {
        let mut c: Option<[u8; 3]> = None;
        for ((cx, cy), fill) in LOGO {
            let d: f64 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d <= LOGO_RADIUS * LOGO_RADIUS {
                c = Some(fill);
            } else if d <= LOGO_HALO * LOGO_HALO {
                c = None;
            }
        }
        c
    }

    pub fn face_colour(dx: f64, dy: f64) -> Option<[u8; 3]> {
        let d: f64 = (dx * dx + dy * dy).powf(0.5);
        if d > FACE_RADIUS {
            return None;
        }
        let inner: f64 = FACE_RADIUS * RIM;
        if d > inner {
            return Some(GOLD);
        }
        if d > inner - 3.0 {
            return Some(GOLD_DARK);
        }
        let scale: f64 = 512.0 / (2.0 * inner * 0.94);
        Some(logo_colour(256.0 + dx * scale, 256.0 + dy * scale).unwrap_or(CREAM))
    }

    pub fn sheet_colour(x: f64, y: f64) -> [u8; 3] {
        FACES.iter().find_map(|(cx, cy)| face_colour(x - cx, y - cy)).unwrap_or(GOLD)
    }

    pub fn on_logo(x: f64, y: f64) -> bool {
        let inner: f64 = FACE_RADIUS * RIM;
        FACES.iter().any(|(cx, cy)| (x - cx) * (x - cx) + (y - cy) * (y - cy) <= inner * inner)
    }

    pub fn void_extent(c: [u8; 4]) -> Vec<u8> {
        let mut out: Vec<u8> = vec![0xfc, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        for v in c {
            out.extend((v as u16 * 257).to_le_bytes());
        }
        out
    }

    pub fn bc4_constant(v: u8) -> Vec<u8> {
        vec![v, v, 0, 0, 0, 0, 0, 0]
    }

    fn rgb565(c: [u8; 3]) -> u16 {
        let [r, g, b]: [u32; 3] = c.map(|x| x as u32);
        (((r * 31 + 127) / 255) << 11 | ((g * 63 + 127) / 255) << 5 | ((b * 31 + 127) / 255)) as u16
    }

    fn expand565(v: u16) -> [i64; 3] {
        let (r, g, b): (i64, i64, i64) = ((v >> 11) as i64, ((v >> 5) & 63) as i64, (v & 31) as i64);
        [(r * 255 + 15) / 31, (g * 255 + 31) / 63, (b * 255 + 15) / 31]
    }

    fn dist(a: [i64; 3], b: [i64; 3]) -> i64 {
        (0..3).map(|k| (a[k] - b[k]) * (a[k] - b[k])).sum()
    }

    fn wide(c: [u8; 3]) -> [i64; 3] {
        c.map(|x| x as i64)
    }

    pub fn bc1_block(texels: &[[u8; 3]; 16]) -> Vec<u8> {
        let mut distinct: Vec<[u8; 3]> = texels.to_vec();
        distinct.sort();
        distinct.dedup();
        let (a, b): ([u8; 3], [u8; 3]) = if distinct.len() == 1 {
            (distinct[0], distinct[0])
        } else {
            let mut best: Option<(i64, [u8; 3], [u8; 3])> = None;
            for (i, p) in distinct.iter().enumerate() {
                for q in &distinct[i + 1..] {
                    let d: i64 = dist(wide(*p), wide(*q));
                    if best.is_none_or(|x| d > x.0) {
                        best = Some((d, *p, *q));
                    }
                }
            }
            let (_, p, q) = best.unwrap_or((0, distinct[0], distinct[0]));
            (p, q)
        };
        let (mut c0, mut c1): (u16, u16) = (rgb565(a), rgb565(b));
        if c0 < c1 {
            std::mem::swap(&mut c0, &mut c1);
        }
        let mut out: Vec<u8> = Vec::with_capacity(8);
        out.extend(c0.to_le_bytes());
        out.extend(c1.to_le_bytes());
        if c0 == c1 {
            out.extend(0u32.to_le_bytes());
            return out;
        }
        let (e0, e1): ([i64; 3], [i64; 3]) = (expand565(c0), expand565(c1));
        let pal: [[i64; 3]; 4] = [e0, e1, [0, 1, 2].map(|k| (2 * e0[k] + e1[k]) / 3), [0, 1, 2].map(|k| (e0[k] + 2 * e1[k]) / 3)];
        let mut idx: u32 = 0;
        for (n, t) in texels.iter().enumerate() {
            let mut pick: usize = 0;
            for i in 1..4 {
                if dist(pal[i], wide(*t)) < dist(pal[pick], wide(*t)) {
                    pick = i;
                }
            }
            idx |= (pick as u32) << (2 * n);
        }
        out.extend(idx.to_le_bytes());
        out
    }

    pub fn albedo_block(x0: f64, y0: f64, w: f64, scale: usize) -> Vec<u8> {
        let step: f64 = w / 4.0;
        let n: usize = if scale == 1 { 2 } else { 1 };
        let mut t: [[u8; 3]; 16] = [[0; 3]; 16];
        for j in 0..4 {
            for i in 0..4 {
                let mut sum: [u32; 3] = [0; 3];
                for a in 0..n {
                    for b in 0..n {
                        let c: [u8; 3] = sheet_colour(x0 + (i as f64 + (a as f64 + 0.5) / n as f64) * step, y0 + (j as f64 + (b as f64 + 0.5) / n as f64) * step);
                        for k in 0..3 {
                            sum[k] += c[k] as u32;
                        }
                    }
                }
                t[j * 4 + i] = sum.map(|v| (v as f64 / (n * n) as f64).round_ties_even() as u8);
            }
        }
        bc1_block(&t)
    }

    pub fn metal_block(x0: f64, y0: f64, w: f64, h: f64) -> Vec<u8> {
        bc4_constant(if on_logo(x0 + w / 2.0, y0 + h / 2.0) { MATTE } else { METAL })
    }

    fn u16_at(d: &[u8], o: usize) -> Res<u16> {
        d.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or_else(|| bad(format!("a field at {o:#x} leaves the file")))
    }

    fn u32_at(d: &[u8], o: usize) -> Res<u32> {
        d.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or_else(|| bad(format!("a field at {o:#x} leaves the file")))
    }

    fn ptr_at(d: &[u8], o: usize) -> Res<usize> {
        let s: &[u8] = d.get(o..o + 8).ok_or_else(|| bad(format!("a pointer at {o:#x} leaves the file")))?;
        let v: u64 = u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]);
        usize::try_from(v).ok().filter(|p| *p <= d.len()).ok_or_else(|| bad(format!("the pointer at {o:#x} leaves the file")))
    }

    fn find(d: &[u8], what: &[u8], from: usize) -> Option<usize> {
        d.get(from..)?.windows(what.len()).position(|w| w == what).map(|i| i + from)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Info {
        pub fmt: u32,
        pub w: usize,
        pub h: usize,
        pub gob: usize,
        pub start: usize,
        pub size: usize,
        pub levels: Vec<usize>,
    }

    pub fn bntx_info(d: &[u8]) -> Res<Info> {
        if d.get(..4) != Some(b"BNTX".as_slice()) {
            return Err(bad("not a BNTX texture".to_string()));
        }
        let at: usize = find(d, b"BRTI", 0).ok_or_else(|| bad("no BRTI texture".to_string()))?;
        if find(d, b"BRTI", at + 4).is_some() {
            return Err(bad("more than one texture in the BNTX".to_string()));
        }
        if u16_at(d, at + 0x12)? != 0 {
            return Err(bad("linear BNTX layout".to_string()));
        }
        let mips: usize = u16_at(d, at + 0x16)? as usize;
        let arr: usize = ptr_at(d, at + 0x70)?;
        let levels: Vec<usize> = (0..mips).map(|l| ptr_at(d, arr + 8 * l)).collect::<Res<Vec<usize>>>()?;
        let start: usize = *levels.first().ok_or_else(|| bad("a texture without mip levels".to_string()))?;
        Ok(Info {
            fmt: u32_at(d, at + 0x1c)? >> 8,
            w: u32_at(d, at + 0x24)? as usize,
            h: u32_at(d, at + 0x28)? as usize,
            gob: 1 << (u32_at(d, at + 0x34)? & 7),
            start,
            size: u32_at(d, at + 0x50)? as usize,
            levels,
        })
    }

    fn footprint(fmt: u32) -> Res<(usize, usize, usize)> {
        match crate::tex::surface::kind(fmt << 8) {
            Some(crate::tex::surface::Kind::Astc(w, h)) => Ok((w, h, 16)),
            _ if fmt == BC1 || fmt == BC4 => Ok((4, 4, 8)),
            _ => Err(bad(format!("texture format {fmt:#x} is not handled"))),
        }
    }

    pub fn block_address(x: usize, y: usize, width_gobs: usize, bpe: usize) -> usize {
        let xb: usize = x * bpe;
        (y / 8) * 512 * width_gobs + (xb / 64) * 512 + (xb & 15) + (y & 1) * 16 + ((y >> 1) & 1) * 32 + ((xb >> 4) & 1) * 64 + ((y >> 2) & 1) * 128 + ((xb >> 5) & 1) * 256
    }

    pub fn paint(d: &[u8], block: &dyn Fn(f64, f64, f64, f64, usize) -> Vec<u8>) -> Res<(Vec<u8>, usize)> {
        let i: Info = bntx_info(d)?;
        let (bw, bh, bpe): (usize, usize, usize) = footprint(i.fmt)?;
        if i.gob != 1 {
            return Err(bad("texture layout with a GOB height above 1".to_string()));
        }
        let mut out: Vec<u8> = d.to_vec();
        let end: usize = i.start + i.size;
        let mut n: usize = 0;
        for (l, start) in i.levels.iter().enumerate() {
            let (w, h): (usize, usize) = ((i.w >> l).max(1), (i.h >> l).max(1));
            let (wb, hb): (usize, usize) = (w.div_ceil(bw), h.div_ceil(bh));
            let wg: usize = (wb * bpe).div_ceil(64);
            let sc: usize = 1 << l;
            for y in 0..hb {
                for x in 0..wb {
                    let b: Vec<u8> = block((x * bw * sc) as f64, (y * bh * sc) as f64, (bw * sc) as f64, (bh * sc) as f64, sc);
                    if b.len() != bpe {
                        return Err(bad("an encoded block of the wrong size".to_string()));
                    }
                    let a: usize = start + block_address(x, y, wg, bpe);
                    if a + bpe > end || a + bpe > out.len() {
                        return Err(bad("a texture block outside the image data".to_string()));
                    }
                    out[a..a + bpe].copy_from_slice(&b);
                    n += 1;
                }
            }
        }
        Ok((out, n))
    }

    pub fn fill(d: &[u8], b: &[u8]) -> Res<Vec<u8>> {
        let i: Info = bntx_info(d)?;
        let (_, _, bpe): (usize, usize, usize) = footprint(i.fmt)?;
        if b.len() != bpe || i.start % bpe != 0 || i.size % bpe != 0 || i.start + i.size > d.len() {
            return Err(bad("a constant block that does not match the texture".to_string()));
        }
        let mut out: Vec<u8> = d.to_vec();
        for c in out[i.start..i.start + i.size].chunks_mut(bpe) {
            c.copy_from_slice(b);
        }
        Ok(out)
    }

    pub fn rename(d: &[u8], old: &str, new: &str) -> Res<Vec<u8>> {
        let (o, n): (&[u8], &[u8]) = (old.as_bytes(), new.as_bytes());
        if o.len() != n.len() || o.is_empty() {
            return Err(bad(format!("{old} and {new} differ in length")));
        }
        let mut out: Vec<u8> = d.to_vec();
        let mut at: usize = 0;
        let mut hits: usize = 0;
        while let Some(i) = find(&out, o, at) {
            out[i..i + o.len()].copy_from_slice(n);
            at = i + o.len();
            hits += 1;
        }
        if hits == 0 {
            return Err(bad(format!("{old} is not in the file")));
        }
        Ok(out)
    }

    pub fn count(d: &[u8], what: &str) -> usize {
        let w: &[u8] = what.as_bytes();
        let mut n: usize = 0;
        let mut at: usize = 0;
        while let Some(i) = find(d, w, at) {
            n += 1;
            at = i + w.len();
        }
        n
    }

    pub struct Shape {
        pub positions: Vec<usize>,
        pub meshes: Vec<Vec<usize>>,
    }

    fn name_at(d: &[u8], o: usize) -> Res<String> {
        let n: usize = u16_at(d, o)? as usize;
        let b: &[u8] = d.get(o + 2..o + 2 + n).ok_or_else(|| bad("a name leaves the file".to_string()))?;
        Ok(String::from_utf8_lossy(b).into_owned())
    }

    pub fn shapes(d: &[u8]) -> Res<Vec<(String, Shape)>> {
        if d.get(..8) != Some(b"FRES    ".as_slice()) {
            return Err(bad("not a BFRES model".to_string()));
        }
        let rlt: usize = u32_at(d, 0x18)? as usize;
        if d.get(rlt..rlt + 4) != Some(b"_RLT".as_slice()) {
            return Err(bad("no BFRES relocation table".to_string()));
        }
        let buffers: usize = u32_at(d, rlt + 48)? as usize;
        let mut out: Vec<(String, Shape)> = Vec::new();
        let mut at: Option<usize> = find(d, b"FSHP", 0);
        while let Some(s) = at.filter(|s| *s < rlt) {
            let name: String = name_at(d, ptr_at(d, s + 8)?)?;
            let v: usize = ptr_at(d, s + 0x10)?;
            if d.get(v..v + 4) != Some(b"FVTX".as_slice()) || d.get(v + 0x4d) != Some(&1) {
                return Err(bad(format!("unexpected vertex buffer for {name}")));
            }
            let attrs: usize = ptr_at(d, v + 8)?;
            let mut pos: Option<(u16, usize)> = None;
            for k in 0..d[v + 0x4c] as usize {
                let e: usize = attrs + k * 16;
                if name_at(d, ptr_at(d, e)?)? == "_p0" {
                    pos = Some((u16_at(d, e + 8)?, u16_at(d, e + 12)? as usize));
                }
            }
            let (fmt, off): (u16, usize) = pos.ok_or_else(|| bad(format!("no position attribute for {name}")))?;
            if fmt != 0x1505 {
                return Err(bad(format!("unexpected position format {fmt:#x} for {name}")));
            }
            let stride: usize = u32_at(d, ptr_at(d, v + 0x38)?)? as usize;
            let first: usize = buffers + u32_at(d, v + 0x48)? as usize;
            let count: usize = u32_at(d, v + 0x50)? as usize;
            let arr: usize = ptr_at(d, s + 0x18)?;
            let lods: usize = *d.get(s + 0x5b).ok_or_else(|| bad(format!("{name} ends early")))? as usize;
            let mut meshes: Vec<Vec<usize>> = Vec::new();
            for l in 0..lods {
                let m: usize = arr + l * 0x38;
                let (start, prim, kind, total, base): (usize, u32, u32, usize, usize) = (u32_at(d, m + 0x20)? as usize, u32_at(d, m + 0x24)?, u32_at(d, m + 0x28)?, u32_at(d, m + 0x2c)? as usize, u32_at(d, m + 0x30)? as usize);
                if prim != 3 || kind != 1 {
                    return Err(bad(format!("unexpected index buffer for {name}")));
                }
                let ix: Vec<usize> = (0..total).map(|i| u16_at(d, buffers + start + 2 * i).map(|x| x as usize + base)).collect::<Res<Vec<usize>>>()?;
                if ix.iter().any(|i| *i >= count) {
                    return Err(bad(format!("an index out of range for {name}")));
                }
                meshes.push(ix);
            }
            let positions: Vec<usize> = (0..count).map(|i| first + i * stride + off).collect();
            if positions.last().is_some_and(|p| p + 6 > d.len()) {
                return Err(bad(format!("the vertices of {name} leave the file")));
            }
            out.push((name, Shape { positions, meshes }));
            at = find(d, b"FSHP", s + 4);
        }
        Ok(out)
    }

    fn shape<'a>(all: &'a [(String, Shape)], name: &str) -> Res<&'a Shape> {
        all.iter().find(|x| x.0 == name).map(|x| &x.1).ok_or_else(|| bad(format!("the model has no shape {name}")))
    }

    pub fn hide(d: &[u8], names: &[&str]) -> Res<Vec<u8>> {
        let all: Vec<(String, Shape)> = shapes(d)?;
        let mut out: Vec<u8> = d.to_vec();
        for n in names {
            for p in &shape(&all, n)?.positions {
                out[*p..*p + 6].fill(0);
            }
        }
        Ok(out)
    }

    pub fn half(b: u16) -> f64 {
        let sign: f64 = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
        let e: i32 = ((b >> 10) & 0x1f) as i32;
        let m: f64 = (b & 0x3ff) as f64;
        match e {
            0 => sign * m * 2f64.powi(-24),
            31 => if m == 0.0 { sign * f64::INFINITY } else { f64::NAN },
            _ => sign * (1.0 + m / 1024.0) * 2f64.powi(e - 15),
        }
    }

    fn root(p: &mut [usize], mut v: usize) -> usize {
        while p[v] != v {
            p[v] = p[p[v]];
            v = p[v];
        }
        v
    }

    fn join(p: &mut [usize], a: usize, b: usize) {
        let (a, b): (usize, usize) = (root(p, a), root(p, b));
        if a != b {
            p[a] = b;
        }
    }

    pub fn islands(d: &[u8], name: &str, face: f64, radius: f64) -> Res<Vec<Vec<usize>>> {
        let all: Vec<(String, Shape)> = shapes(d)?;
        let sh: &Shape = shape(&all, name)?;
        let raw: Vec<[u16; 3]> = sh.positions.iter().map(|a| Ok([u16_at(d, *a)?, u16_at(d, a + 2)?, u16_at(d, a + 4)?])).collect::<Res<Vec<[u16; 3]>>>()?;
        let pts: Vec<[f64; 3]> = raw.iter().map(|r| r.map(half)).collect();
        let mut p: Vec<usize> = (0..pts.len()).collect();
        for ix in &sh.meshes {
            for t in ix.chunks_exact(3) {
                join(&mut p, t[0], t[1]);
                join(&mut p, t[0], t[2]);
            }
        }
        let mut seen: std::collections::HashMap<[u16; 3], usize> = std::collections::HashMap::new();
        for (v, r) in raw.iter().enumerate() {
            let k: [u16; 3] = r.map(|x| if x == 0x8000 { 0 } else { x });
            let first: usize = *seen.entry(k).or_insert(v);
            join(&mut p, v, first);
        }
        let mut pieces: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        for v in 0..pts.len() {
            let r: usize = root(&mut p, v);
            pieces.entry(r).or_default().push(v);
        }
        Ok(pieces.into_values().filter(|pc| pc.len() > 3 && pc.iter().all(|v| pts[*v][2] >= face && pts[*v][0] * pts[*v][0] + pts[*v][1] * pts[*v][1] < radius * radius)).collect())
    }

    pub fn remove_relief(d: &[u8]) -> Res<Vec<u8>> {
        let (name, face, radius) = RELIEF;
        let is: Vec<Vec<usize>> = islands(d, name, face, radius)?;
        if is.is_empty() {
            return Err(bad(format!("no relief found on {name}")));
        }
        let all: Vec<(String, Shape)> = shapes(d)?;
        let pos: &[usize] = &shape(&all, name)?.positions;
        let mut out: Vec<u8> = d.to_vec();
        for pc in &is {
            for v in pc {
                out[pos[*v]..pos[*v] + 6].fill(0);
            }
        }
        Ok(out)
    }

    pub fn texture(sfx: &str, raw: &[u8]) -> Res<Vec<u8>> {
        let i: Info = bntx_info(raw)?;
        let (out, n): (Vec<u8>, usize) = match sfx {
            "Alb" => {
                if (i.fmt, i.w, i.h) != (BC1, SHEET, SHEET) {
                    return Err(bad("unexpected albedo container".to_string()));
                }
                paint(raw, &|x, y, w, _, sc| albedo_block(x, y, w, sc))?
            }
            "Mtl" => {
                if (i.fmt, i.w) != (BC4, SHEET) {
                    return Err(bad("unexpected metalness texture".to_string()));
                }
                paint(raw, &|x, y, w, h, _| metal_block(x, y, w, h))?
            }
            "AO" => (fill(raw, &bc4_constant(OCCLUSION))?, 1),
            "Rgh" => (fill(raw, &bc4_constant(ROUGHNESS))?, 1),
            "Emc" => (fill(raw, &void_extent([0, 0, 0, 255]))?, 1),
            "Nrm" => (fill(raw, &void_extent(FLAT_NORMAL))?, 1),
            _ => return Err(bad(format!("no texture rule for {sfx}"))),
        };
        if n == 0 || out == raw {
            return Err(bad(format!("the {sfx} texture was left unchanged")));
        }
        Ok(out)
    }

    pub fn model(raw: &[u8]) -> Res<Vec<u8>> {
        let mut m: Vec<u8> = remove_relief(&hide(raw, &HIDDEN)?)?;
        for sfx in SUFFIXES {
            let n: String = format!("{OLD_TEX}_{sfx}");
            if count(&m, &n) != 1 {
                return Err(bad(format!("the medallion model names {n} more than once or not at all")));
            }
            m = rename(&m, &n, &format!("{NEW_TEX}_{sfx}"))?;
        }
        Ok(m)
    }

    pub fn files(read: &mut dyn FnMut(&str) -> Res<Vec<u8>>) -> Res<Vec<(String, Vec<u8>, String)>> {
        let mut out: Vec<(String, Vec<u8>, String)> = Vec::new();
        for sfx in SUFFIXES {
            let p: String = tex_file(sfx);
            let src: String = source_of(&p).ok_or_else(|| bad(format!("no source for {p}")))?;
            let old: &str = src.trim_start_matches("Tex/").trim_end_matches(".bntx.zs");
            let raw: Vec<u8> = rename(&read(&src)?, old, &format!("{NEW_TEX}_{sfx}"))?;
            out.push((p, texture(sfx, &raw)?, src));
        }
        out.push((new_model(), model(&read(MODEL)?)?, MODEL.to_string()));
        Ok(out)
    }
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
        let mut look: Option<Look> = None;
        let mut hook = |rf: &mut Romfs, work: &P, outs: &mut Vec<Out>| -> Res<()> {
            let n: usize = outs.len();
            look = Some(stage_look(rf, work, outs, &f, Look::Logo)?);
            staged = outs.len() - n;
            Ok(())
        };
        let b = crate::slice::edit::build_data(d.clone(), &seed, &r, &load, "ootrr-ap-item", crate::modpack::Host::Eden, &extra, Some(&mut hook)).unwrap();
        assert_eq!(look, Some(Look::Logo));
        assert!(staged >= 2 + 7 + 3 + 16, "{staged}");
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
            let mi: Byml = Byml::parse(&sarc::get(&es, &model_info_file()).unwrap().data).unwrap();
            assert_eq!(mi.root.get("ModelProjectName").and_then(|v| v.as_str()), Some(logo::PROJECT));
            assert!(sarc::get(&es, SPIN_INFO).is_some());
            for (i, t) in ts.iter().enumerate() {
                let row: &Node = rows::find(t, &a).unwrap_or_else(|| panic!("{a} has no row in {}", TABLES[i]));
                let mut same: Node = row.clone();
                same.set(rows::ROW_ID, s(TEMPLATE));
                if i == 0 {
                    assert_eq!(row.get(rows::ACTOR_NAME).and_then(|v| v.as_str()), Some(a.as_str()));
                    assert_eq!(row.get("ModelProjectName").and_then(|v| v.as_str()), Some(logo::PROJECT));
                    assert_eq!(row.get("FmdbName").and_then(|v| v.as_str()), Some(logo::FMDB));
                    assert!(row.get("AnimationResources").is_some());
                    continue;
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
        let van: crate::rsizetable::Rst = crate::rsizetable::load(&mut rf).unwrap();
        for (p, _, src) in logo::files(&mut |p: &str| rf.read_zs(p)).unwrap() {
            let raw: Vec<u8> = mod_file(&mut rf, &romfs, &p);
            assert_eq!(raw.len(), rf.read_zs(&src).unwrap().len(), "{p}");
            assert_eq!(rst.get(&p), van.get(&src), "{p}");
        }
        let a0: String = actor(&f[0].label);
        let line =b.chests.iter().find(|c| c.name == chest).unwrap_or_else(|| panic!("{chest} is not among the edited chests"));
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

    const MOZ: [(&str, &str); 7] = [
        ("Tex/Mt_Item_Medal_Arch_Alb.bntx.zs", "381705bd1bee9566a3492e8331ef2c2cdf2b78ca91d2d7f99af906e03d01074a"),
        ("Tex/Mt_Item_Medal_Arch_AO.bntx.zs", "d4f63a0e944ceab72b7e15b7b848f68d8b32ae54089c727e34318ed39b525856"),
        ("Tex/Mt_Item_Medal_Arch_Emc.bntx.zs", "99acfbd853d89f402c3d0d93bcb5964a0ff031603049b2377bd1e5b5df089758"),
        ("Tex/Mt_Item_Medal_Arch_Mtl.bntx.zs", "0da437b52cdb717fa09e2104355f217ab62236997b36c40e2864c5c65f92711e"),
        ("Tex/Mt_Item_Medal_Arch_Nrm.bntx.zs", "ebc4174598c37f425341e59d9ccfcb5fa58632dc4b01b054d5bb4e728fc93572"),
        ("Tex/Mt_Item_Medal_Arch_Rgh.bntx.zs", "e67ce475d484d27150bd7982bc7192f133619798218f1a58b0cf278a87f22afc"),
        ("Model/Item_Medal_Arch.Item_Medal_Fire.bfres.zs", "c50edefed0a0f1e5ad00f4240b4227eb58b1f98eb821c71787788cf9b032f17d"),
    ];

    fn bit(k: &[u8], n: u32) -> u8 {
        let i: usize = (n >> 3) as usize;
        if i >= k.len() { 0 } else { (k[k.len() - 1 - i] >> (n & 7)) & 1 }
    }

    fn dic_ok(d: &[u8]) -> (usize, usize) {
        let u32_at = |o: usize| u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]);
        let u16_at = |o: usize| u16::from_le_bytes([d[o], d[o + 1]]) as usize;
        let (mut dics, mut bad): (usize, usize) = (0, 0);
        for at in (0..d.len() - 4).filter(|i| &d[*i..*i + 4] == b"_DIC") {
            let n: usize = u32_at(at + 4) as usize;
            let node = |k: usize| {
                let o: usize = at + 8 + 16 * k;
                let kp: usize = u64::from_le_bytes(d[o + 8..o + 16].try_into().unwrap()) as usize;
                (u32_at(o) as i32, u16_at(o + 4), u16_at(o + 6), d[kp + 2..kp + 2 + u16_at(kp)].to_vec())
            };
            dics += 1;
            for i in 1..=n {
                let key: Vec<u8> = node(i).3;
                let mut prev = node(0);
                let mut ci: usize = prev.1;
                let mut child = node(ci);
                while prev.0 < child.0 {
                    ci = if bit(&key, child.0 as u32) == 1 { child.2 } else { child.1 };
                    prev = child;
                    child = node(ci);
                }
                if ci != i || child.3 != key {
                    bad += 1;
                }
            }
        }
        (dics, bad)
    }

    #[test]
    fn the_logo_files_match_moz_and_read_back() {
        let Some(r) = root() else { return };
        let mut rf: Romfs = Romfs::open(&r).unwrap();
        let made: Vec<(String, Vec<u8>, String)> = logo::files(&mut |p: &str| rf.read_zs(p)).unwrap();
        assert_eq!(made.len(), MOZ.len());
        for (p, raw, src) in &made {
            let want: &str = MOZ.iter().find(|m| m.0 == p).unwrap().1;
            assert_eq!(crate::settings::digest::sha256_hex(raw), want, "{p} differs from the Python original");
            assert_eq!(logo::source_of(p).as_deref(), Some(src.as_str()));
            assert_eq!(raw.len(), rf.read_zs(src).unwrap().len());
            if p.ends_with(".bntx.zs") {
                let (n, bad) = dic_ok(raw);
                assert_eq!((n, bad), (1, 0), "{p}");
                let ts = crate::tex::bntx::parse(raw).unwrap();
                assert_eq!(ts.len(), 1);
                assert!(ts[0].name.starts_with(logo::NEW_TEX), "{}", ts[0].name);
            }
        }
        let alb: &[u8] = &made[0].1;
        let t = &crate::tex::bntx::parse(alb).unwrap()[0];
        let im = crate::tex::surface::decode(alb, t).unwrap();
        let near = |x: usize, y: usize, c: [u8; 3]| {
            let p: [u8; 4] = im.at(x, y);
            assert!((0..3).all(|k| (p[k] as i32 - c[k] as i32).abs() <= 12), "{x} {y} {p:?} {c:?}");
        };
        near(153, 152, logo::CREAM);
        near(377, 375, logo::CREAM);
        near(153 + 125, 152, logo::GOLD);
        near(500, 40, logo::GOLD);
        near(153, 87, logo::LOGO[0].1);
        let mut off: u64 = 0;
        for y in (0..logo::SHEET).step_by(7) {
            for x in (0..logo::SHEET).step_by(7) {
                let w: [u8; 3] = logo::sheet_colour(x as f64 + 0.5, y as f64 + 0.5);
                let p: [u8; 4] = im.at(x, y);
                off += (0..3).map(|k| (p[k] as i64 - w[k] as i64).unsigned_abs()).sum::<u64>();
            }
        }
        let n: u64 = (logo::SHEET.div_ceil(7) * logo::SHEET.div_ceil(7) * 3) as u64;
        assert!(off / n <= 6, "mean channel error {}", off / n);
        let m: &[u8] = &made[6].1;
        let orig: Vec<u8> = rf.read_zs(logo::MODEL).unwrap();
        assert_eq!(logo::islands(&orig, logo::RELIEF.0, logo::RELIEF.1, logo::RELIEF.2).unwrap().iter().map(|p| p.len()).collect::<Vec<usize>>(), vec![693]);
        assert!(logo::islands(m, logo::RELIEF.0, logo::RELIEF.1, logo::RELIEF.2).unwrap().is_empty());
        let shapes = logo::shapes(m).unwrap();
        assert_eq!(shapes.len(), logo::shapes(&orig).unwrap().len());
        for (name, sh) in &shapes {
            let zero: bool = sh.positions.iter().all(|p| m[*p..*p + 6].iter().all(|b| *b == 0));
            assert_eq!(zero, logo::HIDDEN.contains(&name.as_str()), "{name}");
        }
        for sfx in logo::SUFFIXES {
            assert_eq!(logo::count(m, &format!("{}_{sfx}", logo::NEW_TEX)), 1);
            assert_eq!(logo::count(m, &format!("{}_{sfx}", logo::OLD_TEX)), 0);
        }
    }

    #[test]
    fn block_addresses_follow_the_shared_swizzle() {
        for (wb, bpe) in [(144usize, 8usize), (36, 8), (64, 16), (1, 8)] {
            let wg: usize = (wb * bpe).div_ceil(64);
            for y in 0..24 {
                for x in 0..wb {
                    let want: usize = if bpe == 8 { crate::tex::swizzle::addr(x / 2, y, wb.div_ceil(2), 1) + (x % 2) * 8 } else { crate::tex::swizzle::addr(x, y, wb, 1) };
                    assert_eq!(logo::block_address(x, y, wg, bpe), want, "{wb} {bpe} {x} {y}");
                }
            }
        }
        assert_eq!(logo::void_extent([1, 2, 3, 4]), vec![0xfc, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1, 1, 2, 2, 3, 3, 4, 4]);
        assert_eq!(logo::bc1_block(&[[10, 20, 30]; 16])[4..], [0, 0, 0, 0]);
        assert_eq!(logo::half(0x3c00), 1.0);
        assert_eq!(logo::half(0xc000), -2.0);
        assert_eq!(logo::half(0x0001), 2f64.powi(-24));
    }

    #[test]
    fn a_broken_logo_falls_back_to_the_plain_medallion() {
        let Some(r) = root() else { return };
        let mut rf: Romfs = Romfs::open(&r).unwrap();
        let f: Vec<Foreign> = vec![fx("AP: Bow (Ami)", "Bow", "Ami")];
        let a: String = actor(&f[0].label);
        let none: PB = PB::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ap_item_no_work");
        let mut outs: Vec<Out> = Vec::new();
        assert_eq!(stage_look(&mut rf, &none, &mut outs, &f, Look::Plain).unwrap(), Look::Plain);
        assert!(!outs.iter().any(|o| o.path.starts_with("Model/") || o.path.starts_with("Tex/")));
        let es: Vec<Entry> = sarc::parse(&rf.zs.decompress(&outs.iter().find(|o| o.path == pack_of(&a)).unwrap().z).unwrap()).unwrap();
        let mut want: Vec<String> = sarc::parse(&rf.read_zs(&pack_of(TEMPLATE)).unwrap()).unwrap().into_iter().map(|e| if e.name == actor_file(TEMPLATE) { actor_file(&a) } else { e.name }).collect();
        let mut got: Vec<String> = es.iter().map(|e| e.name.clone()).collect();
        want.sort();
        got.sort();
        assert_eq!(got, want);
        let broken: PB = PB::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ap_item_broken_work");
        let _ = std::fs::remove_dir_all(&broken);
        let spin: PB = pack_of(SPIN_PACK).split('/').fold(broken.clone(), |a, s| a.join(s));
        std::fs::create_dir_all(spin.parent().unwrap()).unwrap();
        std::fs::write(&spin, b"not a pack").unwrap();
        let mut outs: Vec<Out> = Vec::new();
        assert_eq!(stage_look(&mut rf, &broken, &mut outs, &f, Look::Logo).unwrap(), Look::Plain);
        let t: Byml = Byml::parse(&rf.zs.decompress(&outs.iter().find(|o| o.path == ACTOR_INFO).unwrap().z).unwrap()).unwrap();
        assert_eq!(rows::find(&t, &a).and_then(|r| r.get("ModelProjectName")).and_then(|v| v.as_str()), Some(TEMPLATE));
        let mut outs: Vec<Out> = Vec::new();
        assert_eq!(stage_look(&mut rf, &none, &mut outs, &f, Look::Logo).unwrap(), Look::Logo);
        assert_eq!(outs.iter().filter(|o| o.path.starts_with("Model/") || o.path.starts_with("Tex/")).count(), 7);
        let _ = std::fs::remove_dir_all(&broken);
    }
}
