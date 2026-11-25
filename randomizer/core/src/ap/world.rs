//! The OOTRR world file for Archipelago (`ootrr.apworld`), packed by the app itself: the Python
//! sources of `archipelago/ootrr` built into the exe, the logic bundles `ootrr ap-export` writes,
//! and the container manifest. The same contents as `tools/build_apworld.py`, stored without
//! compression, plus `ootrr/built_by.txt` with the app version that packed it.

use crate::error::{Error as E, Res};
use crate::json::Json;
use crate::settings::presets;
use crate::settings::Settings;
use crate::slice::data::Data;
use crate::tex::png::crc32;

use super::export;

pub const FILE: &str = "ootrr.apworld";
/// The container version Archipelago 0.6.7 and newer read.
pub const CONTAINER: i64 = 7;
pub const MANIFEST: &str = "ootrr/archipelago.json";
pub const BUILT_BY: &str = "ootrr/built_by.txt";

/// The Python sources of the world, as `tools/build_apworld.py` packs them.
pub const SOURCES: [(&str, &str); 6] = [
    ("__init__.py", include_str!("../../../../archipelago/ootrr/__init__.py")),
    ("client.py", include_str!("../../../../archipelago/ootrr/client.py")),
    ("components.py", include_str!("../../../../archipelago/ootrr/components.py")),
    ("logic.py", include_str!("../../../../archipelago/ootrr/logic.py")),
    ("options.py", include_str!("../../../../archipelago/ootrr/options.py")),
    ("tables.py", include_str!("../../../../archipelago/ootrr/tables.py")),
];
const MANIFEST_SRC: &str = include_str!("../../../../archipelago/ootrr/archipelago.json");
const LICENSE: &str = include_str!("../../../../LICENSE");
const NOTICE: &str = include_str!("../../../../NOTICE");

fn unix(t: &str) -> String {
    t.replace("\r\n", "\n")
}

/// The `world_version` of the world this app packs.
pub fn version() -> String {
    crate::json::parse(MANIFEST_SRC).ok().and_then(|j| j.get("world_version").and_then(|v| v.as_str()).map(str::to_string)).unwrap_or_default()
}

fn manifest() -> Res<String> {
    let j: Json = crate::json::parse(MANIFEST_SRC)?;
    let rows: &[(String, Json)] = j.as_obj().ok_or(E::Format("archipelago.json is not an object"))?;
    let mut out: Json = Json::obj();
    for (k, v) in rows {
        if k != "version" && k != "compatible_version" {
            out.put(k, v.clone());
        }
    }
    out.put("version", Json::Int(CONTAINER));
    out.put("compatible_version", Json::Int(CONTAINER));
    Ok(out.to_string())
}

/// One preset the world file carries.
#[derive(Debug, Clone, PartialEq)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub settings: Settings,
}

/// The built-in presets Archipelago can use by name: the ones whose logic bundle exports.
/// With the bundle text of each, in the same order.
fn bundles(d: &Data) -> Res<Vec<(Preset, String, Json)>> {
    let mut out: Vec<(Preset, String, Json)> = Vec::new();
    for p in presets::builtin()? {
        let s: Settings = p.apply(&Settings::default())?;
        if let Ok((b, _)) = export::export_with(d, &s, &p.id, export::SEED, 0) {
            let row: Json = Json::obj().with("id", Json::Str(p.id.clone())).with("name", Json::Str(p.name.clone())).with("tip", Json::Str(p.tip.clone()));
            out.push((Preset { id: p.id.clone(), name: p.name.clone(), settings: s }, b.to_string(), row));
        }
    }
    Ok(out)
}

pub fn presets_in_world() -> Res<Vec<Preset>> {
    let d: Data = Data::load()?;
    Ok(bundles(&d)?.into_iter().map(|(p, _, _)| p).collect())
}

/// Every file of the world, by its path inside the archive.
pub fn files(app_version: &str) -> Res<Vec<(String, Vec<u8>)>> {
    if super::build::running() {
        return Err(E::Msg("An Archipelago mod is being built. Wait for it to finish, then try again.".to_string()));
    }
    let d: Data = Data::load()?;
    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, text) in SOURCES {
        out.push((format!("ootrr/{name}"), unix(text).into_bytes()));
    }
    out.push((MANIFEST.to_string(), manifest()?.into_bytes()));
    let mut index: Vec<Json> = Vec::new();
    for (p, text, row) in bundles(&d)? {
        out.push((format!("ootrr/data/{}.json", p.id), text.into_bytes()));
        index.push(row);
    }
    out.push(("ootrr/data/presets.json".to_string(), Json::Arr(index).to_string().into_bytes()));
    out.push(("ootrr/data/names.json".to_string(), export::names(&d)?.to_string().into_bytes()));
    out.push(("ootrr/LICENSE".to_string(), unix(LICENSE).into_bytes()));
    out.push(("ootrr/NOTICE".to_string(), unix(NOTICE).into_bytes()));
    out.push((BUILT_BY.to_string(), app_version.as_bytes().to_vec()));
    Ok(out)
}

/// The bytes of `ootrr.apworld`.
pub fn pack(app_version: &str) -> Res<Vec<u8>> {
    zip(&files(app_version)?)
}

const LOCAL: u32 = 0x0403_4b50;
const CENTRAL: u32 = 0x0201_4b50;
const END: u32 = 0x0605_4b50;
/// Names are UTF-8.
const FLAGS: u16 = 0x0800;
/// 1980-01-01, so the same input gives the same bytes.
const DATE: u16 = 0x0021;

fn u16le(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_le_bytes());
}

fn u32le(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}

/// A ZIP archive with every entry stored (no compression).
pub fn zip(files: &[(String, Vec<u8>)]) -> Res<Vec<u8>> {
    let too_big = || E::Format("the world file does not fit in a zip archive");
    let count: u16 = u16::try_from(files.len()).map_err(|_| too_big())?;
    let mut out: Vec<u8> = Vec::new();
    let mut dir: Vec<u8> = Vec::new();
    for (name, data) in files {
        let at: u32 = u32::try_from(out.len()).map_err(|_| too_big())?;
        let len: u32 = u32::try_from(data.len()).map_err(|_| too_big())?;
        let nlen: u16 = u16::try_from(name.len()).map_err(|_| too_big())?;
        let crc: u32 = crc32(data);
        u32le(&mut out, LOCAL);
        for x in [20, FLAGS, 0, 0, DATE] {
            u16le(&mut out, x);
        }
        for x in [crc, len, len] {
            u32le(&mut out, x);
        }
        u16le(&mut out, nlen);
        u16le(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        u32le(&mut dir, CENTRAL);
        for x in [20, 20, FLAGS, 0, 0, DATE] {
            u16le(&mut dir, x);
        }
        for x in [crc, len, len] {
            u32le(&mut dir, x);
        }
        for x in [nlen, 0, 0, 0, 0] {
            u16le(&mut dir, x);
        }
        u32le(&mut dir, 0);
        u32le(&mut dir, at);
        dir.extend_from_slice(name.as_bytes());
    }
    let dir_at: u32 = u32::try_from(out.len()).map_err(|_| too_big())?;
    let dir_len: u32 = u32::try_from(dir.len()).map_err(|_| too_big())?;
    out.extend_from_slice(&dir);
    u32le(&mut out, END);
    for x in [0, 0, count, count] {
        u16le(&mut out, x);
    }
    u32le(&mut out, dir_len);
    u32le(&mut out, dir_at);
    u16le(&mut out, 0);
    Ok(out)
}

fn r16(b: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?) as usize)
}

fn r32(b: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?) as usize)
}

/// The bytes of a stored entry of a ZIP archive. `None` when the archive cannot be read, has no
/// such entry or compressed it (a world file packed by `tools/build_apworld.py`).
pub fn stored(zip: &[u8], name: &str) -> Option<Vec<u8>> {
    let end: usize = (0..=zip.len().checked_sub(22)?).rev().take(66_000).find(|&i| r32(zip, i) == Some(END as usize))?;
    let count: usize = r16(zip, end + 10)?;
    let mut at: usize = r32(zip, end + 16)?;
    for _ in 0..count {
        if r32(zip, at)? != CENTRAL as usize {
            return None;
        }
        let (method, len, nlen, xlen, clen, local): (usize, usize, usize, usize, usize, usize) = (r16(zip, at + 10)?, r32(zip, at + 20)?, r16(zip, at + 28)?, r16(zip, at + 30)?, r16(zip, at + 32)?, r32(zip, at + 42)?);
        if zip.get(at + 46..at + 46 + nlen)? == name.as_bytes() {
            if method != 0 || r32(zip, local)? != LOCAL as usize {
                return None;
            }
            let data: usize = local + 30 + r16(zip, local + 26)? + r16(zip, local + 28)?;
            return zip.get(data..data + len).map(<[u8]>::to_vec);
        }
        at += 46 + nlen + xlen + clen;
    }
    None
}

/// What a world file on disk says about itself.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Installed {
    /// `world_version` of its manifest; empty when it could not be read.
    pub version: String,
    /// The app version that packed it; empty for a world file this app did not pack.
    pub built_by: String,
}

pub fn installed(zip: &[u8]) -> Installed {
    let version: String = stored(zip, MANIFEST)
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|t| crate::json::parse(&t).ok())
        .and_then(|j| j.get("world_version").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let built_by: String = stored(zip, BUILT_BY).and_then(|b| String::from_utf8(b).ok()).unwrap_or_default();
    Installed { version, built_by }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zip_reads_back_its_stored_entries() {
        let files: Vec<(String, Vec<u8>)> = vec![("a/x.txt".to_string(), b"hello".to_vec()), ("a/empty".to_string(), Vec::new()), ("a/y.bin".to_string(), vec![0, 255, 7])];
        let z: Vec<u8> = zip(&files).unwrap();
        for (n, d) in &files {
            assert_eq!(stored(&z, n).as_ref(), Some(d), "{n}");
        }
        assert_eq!(stored(&z, "a/none"), None);
        assert_eq!(stored(b"not a zip", "a/x.txt"), None);
        assert_eq!(stored(&z[..z.len() - 3], "a/x.txt"), None);
        assert_eq!(zip(&files).unwrap(), z);
    }

    #[test]
    fn the_embedded_sources_are_every_file_of_the_world_folder() {
        let dir: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../archipelago/ootrr");
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n != ".gitignore" && !n.ends_with(".pyc"))
            .collect();
        on_disk.sort();
        let mut ours: Vec<String> = SOURCES.iter().map(|(n, _)| n.to_string()).collect();
        ours.push("archipelago.json".to_string());
        ours.sort();
        assert_eq!(ours, on_disk, "add new files of archipelago/ootrr to SOURCES");
    }

    #[test]
    fn the_manifest_gets_the_container_version_and_keeps_the_world_version() {
        let j: Json = crate::json::parse(&manifest().unwrap()).unwrap();
        assert_eq!(j.get("version").and_then(|v| v.as_i64()), Some(CONTAINER));
        assert_eq!(j.get("compatible_version").and_then(|v| v.as_i64()), Some(CONTAINER));
        assert_eq!(j.get("game").and_then(|v| v.as_str()), Some("OOTRR"));
        assert_eq!(j.get("world_version").and_then(|v| v.as_str()), Some(version().as_str()));
        assert!(!version().is_empty());
    }

    #[test]
    fn the_packed_world_holds_the_sources_the_bundles_and_who_packed_it() {
        let z: Vec<u8> = pack("9.9.9").unwrap();
        assert_eq!(installed(&z), Installed { version: version(), built_by: "9.9.9".to_string() });
        let init: Vec<u8> = stored(&z, "ootrr/__init__.py").unwrap();
        assert!(!init.contains(&b'\r') && init.starts_with(b"import"));
        let index: Json = crate::json::parse(&String::from_utf8(stored(&z, "ootrr/data/presets.json").unwrap()).unwrap()).unwrap();
        let ids: Vec<&str> = index.as_arr().unwrap().iter().map(|p| p.get("id").and_then(|v| v.as_str()).unwrap()).collect();
        assert!(ids.contains(&"default"));
        for id in &ids {
            assert!(stored(&z, &format!("ootrr/data/{id}.json")).is_some(), "{id}");
        }
        assert!(stored(&z, "ootrr/data/names.json").is_some());
        let named: Vec<String> = presets_in_world().unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(named, ids);
    }
}
