use std::path::Path as P;

use ootrr_core::ap::{self, mailbox::Record, markers, saved};
use ootrr_core::error::{Error, Res};
use ootrr_core::json::Json;
use ootrr_core::romfs::Romfs;
use ootrr_core::settings::presets;
use ootrr_core::settings::Settings;
use ootrr_core::slice::data::Data;
use ootrr_core::staging::{commit, Committed, Out};
use ootrr_mem::game::Game;
use ootrr_mem::gdm;

pub const TEST_SEED: &str = "aptest";

pub fn test_mod(a: &[String]) -> Res<()> {
    let root: &str = a.first().map(|s| s.as_str()).ok_or(Error::Format("missing romfs_root"))?;
    let load: &str = a.get(1).map(|s| s.as_str()).ok_or(Error::Format("missing load_dir"))?;
    let name: &str = a.get(2).map(|s| s.as_str()).ok_or(Error::Format("missing mod_name"))?;
    let actors: &[String] = a.get(3..).unwrap_or(&[]);
    if actors.is_empty() {
        return Err(Error::Format("give at least one item actor, for example Item_Arrow_Fire"));
    }
    let recs: Vec<Record> = actors.iter().enumerate().map(|(i, x)| Record::new(&format!("{:012x}", i + 1), x)).collect::<Res<_>>()?;
    let mut rf: Romfs = Romfs::open(P::new(root))?;
    let mut outs: Vec<Out> = Vec::new();
    ap::stage(&mut rf, &mut outs, P::new(root), TEST_SEED, &recs)?;
    let c: Committed = commit(&mut rf, P::new(load), name, &outs, None)?;
    print!("{}", c.report);
    println!("seed flag {} (set when a new game reaches Kokiri Forest)", markers::seed_flag(TEST_SEED));
    println!("goal flag {} (set after Ganon, before the ending)", markers::goal_flag(TEST_SEED));
    for r in &recs {
        println!("{}: ootrr ap-flag {} on   gives it, then {} turns true", r.item, r.recv(), r.got());
    }
    Ok(())
}

pub fn flag(a: &[String]) -> Res<()> {
    let name: &str = a.first().map(|s| s.as_str()).ok_or(Error::Format("missing flag name"))?;
    let g: Game = Game::attach()?;
    let h: u32 = saved::hash(name);
    match a.get(1).map(|s| s.as_str()) {
        None => println!("{name} = {}", gdm::get_bool(&g, h)?),
        Some(v @ ("on" | "off")) => {
            let was: bool = gdm::set_bool(&g, h, v == "on")?;
            println!("{name}: {was} -> {}", v == "on");
        }
        Some(x) => return Err(Error::Msg(format!("the value must be on or off, not {x}"))),
    }
    Ok(())
}

fn after<'a>(a: &'a [String], flag: &str) -> Option<&'a str> {
    a.iter().position(|x| x == flag).and_then(|i| a.get(i + 1)).map(|s| s.as_str())
}

fn write_new(path: &P, text: &str) -> Res<()> {
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn export_one(a: &[String]) -> Res<()> {
    let usage = Error::Format("ap-export --settings <settings string> --out <file> [--seed <seed>] [--probes <file>]");
    let text: &str = a.first().map(|s| s.as_str()).ok_or(usage)?;
    let out: &P = P::new(after(a, "--out").ok_or(Error::Format("ap-export --settings needs --out <file>"))?);
    let seed: &str = after(a, "--seed").unwrap_or(ap::export::SEED);
    let probes: Option<&P> = after(a, "--probes").map(P::new);
    let s: Settings = Settings::decode(text)?.settings;
    let d: Data = Data::load()?;
    let (b, c) = ap::export::export_with(&d, &s, "custom", seed, if probes.is_some() { 40 } else { 0 })?;
    if let Some(p) = probes {
        write_new(p, &c.to_string())?;
    }
    write_new(out, &b.to_string())?;
    println!("custom: {} shuffled", b.get("free").and_then(|v| v.as_arr()).map(|v| v.len()).unwrap_or(0));
    Ok(())
}

fn export_rolls(a: &[String]) -> Res<()> {
    let usage = || Error::Format("ap-export --roll <count> <folder>");
    let n: usize = a.first().and_then(|s| s.parse().ok()).ok_or_else(usage)?;
    let out: &P = P::new(a.get(1).ok_or_else(usage)?);
    std::fs::create_dir_all(out)?;
    let d: Data = Data::load()?;
    let ps = presets::builtin()?;
    let mut rng = ootrr_core::rng::Rng::from_text("ootrr archipelago rolls");
    let mut index: Vec<Json> = Vec::new();
    let mut k: usize = 0;
    while index.len() < n && k < n * 8 {
        let base: Settings = ps[k % ps.len()].apply(&Settings::default())?;
        k += 1;
        let Some(s) = ap::export::roll(&mut rng, &base, 12) else { continue };
        let id: String = format!("roll_{}", index.len());
        let seed: String = format!("roll{k}");
        match ap::export::export_with(&d, &s, &id, &seed, 40) {
            Ok((b, c)) => {
                std::fs::write(out.join(format!("{id}.json")), b.to_string())?;
                std::fs::write(out.join(format!("{id}.probes.json")), c.to_string())?;
                println!("{id} (from {}): {} shuffled", ps[(k - 1) % ps.len()].id, b.get("free").and_then(|v| v.as_arr()).map(|v| v.len()).unwrap_or(0));
                index.push(Json::obj().with("id", Json::Str(id)).with("settings", Json::Str(s.encode()?)).with("seed", Json::Str(seed)));
            }
            Err(e) => println!("roll {k}: skipped, {e}"),
        }
    }
    std::fs::write(out.join("rolls.json"), Json::Arr(index).to_string())?;
    Ok(())
}

pub fn export(a: &[String]) -> Res<()> {
    match a.first().map(|s| s.as_str()) {
        Some("--settings") => return export_one(&a[1..]),
        Some("--roll") => return export_rolls(&a[1..]),
        _ => {}
    }
    let out: &P = P::new(a.first().map(|s| s.as_str()).ok_or(Error::Format("missing out folder"))?);
    let probes: Option<&P> = match (a.get(1).map(|s| s.as_str()), a.get(2)) {
        (Some("--probes"), Some(dir)) => Some(P::new(dir)),
        (None, _) => None,
        _ => return Err(Error::Format("ap-export <out> [--probes <folder>]")),
    };
    std::fs::create_dir_all(out)?;
    if let Some(p) = probes {
        std::fs::create_dir_all(p)?;
    }
    let d: Data = Data::load()?;
    let mut index: Vec<Json> = Vec::new();
    for p in presets::builtin()? {
        let s: Settings = p.apply(&Settings::default())?;
        match ap::export::export_with(&d, &s, &p.id, ap::export::SEED, if probes.is_some() { 40 } else { 0 }) {
            Ok((b, c)) => {
                if let Some(dir) = probes {
                    std::fs::write(dir.join(format!("{}.probes.json", p.id)), c.to_string())?;
                }
                let text: String = b.to_string();
                std::fs::write(out.join(format!("{}.json", p.id)), &text)?;
                println!("{}: {} bytes, {} shuffled", p.id, text.len(), b.get("free").and_then(|v| v.as_arr()).map(|v| v.len()).unwrap_or(0));
                index.push(Json::obj().with("id", Json::Str(p.id.clone())).with("name", Json::Str(p.name.clone())).with("tip", Json::Str(p.tip.clone())));
            }
            Err(e) => println!("{}: skipped, {e}", p.id),
        }
    }
    std::fs::write(out.join("presets.json"), Json::Arr(index).to_string())?;
    std::fs::write(out.join("names.json"), ap::export::names(&d)?.to_string())?;
    Ok(())
}

pub fn build(a: &[String]) -> Res<()> {
    let mut rest: Vec<String> = Vec::new();
    let mut host: ootrr_core::modpack::Host = ootrr_core::modpack::Host::Eden;
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] == "--host" {
            let v: &str = a.get(i + 1).map(|s| s.as_str()).ok_or(Error::Format("missing host after --host"))?;
            host = ootrr_core::modpack::Host::parse(v).ok_or(Error::Format("--host is eden, hookshot, longshot or nx2navi"))?;
            i += 2;
            continue;
        }
        rest.push(a[i].clone());
        i += 1;
    }
    let path: &str = rest.first().map(|s| s.as_str()).ok_or(Error::Format("ap-build <patch.apootrr> <romfs_root> <load_dir> [mod_name] [--host longshot]"))?;
    let romfs: &P = P::new(rest.get(1).map(|s| s.as_str()).ok_or(Error::Format("missing romfs_root"))?);
    let load: &P = P::new(rest.get(2).map(|s| s.as_str()).ok_or(Error::Format("missing load_dir"))?);
    let p: ap::patch::Patch = ap::build::read(&std::fs::read_to_string(path)?)?;
    let name: String = rest.get(3).cloned().unwrap_or_else(|| ap::build::default_name(&p.slot));
    let (b, _) = ap::build::build(&p, romfs, load, &name, host)?;
    print!("{}", b.report);
    println!("mod written to {}", b.root.display());
    Ok(())
}
