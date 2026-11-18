pub mod drops;
pub mod dupes;
pub mod extras;
pub mod flows;
pub mod lottery;
pub mod minibosses;
pub mod minions;
pub mod regen;
pub mod respawn;
pub mod stats;
pub mod target;

use std::fs;

use crate::error::Res;
use crate::qol::{edit_plain, edit_zs, Have};
use crate::romfs::Romfs;
use crate::sarc::{self, Entry};
use crate::settings::Settings;
use crate::staging::Out;
use crate::zs;

pub const HEALTH: &str = "enemy_health";
pub const DAMAGE: &str = "enemy_damage";
pub const SPEED: &str = "enemy_speed";
pub const NO_HEARTS: &str = "no_heart_drops";
pub const NO_FAIRIES: &str = "no_gossip_fairies";
pub const NO_AMMO: &str = "no_ammo_drops";
pub const LOSE_HEART: &str = "lose_heart_on_death";
pub const NO_BOTTLES: &str = "no_bottles_in_combat";
pub const NO_TARGET: &str = "no_z_targeting";
pub const NO_DUPES: &str = "no_dupe_items";
pub const MORE: &str = "more_enemies";
pub const RESPAWN: &str = "enemies_respawn_sooner";
pub const MINIBOSSES: &str = "boss_minibosses";
pub const REGEN: &str = "enemy_regen";
pub const MINIONS: &str = "boss_minions";

pub const HEALTH_NUM: u16 = 110;
pub const DAMAGE_NUM: u16 = 111;
pub const SPEED_NUM: u16 = 112;
pub const NO_HEARTS_NUM: u16 = 113;
pub const NO_FAIRIES_NUM: u16 = 114;
pub const NO_AMMO_NUM: u16 = 115;
pub const LOSE_HEART_NUM: u16 = 116;
pub const NO_BOTTLES_NUM: u16 = 117;
pub const NO_TARGET_NUM: u16 = 124;
pub const NO_DUPES_NUM: u16 = 122;
pub const MORE_NUM: u16 = 119;
pub const RESPAWN_NUM: u16 = 123;
pub const MINIBOSSES_NUM: u16 = 120;
pub const REGEN_NUM: u16 = 118;
pub const MINIONS_NUM: u16 = 121;

pub const NORMAL: i64 = 100;
pub const STAT_RANGE: (i64, i64) = (50, 500);
pub const SPEED_RANGE: (i64, i64) = (75, 200);
pub const STAT_STEP: i64 = 25;
pub const SPEED_STEP: i64 = 5;
pub const ACTOR_PACKS: &str = "Pack/Actor";
pub const MORE_OFF: &str = "off";
pub const MORE_LEVELS: &[(&str, &str, i64)] = &[(MORE_OFF, "Off", 0), ("half", "Half again", 50), ("double", "Double", 100)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opts {
    pub health: i64,
    pub damage: i64,
    pub speed: i64,
    pub no_hearts: bool,
    pub no_fairies: bool,
    pub no_ammo: bool,
    pub lose_heart: bool,
    pub no_bottles: bool,
    pub no_target: bool,
    pub no_dupes: bool,
    pub extra: i64,
    pub respawn: bool,
    pub minibosses: bool,
    pub regen: bool,
    pub minions: bool,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts { health: NORMAL, damage: NORMAL, speed: NORMAL, no_hearts: false, no_fairies: false, no_ammo: false, lose_heart: false, no_bottles: false, no_target: false, no_dupes: false, extra: 0, respawn: false, minibosses: false, regen: false, minions: false }
    }
}

fn times(pct: i64) -> String {
    let s: String = format!("{:.2}", pct as f64 / 100.0);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl Opts {
    pub fn cut(&self) -> drops::Cut {
        drops::Cut { hearts: self.no_hearts, ammo: self.no_ammo }
    }

    pub fn packs(&self) -> bool {
        self.health != NORMAL || self.damage != NORMAL || self.speed != NORMAL || self.cut().any() || self.no_target
    }

    pub fn any(&self) -> bool {
        self.packs() || self.no_fairies || self.lose_heart || self.no_bottles || self.no_dupes || self.extra > 0 || self.respawn || self.minibosses || self.regen || self.minions
    }

    pub fn notes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if self.health != NORMAL {
            out.push(format!("Enemies and bosses have {} times their usual health. Enemies that are beaten by a scripted hit, such as Deku Scrubs, are unchanged. Experimental, untested in the game.", times(self.health)));
        }
        if self.damage != NORMAL {
            out.push(format!("Enemy attacks and enemy projectiles deal {} times their usual damage. Grabs and traps that are not enemies are unchanged. Experimental, untested in the game.", times(self.damage)));
        }
        if self.speed != NORMAL {
            out.push(format!("Enemies move {} times as fast and wait that much less between attacks. Enemies that only move through their animations keep their pace. Experimental, untested in the game.", times(self.speed)));
        }
        if self.regen {
            out.push("Enemies and bosses that are still alive win back one point of health about every five seconds, up to their usual health. Experimental, untested in the game.".to_string());
        }
        if self.no_hearts {
            out.push("Enemies, pots, grass and other random drops never give recovery hearts, also when your health is low. Where the game would hand out a heart because you need one, a green rupee drops. Experimental, untested in the game.".to_string());
        }
        if self.no_ammo {
            out.push("Enemies, pots, grass and other random drops never give arrows, seeds, bombs, Deku Nuts or Deku Sticks. Experimental, untested in the game.".to_string());
        }
        if self.no_fairies {
            out.push("Playing a song to a gossip stone no longer calls a fairy that heals you. Experimental, untested in the game.".to_string());
        }
        if self.lose_heart {
            out.push("Each game over that you continue from takes away one heart container, down to three hearts, and the game is saved as it stood when you fell, so the loss and everything you had collected stay. A fairy in a bottle that revives you costs nothing. Experimental, untested in the game.".to_string());
        }
        if self.no_bottles {
            out.push("While the enemy battle music plays or during a boss battle, Link shakes his head and puts the bottle away without drinking or letting out its contents. Experimental, untested in the game.".to_string());
        }
        if self.no_target {
            out.push("Enemies and bosses cannot be locked on to. Aim assist, talking to people, reading signs and locking on to things that are not enemies are unchanged. Experimental, untested in the game.".to_string());
        }
        if self.no_dupes {
            out.push("A bottle is not filled with something that another of your bottles already holds. This covers catching things with a bottle, shop shelves, Deku Scrub sellers, cows, Talon and the potion shops. A refused catch may still use up what you swung at. Bottle contents from chests and other shuffled items are not covered. Experimental, untested in the game.".to_string());
        }
        if self.extra > 0 {
            out.push(format!("About {} in 100 of the ordinary enemies that no door, chest, counter or event is tied to have one extra copy standing next to them. Experimental, untested in the game.", self.extra));
        }
        if self.respawn {
            out.push("Most ordinary enemies come back when you leave their room or area and return, not only when you leave the whole dungeon or region. Bosses, minibosses, Gold Skulltulas, Poes, Deku Scrubs and a few enemies that share their data with a boss are unchanged. Experimental, untested in the game.".to_string());
        }
        if self.minibosses {
            out.push(format!(
                "A strong enemy waits in the boss room of these bosses and leaves when the boss is beaten: {}. The other bosses and the dream world fights are unchanged. Experimental, untested in the game.",
                minibosses::covered()
            ));
        }
        if self.minions {
            out.push(minions::NOTE.to_string());
        }
        out
    }
}

pub fn read(s: &Settings, bad: &mut Vec<String>) -> Opts {
    let mut num = |id: &str, r: (i64, i64)| -> i64 {
        match s.int(id) {
            Ok(v) if (r.0..=r.1).contains(&v) => v,
            Ok(v) => {
                bad.push(format!("{id} is {v}, which is outside {} to {}", r.0, r.1));
                NORMAL
            }
            Err(e) => {
                bad.push(e.to_string());
                NORMAL
            }
        }
    };
    let health: i64 = num(HEALTH, STAT_RANGE);
    let damage: i64 = num(DAMAGE, STAT_RANGE);
    let speed: i64 = num(SPEED, SPEED_RANGE);
    let mut flag = |id: &str| -> bool {
        s.flag(id).unwrap_or_else(|e| {
            bad.push(e.to_string());
            false
        })
    };
    let o: Opts = Opts { health, damage, speed, no_hearts: flag(NO_HEARTS), no_fairies: flag(NO_FAIRIES), no_ammo: flag(NO_AMMO), lose_heart: flag(LOSE_HEART), no_bottles: flag(NO_BOTTLES), no_target: flag(NO_TARGET), no_dupes: flag(NO_DUPES), extra: 0, respawn: flag(RESPAWN), minibosses: flag(MINIBOSSES), regen: flag(REGEN), minions: flag(MINIONS) };
    let extra: i64 = match s.text(MORE) {
        Ok(t) => MORE_LEVELS.iter().find(|l| l.0 == t).map(|l| l.2).unwrap_or_else(|| {
            bad.push(format!("{MORE} is {t}, which is not a known level"));
            0
        }),
        Err(e) => {
            bad.push(e.to_string());
            0
        }
    };
    Opts { extra, ..o }
}

fn edit_entry(e: &Entry, o: &Opts) -> Res<Option<Vec<u8>>> {
    let n: &str = &e.name;
    if o.minions && n == minions::DODONGO_PARAM {
        return minions::babies(&e.data);
    }
    if o.health != NORMAL
        && let Some(s) = stats::stem(n, stats::LIFE_DIR, stats::LIFE_EXT)
    {
        return if stats::is_enemy(s) { stats::life(&e.data, o.health) } else { Ok(None) };
    }
    if o.damage != NORMAL
        && let Some(s) = stats::stem(n, stats::ATTACK_DIR, stats::ATTACK_EXT)
    {
        return if stats::is_enemy_attack(s) { stats::attack(&e.data, o.damage) } else { Ok(None) };
    }
    if o.speed != NORMAL
        && let Some(s) = stats::stem(n, stats::MOVE_DIR, stats::MOVE_EXT)
    {
        return if stats::is_enemy(s) { stats::movement(&e.data, o.speed) } else { Ok(None) };
    }
    if o.speed != NORMAL
        && let Some(s) = stats::stem(n, stats::PACE_DIR, stats::PACE_EXT)
    {
        return if stats::is_enemy(s) { stats::pace(&e.data, o.speed) } else { Ok(None) };
    }
    if n.starts_with(drops::DIR) {
        return drops::edit(&e.data, o.cut());
    }
    if o.no_target
        && let Some(s) = stats::stem(n, target::DIR, target::EXT)
    {
        return if target::is_foe(s) { target::edit(&e.data) } else { Ok(None) };
    }
    Ok(None)
}

pub fn edit_pack(raw: &[u8], o: &Opts) -> Res<Option<Vec<u8>>> {
    let mut es: Vec<Entry> = sarc::parse(raw)?;
    let mut hit: bool = false;
    for e in es.iter_mut() {
        if let Some(d) = edit_entry(e, o)? {
            e.data = d;
            hit = true;
        }
    }
    if !hit {
        return Ok(None);
    }
    Ok(Some(sarc::write(&es)?))
}

fn pack_out(rf: &mut Romfs, have: Have, path: &str, o: &Opts) -> Res<Option<Out>> {
    let z: Vec<u8> = match have(path) {
        Some(b) => b,
        None => rf.read(path)?,
    };
    let id: Option<u32> = zs::dict_of(&z);
    let raw: Vec<u8> = rf.zs.decompress(&z)?;
    let Some(out) = edit_pack(&raw, o)? else {
        return Ok(None);
    };
    Ok(Some(Out { path: path.to_string(), old: raw.len(), new: out.len(), z: zs::compress_raw(&out, id)? }))
}

pub fn pack_files(rf: &Romfs, o: &Opts) -> Res<Vec<String>> {
    if !o.packs() {
        return Ok(if o.minions { vec![minions::DODONGO_PACK.to_string()] } else { Vec::new() });
    }
    let mut out: Vec<String> = rf.list(ACTOR_PACKS)?.into_iter().filter(|p| p.ends_with(".pack.zs")).collect();
    if o.cut().any() {
        out.push(drops::FOLIAGE.to_string());
    }
    out.sort();
    Ok(out)
}

fn fold(outs: &mut Vec<Out>, more: Vec<Out>) {
    for m in more {
        match outs.iter_mut().find(|x| x.path == m.path) {
            Some(x) => {
                x.z = m.z;
                x.new = m.new;
            }
            None => outs.push(m),
        }
    }
}

fn step(rf: &mut Romfs, outs: &mut Vec<Out>, have: Have, f: impl FnOnce(&mut Romfs, Have) -> Res<Vec<Out>>) -> Res<()> {
    let more: Vec<Out> = {
        let seen = |p: &str| -> Option<Vec<u8>> { outs.iter().find(|x| x.path == p).map(|x| x.z.clone()).or_else(|| have(p)) };
        f(rf, &seen)?
    };
    fold(outs, more);
    Ok(())
}

pub fn stage(rf: &mut Romfs, o: &Opts, have: Have, seed: &str) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = Vec::new();
    for f in pack_files(rf, o)? {
        if let Some(x) = pack_out(rf, have, &f, o)? {
            outs.push(x);
        }
    }
    if o.cut().any() {
        let raw: Vec<u8> = match have(lottery::FILE) {
            Some(b) => b,
            None => rf.read(lottery::FILE)?,
        };
        if let Some(out) = lottery::edit(&raw, o.cut())? {
            outs.push(Out { path: lottery::FILE.to_string(), old: raw.len(), new: out.len(), z: out });
        }
    }
    if o.regen {
        step(rf, &mut outs, have, |rf, have| Ok(regen::stage(rf, have, ACTOR_PACKS)?.1))?;
    }
    if o.no_fairies {
        step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, flows::GOSSIP_FILE, flows::no_gossip)?]))?;
    }
    if o.lose_heart {
        step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, flows::OVER_FILE, flows::lose_heart)?]))?;
    }
    if o.no_bottles {
        for (file, name) in flows::BOTTLE_FLOWS {
            step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, file, |raw| flows::no_bottles(raw, name))?]))?;
        }
    }
    if o.no_dupes {
        step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, dupes::CATCH_FILE, |raw| dupes::entries(raw, dupes::CATCH_FLOW, &dupes::CATCH_ENTRIES))?]))?;
        step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, dupes::GIVE_FILE, |raw| dupes::entries(raw, dupes::GIVE_FLOW, &dupes::GIVE_ENTRIES))?]))?;
        step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, dupes::SHOP_FILE, |raw| dupes::fronts(raw, dupes::SHOP_FLOW, &dupes::goods()))?]))?;
        for (file, name, item) in dupes::SELLERS {
            step(rf, &mut outs, have, |rf, have| Ok(vec![edit_zs(rf, have, file, |raw| dupes::fronts(raw, name, &[(None, item)]))?]))?;
        }
    }
    if o.respawn {
        for f in respawn::files() {
            step(rf, &mut outs, have, |rf, have| Ok(vec![edit_plain(rf, have, &f, respawn::always)?]))?;
        }
    }
    if o.extra > 0 {
        step(rf, &mut outs, have, |rf, have| extras::stage(rf, have, seed, o.extra))?;
    }
    if o.minibosses {
        for s in &minibosses::SPOTS {
            step(rf, &mut outs, have, |rf, have| Ok(minibosses::one(rf, have, s)?.to_vec()))?;
        }
    }
    if o.minions {
        step(rf, &mut outs, have, minions::stage)?;
    }
    Ok(outs)
}

pub fn worked(rf: &mut Romfs, pk: &crate::modpack::Pack, o: &Opts, seed: &str, pending: &[Out]) -> Res<Vec<Out>> {
    if !o.any() {
        return Ok(Vec::new());
    }
    let root: std::path::PathBuf = pk.work.join("romfs");
    let have = |p: &str| -> Option<Vec<u8>> {
        match pending.iter().rev().find(|x| x.path == p) {
            Some(x) => Some(x.z.clone()),
            None => fs::read(p.split('/').fold(root.clone(), |a, s| a.join(s))).ok(),
        }
    };
    stage(rf, o, &have, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::byml::{Byml, Node};
    use crate::settings::Value;

    #[test]
    fn everything_is_off_by_default_and_the_settings_string_is_unchanged() {
        let mut bad: Vec<String> = Vec::new();
        let o: Opts = read(&Settings::default(), &mut bad);
        assert!(bad.is_empty());
        assert_eq!(o, Opts::default());
        assert!(!o.any() && !o.packs() && o.notes().is_empty());
        let mut s: Settings = Settings::default();
        s.set(HEALTH, Value::Int(100)).unwrap();
        s.set(NO_HEARTS, Value::Bool(false)).unwrap();
        s.set(MORE, Value::s(MORE_OFF)).unwrap();
        s.set(RESPAWN, Value::Bool(false)).unwrap();
        s.set(MINIBOSSES, Value::Bool(false)).unwrap();
        s.set(REGEN, Value::Bool(false)).unwrap();
        s.set(MINIONS, Value::Bool(false)).unwrap();
        assert_eq!(s.encode().unwrap(), Settings::default().encode().unwrap());
    }

    #[test]
    fn each_setting_is_read_and_survives_the_settings_string() {
        let mut s: Settings = Settings::default();
        s.set(HEALTH, Value::Int(250)).unwrap();
        s.set(DAMAGE, Value::Int(50)).unwrap();
        s.set(SPEED, Value::Int(200)).unwrap();
        for id in [NO_HEARTS, NO_FAIRIES, NO_AMMO, LOSE_HEART, NO_BOTTLES, NO_TARGET, NO_DUPES, RESPAWN, MINIBOSSES, REGEN, MINIONS] {
            s.set(id, Value::Bool(true)).unwrap();
        }
        s.set(MORE, Value::s("half")).unwrap();
        let back: Settings = Settings::decode(&s.encode().unwrap()).unwrap().settings;
        let mut bad: Vec<String> = Vec::new();
        let o: Opts = read(&back, &mut bad);
        assert!(bad.is_empty());
        assert_eq!(o, Opts { health: 250, damage: 50, speed: 200, no_hearts: true, no_fairies: true, no_ammo: true, lose_heart: true, no_bottles: true, no_target: true, no_dupes: true, extra: 50, respawn: true, minibosses: true, regen: true, minions: true, ..Opts::default() });
        assert!(o.any() && o.packs());
        assert_eq!(o.notes().len(), 15);
        assert_eq!(o.notes().iter().collect::<std::collections::BTreeSet<&String>>().len(), 15);
        assert!(o.notes().iter().any(|n| n.contains("About 50 in 100")));
        assert!(Opts { extra: 100, ..Opts::default() }.any() && !Opts { extra: 100, ..Opts::default() }.packs());
        assert!(s.set(MORE, Value::s("triple")).is_err() || s.clone().finish().is_err());
        assert!(o.notes().iter().any(|n| n.contains("Queen Gohma (Stalfos)") && n.contains("untested in the game")));
        assert!(o.notes().iter().any(|n| n == minions::NOTE));
        assert!(o.notes()[0].contains("2.5 times"));
        assert!(o.notes()[1].contains("0.5 times"));
        assert!(o.notes()[2].contains("2 times"));
        assert!(s.set(HEALTH, Value::Int(49)).is_err() || s.clone().finish().is_err());
        let mut t: Settings = Settings::default();
        assert!(t.set(SPEED, Value::Int(74)).is_err() || t.finish().is_err());
    }

    fn entry(name: &str, items: Vec<(&str, Node)>) -> Entry {
        let y: Byml = Byml { root: Node::Map(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect()), version: 7, big: false };
        Entry { name: name.to_string(), data: y.write().unwrap() }
    }

    fn max_life(es: &[Entry], name: &str) -> Option<Node> {
        Byml::parse(&sarc::get(es, name)?.data).ok()?.root.get("MaxLife").cloned()
    }

    #[test]
    fn a_pack_only_changes_its_enemy_files() {
        let foe: &str = "Life/LifeParameters/Enemy_Stalfos.game__life__LifeParameters.bgyml";
        let hero: &str = "Life/LifeParameters/Player.game__life__LifeParameters.bgyml";
        let es: Vec<Entry> = vec![entry(foe, vec![("MaxLife", Node::I32(18))]), entry(hero, vec![("MaxLife", Node::I32(12))])];
        let raw: Vec<u8> = sarc::write(&es).unwrap();
        assert!(edit_pack(&raw, &Opts::default()).unwrap().is_none());
        assert!(edit_pack(&raw, &Opts { damage: 300, speed: 150, no_hearts: true, ..Opts::default() }).unwrap().is_none());
        let out: Vec<Entry> = sarc::parse(&edit_pack(&raw, &Opts { health: 300, ..Opts::default() }).unwrap().unwrap()).unwrap();
        assert_eq!(max_life(&out, foe), Some(Node::I32(54)));
        assert_eq!(max_life(&out, hero), Some(Node::I32(12)));
    }

    fn up(y: &Byml) -> Option<&str> {
        y.root.get("$parent")?.as_str()?.rsplit('/').next()?.strip_suffix(".game__attention__ClientParam.gyml")
    }

    fn field(es: &[Entry], stem: &str, key: &str) -> Option<u32> {
        let y: Byml = Byml::parse(&sarc::get(es, &format!("{}{stem}{}", target::DIR, target::EXT))?.data).ok()?;
        if let Some(Node::U32(v)) = y.root.get(key) {
            return Some(*v);
        }
        field(es, up(&y)?, key)
    }

    fn origin(es: &[Entry], stem: &str) -> Option<String> {
        let y: Byml = Byml::parse(&sarc::get(es, &format!("{}{stem}{}", target::DIR, target::EXT))?.data).ok()?;
        if y.root.get(target::KIND).is_some() {
            return Some(stem.to_string());
        }
        origin(es, up(&y)?)
    }

    fn kind(es: &[Entry], stem: &str) -> Option<u32> {
        field(es, stem, target::KIND)
    }

    fn action(es: &[Entry], stem: &str) -> Option<u32> {
        field(es, stem, target::ACTION)
    }

    fn client(stem: &str, kind: &str, action: &str) -> Entry {
        entry(&format!("{}{stem}{}", target::DIR, target::EXT), vec![(target::ACTION, Node::U32(target::hash(action))), (target::KIND, Node::U32(target::hash(kind)))])
    }

    #[test]
    fn no_targeting_only_changes_enemy_lock_on_clients() {
        let child: Entry = entry(&format!("{}Enemy_Stalfos_LockOn{}", target::DIR, target::EXT), vec![("$parent", Node::Str("Work/Attention/ClientParam/EnemyCommon_LockOn.game__attention__ClientParam.gyml".to_string()))]);
        let es: Vec<Entry> = vec![
            client("EnemyCommon_LockOn", "LockOn", "None"),
            child,
            client("EnemyCommon_AutoAim", "AutoAim", "None"),
            client("NpcStandard_LockOn", "LockOn", "Talk"),
            client("Object_M_LockOn", "LockOn", "None"),
            client("Enemy_Scrub_Business_LockOn_EventOn", "LockOn", "Talk"),
        ];
        let raw: Vec<u8> = sarc::write(&es).unwrap();
        assert!(edit_pack(&raw, &Opts { health: 300, no_hearts: true, ..Opts::default() }).unwrap().is_none());
        let out: Vec<u8> = edit_pack(&raw, &Opts { no_target: true, ..Opts::default() }).unwrap().unwrap();
        assert_eq!(out.len(), raw.len());
        let got: Vec<Entry> = sarc::parse(&out).unwrap();
        let look: Option<u32> = Some(target::hash(target::LOOK_ONLY));
        let lock: Option<u32> = Some(target::hash(target::LOCK_ON));
        assert_eq!(kind(&got, "EnemyCommon_LockOn"), look);
        assert_eq!(kind(&got, "Enemy_Stalfos_LockOn"), look);
        assert_eq!(kind(&got, "NpcStandard_LockOn"), lock);
        assert_eq!(kind(&got, "Object_M_LockOn"), lock);
        assert_eq!(kind(&got, "Enemy_Scrub_Business_LockOn_EventOn"), lock);
        let same: usize = got.iter().filter(|x| sarc::get(&es, &x.name).is_some_and(|y| y.data == x.data)).count();
        assert_eq!(same, es.len() - 1);
    }

    fn dump() -> Option<std::path::PathBuf> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        if p.is_dir() { Some(p) } else { None }
    }

    fn none(_p: &str) -> Option<Vec<u8>> {
        None
    }

    #[test]
    fn the_game_files_take_every_option() {
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let o: Opts = Opts { health: 200, damage: 150, speed: 150, no_hearts: true, no_fairies: true, no_ammo: true, lose_heart: true, no_bottles: true, no_target: true, no_dupes: true, extra: 100, respawn: true, minibosses: true, regen: true, minions: true, ..Opts::default() };
        let outs: Vec<Out> = stage(&mut rf, &o, &none, "seed").unwrap();
        let at = |p: &str| -> &Out { outs.iter().find(|x| x.path == p).unwrap_or_else(|| panic!("{p} was not staged")) };
        for f in [flows::GOSSIP_FILE, flows::OVER_FILE, drops::FOLIAGE, lottery::FILE, regen::GLOBAL, regen::ACTOR_INFO, "Pack/Actor/Enemy_Stalfos.pack.zs", "Pack/Actor/Enemy_Keese.pack.zs"] {
            at(f);
        }
        for (f, _) in flows::BOTTLE_FLOWS {
            at(f);
        }
        for f in [dupes::CATCH_FILE, dupes::GIVE_FILE, dupes::SHOP_FILE] {
            at(f);
        }
        let added = |rf: &mut Romfs, p: &str| -> usize {
            let b: crate::bfevfl::Bf = crate::bfevfl::Bf::parse(&rf.zs.decompress(&at(p).z).unwrap()).unwrap();
            b.flows.iter().flat_map(|f| f.events.iter()).filter(|e| e.name.starts_with(dupes::MARK)).count()
        };
        assert_eq!(added(&mut rf, dupes::CATCH_FILE), dupes::CATCH_ENTRIES.len());
        assert_eq!(added(&mut rf, dupes::GIVE_FILE), dupes::GIVE_ENTRIES.len());
        assert_eq!(added(&mut rf, dupes::SHOP_FILE), 2 * dupes::GOODS.len(), "the shop asks for an empty bottle in one place");
        assert_eq!(dupes::SELLERS.map(|(f, _, _)| added(&mut rf, f)), [3, 1]);
        for f in respawn::files() {
            at(&f);
        }
        let only: Vec<Out> = stage(&mut rf, &Opts { respawn: true, ..Opts::default() }, &none, "seed").unwrap();
        assert_eq!(only.len(), respawn::TABLES.len());
        assert!(only.iter().all(|x| x.path.starts_with(respawn::DIR) && x.new > x.old && x.z == respawn::always(&rf.read(&x.path).unwrap()).unwrap()));
        assert!(!outs.iter().any(|x| x.path.contains("Player_")), "the player packs are left alone");
        for x in outs.iter().filter(|x| x.path.ends_with(".pack.zs") && x.path != regen::GLOBAL && x.path != minions::DODONGO_PACK && !x.path.contains("Enemy_Stalfos")) {
            assert!(x.new <= x.old, "{} grew", x.path);
        }
        let es: Vec<Entry> = sarc::parse(&rf.zs.decompress(&at("Pack/Actor/Enemy_Stalfos.pack.zs").z).unwrap()).unwrap();
        let was: Vec<Entry> = sarc::parse(&rf.read_zs("Pack/Actor/Enemy_Stalfos.pack.zs").unwrap()).unwrap();
        let life: &str = "Life/LifeParameters/Enemy_Stalfos.game__life__LifeParameters.bgyml";
        let Some(Node::I32(a)) = max_life(&was, life) else { panic!() };
        assert_eq!(max_life(&es, life), Some(Node::I32(a * 2)));
        let same: usize = es.iter().zip(was.iter()).filter(|(x, y)| x.name == y.name && x.data == y.data).count();
        assert!(same >= es.len() - 20, "only a handful of files change, {same} of {} match", es.len());
        let look: Node = Node::U32(target::hash(target::LOOK_ONLY));
        let aims = |es: &[Entry]| -> usize { es.iter().filter(|e| stats::stem(&e.name, target::DIR, target::EXT).is_some_and(target::is_foe) && Byml::parse(&e.data).unwrap().root.get(target::KIND) == Some(&look)).count() };
        assert!(aims(&was) == 0 && aims(&es) > 0, "the Stalfos pack carries the lock-on edit next to the others");
        let paths: std::collections::BTreeSet<&str> = outs.iter().map(|x| x.path.as_str()).collect();
        assert_eq!(paths.len(), outs.len(), "no file is staged twice");
        let boss: Vec<Entry> = sarc::parse(&rf.zs.decompress(&at(minions::DODONGO_PACK).z).unwrap()).unwrap();
        let calm: Vec<Entry> = sarc::parse(&rf.read_zs(minions::DODONGO_PACK).unwrap()).unwrap();
        let moved = |n: &str| -> bool { sarc::get(&boss, n).map(|e| &e.data) != sarc::get(&calm, n).map(|e| &e.data) };
        assert!(moved(minions::DODONGO_PARAM), "King Dodongo keeps his extra Baby Dodongos");
        assert!(calm.iter().any(|e| e.name.starts_with(stats::LIFE_DIR) && moved(&e.name)), "King Dodongo keeps his scaled health");
        let count = |y: &Byml| -> usize { y.root.get("Actors").and_then(|a| a.as_arr()).map_or(0, |a| a.len()) };
        let has = |y: &Byml, g: &str| -> usize { y.root.get("Actors").and_then(|a| a.as_arr()).unwrap_or(&[]).iter().filter(|a| a.get("Gyaml").and_then(|v| v.as_str()) == Some(g)).count() };
        let solo: Vec<Out> = extras::stage(&mut rf, &none, "seed", 100).unwrap();
        for r in minions::ROOMS.iter() {
            let now: Byml = Byml::parse(&rf.zs.decompress(&at(r.free).z).unwrap()).unwrap();
            let base: Byml = match solo.iter().find(|x| x.path == r.free) {
                Some(x) => Byml::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap(),
                None => rf.read_byml(r.free).unwrap(),
            };
            assert_eq!(count(&now), count(&base) + r.foes.len(), "{} carries the extra enemies and the minions", r.free);
        }
        for s in minibosses::SPOTS.iter() {
            let now: Byml = Byml::parse(&rf.zs.decompress(&at(s.file).z).unwrap()).unwrap();
            let base: Byml = match solo.iter().find(|x| x.path == s.file) {
                Some(x) => Byml::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap(),
                None => rf.read_byml(s.file).unwrap(),
            };
            assert_eq!(count(&now), count(&base) + 1, "{} carries the miniboss", s.file);
            assert_eq!(has(&now, s.mini), has(&base, s.mini) + 1);
        }
        let graph: crate::ainb::Ainb = crate::ainb::Ainb::parse(&sarc::get(&es, regen::STALFOS_GRAPH).unwrap().data).unwrap();
        assert_eq!(graph.find_els(regen::HEAL).len(), 1, "the Stalfos pack carries both the health and the regen edit");
        assert_eq!(outs.iter().filter(|x| x.path == "Pack/Actor/Enemy_Stalfos.pack.zs").count(), 1);
        let table: Byml = Byml::parse(&rf.zs.decompress(&at(regen::ACTOR_INFO).z).unwrap()).unwrap();
        let before: Byml = rf.read_byml(regen::ACTOR_INFO).unwrap();
        let heap = |y: &Byml, n: &str| -> Option<Node> { crate::pickup_pack::rows::find(y, n)?.get(crate::pickup_pack::rows::HEAP).cloned() };
        let Some(Node::I32(h)) = heap(&before, "Enemy_Stalfos") else { panic!() };
        assert_eq!(heap(&table, "Enemy_Stalfos"), Some(Node::I32(h + regen::HEAP_EXTRA)));
        assert_eq!(heap(&table, "Player_Child"), heap(&before, "Player_Child"));
        assert!(stage(&mut rf, &Opts::default(), &none, "seed").unwrap().is_empty());
    }

    #[test]
    fn the_game_files_lose_enemy_lock_on_and_nothing_else() {
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let o: Opts = Opts { no_target: true, ..Opts::default() };
        let outs: Vec<Out> = stage(&mut rf, &o, &none, "seed").unwrap();
        let lock: u32 = target::hash(target::LOCK_ON);
        let look: u32 = target::hash(target::LOOK_ONLY);
        let idle: u32 = target::hash(target::NO_ACTION);
        let mut copies: std::collections::BTreeMap<String, Vec<u8>> = std::collections::BTreeMap::new();
        let mut packs: usize = 0;
        let mut kept: usize = 0;
        for f in pack_files(&rf, &o).unwrap() {
            let was: Vec<Entry> = sarc::parse(&rf.read_zs(&f).unwrap()).unwrap();
            let now: Vec<Entry> = match outs.iter().find(|x| x.path == f) {
                Some(x) => {
                    assert_eq!(x.new, x.old, "{f} changed size");
                    packs += 1;
                    sarc::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap()
                }
                None => was.clone(),
            };
            assert_eq!(now.len(), was.len());
            for (a, b) in now.iter().zip(was.iter()) {
                assert_eq!(a.name, b.name);
                let Some(s) = stats::stem(&a.name, target::DIR, target::EXT) else {
                    assert!(a.data == b.data, "{f} {} changed", a.name);
                    continue;
                };
                if a.data != b.data {
                    assert!(target::is_foe(s), "{f} {s} is not an enemy client");
                    assert_eq!(a.data.len(), b.data.len());
                    let seen: &Vec<u8> = copies.entry(s.to_string()).or_insert_with(|| a.data.clone());
                    assert!(*seen == a.data, "{s} differs between packs");
                } else {
                    assert!(!copies.contains_key(s), "{f} keeps an unedited copy of {s}");
                }
                assert_eq!(action(&now, s), action(&was, s), "{f} {s}");
                if kind(&was, s) != Some(lock) {
                    assert_eq!(kind(&now, s), kind(&was, s), "{f} {s}");
                    continue;
                }
                if origin(&was, s).is_some_and(|x| target::is_foe(&x)) && action(&was, s) == Some(idle) {
                    assert_eq!(kind(&now, s), Some(look), "{f} {s} can still be locked on to");
                } else {
                    assert_eq!(kind(&now, s), Some(lock), "{f} {s} lost its lock-on");
                    kept += 1;
                }
            }
        }
        assert_eq!(packs, outs.len());
        assert!(packs > 100, "{packs} packs changed");
        assert!(copies.contains_key("EnemyCommon_LockOn") && copies.contains_key("Ganon_LockOn"));
        assert!(copies.len() < 30, "{} files changed", copies.len());
        assert!(kept > 100, "{kept} lock-on clients of people, signs and objects are kept");
        assert!(!outs.iter().any(|x| x.path.contains("Player") || x.path.contains("Npc_")), "player and people packs are left alone");
    }

    #[test]
    fn the_game_files_take_a_miniboss_in_each_covered_boss_room() {
        use crate::ainb::Ainb;
        use crate::placement::hash_of;
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let o: Opts = Opts { minibosses: true, ..Opts::default() };
        assert!(o.any() && !o.packs());
        let outs: Vec<Out> = stage(&mut rf, &o, &none, "seed").unwrap();
        assert_eq!(outs.len(), 2 * minibosses::SPOTS.len());
        for (s, pair) in minibosses::SPOTS.iter().zip(outs.chunks(2)) {
            assert!(rf.root.join(crate::pickup_pack::pack_of(s.mini)).is_file(), "{} has no actor pack", s.mini);
            assert_eq!(pair[0].path, s.file);
            let was: Byml = Byml::parse(&rf.read_zs(s.file).unwrap()).unwrap();
            let now: Byml = Byml::parse(&rf.zs.decompress(&pair[0].z).unwrap()).unwrap();
            let list = |y: &Byml| -> Vec<Node> { y.root.get("Actors").and_then(|a| a.as_arr()).unwrap().to_vec() };
            let (a, b): (Vec<Node>, Vec<Node>) = (list(&was), list(&now));
            assert_eq!(b.len(), a.len() + 1);
            assert_eq!(&b[..a.len()], &a[..], "{}: the actors of the game are untouched", s.file);
            let new: &Node = &b[a.len()];
            let boss: &Node = a.iter().find(|x| x.get("Gyaml").and_then(|g| g.as_str()) == Some(s.actor)).unwrap();
            assert_eq!(new.get("Gyaml").and_then(|g| g.as_str()), Some(s.mini));
            assert_eq!(new.get("Dynamic").and_then(|d| d.get(minibosses::ROOM)), boss.get("Dynamic").and_then(|d| d.get(minibosses::ROOM)));
            let h: u64 = hash_of(new).unwrap();
            assert!(!a.iter().any(|x| hash_of(x) == Some(h)));
            let heart: &Node = a.iter().find(|x| x.get("Gyaml").and_then(|g| g.as_str()) == Some("Obj_Heart_Container") && x.get("Dynamic").and_then(|d| d.get(minibosses::ROOM)) == boss.get("Dynamic").and_then(|d| d.get(minibosses::ROOM))).unwrap();
            let pos = |n: &Node| -> Vec<f32> { n.get("Translate").and_then(|t| t.as_arr()).unwrap().iter().map(|v| if let Node::F32(f) = v { *f } else { panic!() }).collect() };
            let (m, c): (Vec<f32>, Vec<f32>) = (pos(new), pos(heart));
            let flat: f32 = ((m[0] - c[0]).powi(2) + (m[2] - c[2]).powi(2)).sqrt();
            assert!(flat > 2.0 && flat < 25.0 && (m[1] - c[1]).abs() < 1.0, "{}: {flat} from the heart container, {} above it", s.boss, m[1] - c[1]);
            let raw: Vec<u8> = rf.read(&pair[1].path).unwrap();
            assert!(pair[1].path.starts_with("Logic/") && pair[1].new > pair[1].old && pair[1].old == raw.len());
            let old: Ainb = Ainb::parse(&raw).unwrap();
            let g: Ainb = Ainb::parse(&pair[1].z).unwrap();
            assert_eq!(g.write().unwrap(), pair[1].z);
            assert!(crate::ainb::audit(&g).is_empty());
            let n: usize = old.els.len();
            assert_eq!(g.els.len(), n + 1);
            let plain = |e: &[u8; crate::ainb::EL]| -> Vec<u8> { e.iter().enumerate().map(|(j, v)| if (0x14..0x18).contains(&j) { 0 } else { *v }).collect() };
            assert!(g.els[..n].iter().zip(old.els.iter()).all(|(x, y)| plain(x) == plain(y)), "{}: the nodes of the game are untouched", pair[1].path);
            assert_eq!(&g.params[..n], &old.params[..], "{}: the inputs of the game are untouched", pair[1].path);
            let id: &str = g.prop_str(n, minibosses::NAME).unwrap();
            let groups: Vec<Node> = now.root.get("AiGroups").and_then(|x| x.as_arr()).unwrap().to_vec();
            let named = |r: &Node| r.get("Id").and_then(|v| v.as_str()) == Some(id) && r.get("Reference") == Some(&Node::U64(h));
            let home: Vec<&Node> = groups.iter().filter(|x| x.get("References").and_then(|r| r.as_arr()).is_some_and(|r| r.iter().any(named))).collect();
            assert_eq!(home.len(), 1);
            assert_eq!(minibosses::logic_file(home[0].get("Logic").and_then(|v| v.as_str()).unwrap()).unwrap(), pair[1].path);
            let hn: usize = (0..n).find(|&i| g.prop_str(i, minibosses::NAME).is_some_and(|x| x.starts_with(minibosses::HEART))).unwrap();
            let made: crate::ainb::Input = g.input(1, g.find_in(hn, 1, minibosses::CREATE).unwrap()).unwrap();
            let gone: crate::ainb::Input = g.input(1, g.find_in(n, 1, minibosses::DELETE).unwrap()).unwrap();
            assert!(made.src >= 0 && (gone.src, gone.plug) == (made.src, made.plug), "{}: the miniboss leaves on the flag that brings the heart container", s.boss);
            let twin: Vec<u8> = {
                let mut t: Ainb = old.clone();
                minibosses::wire(&mut t, g.prop_str(hn, minibosses::NAME).unwrap(), id).unwrap();
                t.write().unwrap()
            };
            assert_eq!(twin, pair[1].z);
        }
        let again: Vec<Out> = stage(&mut rf, &o, &none, "seed").unwrap();
        assert!(outs.iter().zip(again.iter()).all(|(x, y)| x.path == y.path && x.z == y.z), "the edit is the same on every build");
    }

    fn list(y: &Byml) -> &[Node] {
        y.root.get("Actors").and_then(|a| a.as_arr()).unwrap_or(&[])
    }

    #[test]
    fn the_game_files_take_the_boss_minions() {
        use crate::placement::hash_of;
        use std::collections::BTreeSet as Set;
        let Some(root) = dump() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        let o: Opts = Opts { minions: true, ..Opts::default() };
        assert!(o.any() && !o.packs());
        let outs: Vec<Out> = stage(&mut rf, &o, &none, "seed").unwrap();
        assert_eq!(outs.len(), minions::ROOMS.len() + 1);
        let paths: Set<&str> = outs.iter().map(|x| x.path.as_str()).collect();
        assert_eq!(paths.len(), outs.len());
        for r in minions::ROOMS.iter() {
            let x: &Out = outs.iter().find(|x| x.path == r.free).unwrap();
            let was: Byml = rf.read_byml(r.free).unwrap();
            let now: Byml = Byml::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap();
            let fixed: Byml = rf.read_byml(r.fixed).unwrap();
            let room: i32 = minions::room_of(&fixed, r).unwrap();
            let n: usize = list(&was).len();
            assert_eq!(list(&now).len(), n + r.foes.len());
            assert_eq!(&list(&now)[..n], list(&was));
            assert_eq!(x.old, was.write().unwrap().len(), "{} does not rewrite to its own size", r.free);
            let ids: Set<u64> = list(&now).iter().chain(list(&fixed).iter()).filter_map(hash_of).collect();
            assert_eq!(ids.len(), list(&now).len() + list(&fixed).len(), "a hash is used twice in {}", r.free);
            let peer = |g: &str| list(&was).iter().chain(list(&fixed).iter()).any(|a| a.get("Gyaml").and_then(|v| v.as_str()) == Some(g));
            for (i, a) in list(&now)[n..].iter().enumerate() {
                assert_eq!(a.get("Gyaml").and_then(|g| g.as_str()), Some(r.foes[i]));
                assert_eq!(a.get("Dynamic").and_then(|d| d.get(minions::ROOM)), Some(&Node::I32(room)));
                assert!(rf.read(&format!("Pack/Actor/{}.pack.zs", r.foes[i])).is_ok());
            }
            eprintln!("{}: room {room}, {:?}, already in the dungeon: {:?}", r.boss, r.foes, r.foes.iter().map(|g| peer(g)).collect::<Vec<bool>>());
        }
        let x: &Out = outs.iter().find(|x| x.path == minions::DODONGO_PACK).unwrap();
        let es: Vec<Entry> = sarc::parse(&rf.zs.decompress(&x.z).unwrap()).unwrap();
        let was: Vec<Entry> = sarc::parse(&rf.read_zs(minions::DODONGO_PACK).unwrap()).unwrap();
        let rolls = |es: &[Entry]| -> Vec<i32> {
            let y: Byml = Byml::parse(&sarc::get(es, minions::DODONGO_PARAM).unwrap().data).unwrap();
            let st: &[Node] = y.root.get(minions::STAGES).unwrap().as_arr().unwrap();
            st.iter().map(|s| if let Some(Node::I32(n)) = s.get(minions::AFTER_ROLL) { *n } else { 0 }).collect()
        };
        assert_eq!(rolls(&was), vec![0, 2, 3]);
        assert_eq!(rolls(&es), vec![2, 3, 4]);
        let same: usize = es.iter().zip(was.iter()).filter(|(x, y)| x.name == y.name && x.data == y.data).count();
        assert_eq!(same, es.len() - 1);
        let both: Vec<Out> = stage(&mut rf, &Opts { health: 200, minions: true, ..Opts::default() }, &none, "seed").unwrap();
        assert_eq!(both.iter().filter(|x| x.path == minions::DODONGO_PACK).count(), 1);
        let es: Vec<Entry> = sarc::parse(&rf.zs.decompress(&both.iter().find(|x| x.path == minions::DODONGO_PACK).unwrap().z).unwrap()).unwrap();
        assert_eq!(rolls(&es), vec![2, 3, 4]);
    }
}
