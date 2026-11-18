use std::fs;
use std::path::{Path as P, PathBuf as PB};

use crate::error::{Error as E, Res};
use crate::json::{self, Json};
use crate::murmur::murmur3_32;
use crate::romfs::Romfs;
use crate::tex::bntx::{self, Tex};

pub const JSON: &str = include_str!("../../../data/title_logo.json");
const BLOCK: usize = 16;
const MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];
const FCS_4: u8 = 0b10 << 6;
const SINGLE: u8 = 1 << 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub texture: String,
    pub files: Vec<String>,
    pub old_hash: u32,
    pub old_len: usize,
    pub runs: Vec<(usize, Vec<u8>)>,
}

fn unhex(s: &str) -> Res<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(E::Format("title logo: a run has an odd hex length"));
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| E::Format("title logo: a run is not hex"))).collect()
}

fn text<'a>(j: &'a Json, k: &str) -> Res<&'a str> {
    j.get(k).and_then(|v| v.as_str()).ok_or_else(|| E::Msg(format!("title logo: {k} is missing")))
}

fn num(j: &Json, k: &str) -> Res<i64> {
    j.get(k).and_then(|v| v.as_i64()).ok_or_else(|| E::Msg(format!("title logo: {k} is missing")))
}

pub fn patch() -> Res<Patch> {
    let j: Json = json::parse(JSON)?;
    let files: Vec<String> = j.get("files").and_then(|v| v.as_arr()).ok_or(E::Format("title logo: files are missing"))?.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    let mut runs: Vec<(usize, Vec<u8>)> = Vec::new();
    for r in j.get("runs").and_then(|v| v.as_arr()).ok_or(E::Format("title logo: runs are missing"))? {
        let a: &[Json] = r.as_arr().ok_or(E::Format("title logo: a run is not a pair"))?;
        let [at, hex] = a else {
            return Err(E::Format("title logo: a run is not a pair"));
        };
        let at: usize = at.as_i64().and_then(|v| usize::try_from(v).ok()).ok_or(E::Format("title logo: a run offset is not a number"))?;
        let b: Vec<u8> = unhex(hex.as_str().ok_or(E::Format("title logo: a run has no blocks"))?)?;
        if at % BLOCK != 0 || b.is_empty() || b.len() % BLOCK != 0 {
            return Err(E::Format("title logo: a run is not whole blocks"));
        }
        runs.push((at, b));
    }
    let old_len: usize = usize::try_from(num(&j, "old_len")?).map_err(|_| E::Format("title logo: old_len is negative"))?;
    if runs.iter().map(|r| r.1.len()).sum::<usize>() != old_len {
        return Err(E::Format("title logo: old_len does not match the runs"));
    }
    Ok(Patch { texture: text(&j, "texture")?.to_string(), files, old_hash: u32::try_from(num(&j, "old_hash")?).map_err(|_| E::Format("title logo: old_hash is out of range"))?, old_len, runs })
}

pub enum State {
    Vanilla,
    Done,
}

fn spans(p: &Patch, t: &Tex) -> Res<Vec<(usize, usize)>> {
    p.runs
        .iter()
        .map(|(at, b)| {
            let end: usize = at.checked_add(b.len()).ok_or(E::Format("title logo: a run overflows"))?;
            if end > t.size {
                return Err(E::Format("title logo: a run leaves the logo texture, the game files differ"));
            }
            Ok((t.off + at, t.off + end))
        })
        .collect()
}

pub fn state(raw: &[u8], p: &Patch) -> Res<State> {
    let ts: Vec<Tex> = bntx::parse(raw)?;
    let t: &Tex = bntx::find(&ts, &p.texture).ok_or_else(|| E::Msg(format!("title logo: {} is missing, the game files differ", p.texture)))?;
    let sp: Vec<(usize, usize)> = spans(p, t)?;
    let now: Vec<u8> = sp.iter().flat_map(|(a, b)| raw[*a..*b].iter().copied()).collect();
    if murmur3_32(&now, 0) == p.old_hash {
        return Ok(State::Vanilla);
    }
    if p.runs.iter().zip(&sp).all(|((_, b), (x, y))| &raw[*x..*y] == b.as_slice()) {
        return Ok(State::Done);
    }
    Err(E::Format("title logo: the logo texture is not the one the patch was made for, the game files differ"))
}

pub fn apply_raw(raw: &mut [u8], p: &Patch) -> Res<bool> {
    match state(raw, p)? {
        State::Done => Ok(false),
        State::Vanilla => {
            let ts: Vec<Tex> = bntx::parse(raw)?;
            let t: Tex = bntx::find(&ts, &p.texture).cloned().ok_or(E::Format("title logo: the logo texture is missing"))?;
            for ((_, b), (x, y)) in p.runs.iter().zip(spans(p, &t)?) {
                raw[x..y].copy_from_slice(b);
            }
            Ok(true)
        }
    }
}

pub fn sized(frame: &[u8], n: usize) -> Res<Vec<u8>> {
    let fhd: u8 = *frame.get(4).ok_or(E::Format("title logo: the compressed frame is empty"))?;
    if frame[..4] != MAGIC || fhd & 0xC3 != 0 || fhd & SINGLE != 0 {
        return Err(E::Format("title logo: the compressor wrote an unexpected frame header"));
    }
    let size: u32 = u32::try_from(n).map_err(|_| E::Format("title logo: the file is too large for one frame"))?;
    let mut out: Vec<u8> = Vec::with_capacity(frame.len() + 4);
    out.extend_from_slice(&frame[..4]);
    out.push(fhd | FCS_4);
    out.push(frame[5]);
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&frame[6..]);
    Ok(out)
}

pub fn compress(raw: &[u8]) -> Res<Vec<u8>> {
    let frame: Vec<u8> = ruzstd::encoding::compress_to_vec(raw, ruzstd::encoding::CompressionLevel::Fastest);
    sized(&frame, raw.len())
}

/// A block decompresses to at most this much.
const BLOCK_MAX: usize = 131072;

/// The length of a zstd frame header and its window byte, for a frame with no dictionary and no
/// checksum that is not a single segment (as the game's files are); None for any other frame.
fn plain_header(z: &[u8]) -> Option<(usize, u8)> {
    if z.len() < 6 || z[..4] != MAGIC {
        return None;
    }
    let fhd: u8 = z[4];
    if fhd & 0x07 != 0 || fhd & SINGLE != 0 {
        return None;
    }
    let fcs: usize = [0, 2, 4, 8][usize::from(fhd >> 6)];
    Some((6 + fcs, z[5]))
}

/// The length of any dictionary-less zstd frame header, and whether the frame ends in a checksum.
fn header_len(z: &[u8]) -> Option<(usize, bool)> {
    if z.len() < 5 || z[..4] != MAGIC || z[4] & 0x03 != 0 {
        return None;
    }
    let fhd: u8 = z[4];
    let single: bool = fhd & SINGLE != 0;
    let fcs: usize = match fhd >> 6 {
        0 if single => 1,
        0 => 0,
        1 => 2,
        2 => 4,
        _ => 8,
    };
    Some((5 + usize::from(!single) + fcs, fhd & 0x04 != 0))
}

/// Where each block of the frame `z` starts, from `from`, and where the last one ends.
fn block_starts(z: &[u8], from: usize) -> Option<Vec<usize>> {
    let mut at: usize = from;
    let mut out: Vec<usize> = Vec::new();
    loop {
        let h: u32 = u32::from_le_bytes([*z.get(at)?, *z.get(at + 1)?, *z.get(at + 2)?, 0]);
        let n: usize = (h >> 3) as usize;
        let len: usize = match (h >> 1) & 3 {
            0 | 2 => n,
            1 => 1,
            _ => return None,
        };
        out.push(at);
        at += 3 + len;
        if h & 1 == 1 {
            out.push(at);
            return Some(out);
        }
    }
}

/// The game's own compressed file `vz` with `raw` (the file with the logo drawn) in it: the game's
/// compressed blocks up to the block holding the first changed byte, then the rest compressed
/// here. Only the logo differs, late in each archive, so most of the file keeps the game's
/// compression instead of ruzstd's fastest level. The compressor here writes every offset in full
/// and every table anew, so its blocks lean on nothing the game's blocks left behind; the result
/// is decoded and compared before it is used. None when it cannot be spliced.
pub fn splice(vz: &[u8], raw: &[u8]) -> Res<Option<Vec<u8>>> {
    let Some((hlen, window)) = plain_header(vz) else {
        return Ok(None);
    };
    let Some(starts) = block_starts(vz, hlen) else {
        return Ok(None);
    };
    let mut z: crate::zs::Zs = crate::zs::Zs::new();
    let old: Vec<u8> = z.decompress(vz)?;
    let Some(first) = old.iter().zip(raw).position(|(a, b)| a != b) else {
        return Ok(None);
    };
    let k: usize = (first / BLOCK_MAX).min(starts.len().saturating_sub(2));
    if k == 0 || old.len() != raw.len() {
        return Ok(None);
    }
    // The first k blocks alone, as a frame of their own, tell how much they decompress to.
    let mut head: Vec<u8> = MAGIC.to_vec();
    head.extend_from_slice(&[0x00, window]);
    let at: usize = head.len() + starts[k - 1] - starts[0];
    head.extend_from_slice(&vz[starts[0]..starts[k]]);
    head[at] |= 1;
    let kept: usize = z.decompress(&head)?.len();
    if kept > first {
        return Ok(None);
    }
    let tail: Vec<u8> = ruzstd::encoding::compress_to_vec(&raw[kept..], ruzstd::encoding::CompressionLevel::Fastest);
    let Some((th, sum)) = header_len(&tail) else {
        return Ok(None);
    };
    let end: usize = if sum { tail.len().saturating_sub(4) } else { tail.len() };
    let mut out: Vec<u8> = vz[..starts[k]].to_vec();
    out.extend_from_slice(&tail[th..end]);
    if z.decompress(&out)? != raw {
        return Ok(None);
    }
    Ok(Some(out))
}

pub fn apply(mod_root: &P, romfs: &P) -> Res<Vec<String>> {
    let p: Patch = patch()?;
    let mut rf: Romfs = Romfs::open(romfs)?;
    let mut notes: Vec<String> = Vec::new();
    for f in &p.files {
        let out: PB = mod_root.join("romfs").join(f);
        let mut raw: Vec<u8> = if out.is_file() { rf.zs.decompress(&fs::read(&out)?)? } else { rf.read_zs(f)? };
        if !apply_raw(&mut raw, &p)? {
            continue;
        }
        if let Some(d) = out.parent() {
            fs::create_dir_all(d)?;
        }
        let z: Vec<u8> = match splice(&rf.read(f)?, &raw)? {
            Some(z) => z,
            None => compress(&raw)?,
        };
        let mb = |n: usize| n as f64 / 1_048_576.0;
        notes.push(format!("Title logo: the Randomizer line is drawn under the logo in {f} ({:.1} MB)", mb(z.len())));
        fs::write(&out, z)?;
    }
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_patch_parses() {
        let p: Patch = patch().unwrap();
        assert_eq!(p.texture, "Logo_USEU_00");
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.runs.len(), 225);
        assert_eq!(p.old_len, 1747 * BLOCK);
        assert!(p.runs.windows(2).all(|w| w[0].0 + w[0].1.len() < w[1].0));
    }

    #[test]
    fn a_sized_frame_carries_the_content_size_and_decodes() {
        let raw: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8 ^ (i >> 9) as u8).collect();
        let z: Vec<u8> = compress(&raw).unwrap();
        assert_eq!(z[4] & 0xC0, FCS_4);
        assert_eq!(u32::from_le_bytes([z[6], z[7], z[8], z[9]]) as usize, raw.len());
        assert!(z.len() < raw.len());
        let mut d = crate::zs::Zs::new();
        assert_eq!(d.decompress(&z).unwrap(), raw);
    }

    #[test]
    fn a_late_change_keeps_the_game_blocks_before_it() {
        let old: Vec<u8> = (0..700_000u32).map(|i| (i % 253) as u8 ^ (i >> 11) as u8).collect();
        let vz: Vec<u8> = compress(&old).unwrap();
        let mut raw: Vec<u8> = old.clone();
        for b in &mut raw[600_000..600_100] {
            *b ^= 0x5A;
        }
        let out: Vec<u8> = splice(&vz, &raw).unwrap().unwrap();
        let mut d = crate::zs::Zs::new();
        assert_eq!(d.decompress(&out).unwrap(), raw);
        let (h, _) = plain_header(&vz).unwrap();
        let starts: Vec<usize> = block_starts(&vz, h).unwrap();
        let k: usize = 600_000 / BLOCK_MAX;
        assert_eq!(out[..starts[k]], vz[..starts[k]], "the blocks before the change are the game's");
        assert!(splice(&vz, &old).unwrap().is_none(), "nothing changed, nothing to splice");
        let mut early: Vec<u8> = old.clone();
        early[10] ^= 1;
        assert!(splice(&vz, &early).unwrap().is_none(), "a change in the first block leaves nothing to keep");
    }

    #[test]
    fn both_real_archives_take_the_logo_once() {
        let root: PB = ootrr_game::test_romfs::path();
        if !root.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped, no RomFS dump at {}", root.display());
            return;
        }
        let p: Patch = patch().unwrap();
        let mut rf: Romfs = Romfs::open(&root).unwrap();
        for f in &p.files {
            let old: Vec<u8> = rf.read_zs(f).unwrap();
            let mut raw: Vec<u8> = old.clone();
            assert!(apply_raw(&mut raw, &p).unwrap(), "{f}");
            assert!(!apply_raw(&mut raw, &p).unwrap(), "{f}");
            assert_eq!(raw.len(), old.len());
            let changed: usize = raw.chunks(BLOCK).zip(old.chunks(BLOCK)).filter(|(a, b)| a != b).count();
            assert_eq!(changed, 1747, "{f}");
            let ts: Vec<Tex> = bntx::parse(&raw).unwrap();
            let t: &Tex = bntx::find(&ts, &p.texture).unwrap();
            let outside: bool = raw.chunks(BLOCK).zip(old.chunks(BLOCK)).enumerate().any(|(k, (a, b))| a != b && (k * BLOCK < t.off || k * BLOCK >= t.off + t.size));
            assert!(!outside, "{f}");
        }
    }
}
