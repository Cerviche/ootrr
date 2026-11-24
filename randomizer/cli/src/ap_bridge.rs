use std::io::{self, BufRead, Write};

use ootrr_core::error::{Error, Res};
use ootrr_core::json::{self, Json};
use ootrr_core::murmur::murmur3_32;
use ootrr_mem::game::Game;
use ootrr_mem::gdm;
use ootrr_mem::snap::Snap;

pub const PROTOCOL: i64 = 1;
pub const WRITABLE: &str = "OOTRR.AP.";
pub const MAX_LINE: usize = 4 << 20;
pub const MAX_NAMES: usize = 20_000;

struct Bridge {
    g: Option<Game>,
}

fn hash(name: &str) -> u32 {
    murmur3_32(name.as_bytes(), 0)
}

fn names<'a>(j: &'a Json, k: &str) -> Res<Vec<&'a str>> {
    let Some(a) = j.get(k) else { return Ok(Vec::new()) };
    let a: &[Json] = a.as_arr().ok_or_else(|| Error::Msg(format!("{k} must be a list")))?;
    if a.len() > MAX_NAMES {
        return Err(Error::Msg(format!("{k} has too many entries")));
    }
    a.iter().map(|x| x.as_str().ok_or_else(|| Error::Msg(format!("{k} must hold text")))).collect()
}

impl Bridge {
    fn game(&mut self) -> Res<&Game> {
        if self.g.as_ref().is_some_and(|g| !g.alive()) {
            self.g = None;
        }
        if self.g.is_none() {
            self.g = Some(Game::attach()?);
        }
        self.g.as_ref().ok_or_else(|| Error::Msg("the game is not running".to_string()))
    }

    fn read(&mut self, j: &Json) -> Res<Json> {
        let bools: Vec<&str> = names(j, "bools")?;
        let keys: Vec<&str> = names(j, "keys")?;
        let s: Snap = {
            let g: &Game = self.game()?;
            match gdm::snap(g) {
                Ok(s) => s,
                Err(e) => {
                    if !g.alive() {
                        self.g = None;
                    }
                    return Err(e);
                }
            }
        };
        let mut b: Vec<(String, Json)> = Vec::with_capacity(bools.len());
        for n in bools {
            let h: u32 = hash(n);
            let v: Json = match s.rows.binary_search_by_key(&h, |r| r.0) {
                Ok(i) => Json::Bool(s.rows[i].1 & 1 != 0),
                Err(_) => Json::Null,
            };
            b.push((n.to_string(), v));
        }
        let mut k: Vec<(String, Json)> = Vec::with_capacity(keys.len());
        for n in keys {
            let v: Json = match n.parse::<u64>() {
                Ok(x) => Json::Bool(s.keys.binary_search(&x).is_ok()),
                Err(_) => Json::Null,
            };
            k.push((n.to_string(), v));
        }
        Ok(Json::obj().with("bools", Json::Obj(b)).with("keys", Json::Obj(k)))
    }

    fn write(&mut self, j: &Json) -> Res<Json> {
        let name: &str = j.get("name").and_then(|v| v.as_str()).ok_or(Error::Format("write needs a name"))?;
        let value: bool = j.get("value").and_then(|v| v.as_bool()).ok_or(Error::Format("write needs a true or false value"))?;
        if !name.starts_with(WRITABLE) {
            return Err(Error::Msg(format!("only flags that start with {WRITABLE} can be written")));
        }
        let g: &Game = self.game()?;
        let was: bool = gdm::set_bool(g, hash(name), value)?;
        Ok(Json::obj().with("was", Json::Bool(was)))
    }

    fn answer(&mut self, line: &str) -> Json {
        let r: Res<Json> = (|| {
            let j: Json = json::parse(line)?;
            match j.get("op").and_then(|v| v.as_str()) {
                Some("hello") => Ok(Json::obj().with("protocol", Json::Int(PROTOCOL)).with("version", Json::Str(env!("CARGO_PKG_VERSION").to_string()))),
                Some("read") => self.read(&j),
                Some("write") => self.write(&j),
                Some(o) => Err(Error::Msg(format!("unknown op {o}"))),
                None => Err(Error::Format("a request needs an op")),
            }
        })();
        let id: Json = json::parse(line).ok().and_then(|j| j.get("id").cloned()).unwrap_or(Json::Null);
        match r {
            Ok(v) => v.with("ok", Json::Bool(true)).with("id", id),
            Err(e) => Json::obj().with("ok", Json::Bool(false)).with("error", Json::Str(e.to_string())).with("attached", Json::Bool(self.g.is_some())).with("id", id),
        }
    }
}

pub fn run(_a: &[String]) -> Res<()> {
    let mut b: Bridge = Bridge { g: None };
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line: String = line?;
        let t: &str = line.trim();
        if t.is_empty() {
            continue;
        }
        let reply: Json = if t.len() > MAX_LINE { Json::obj().with("ok", Json::Bool(false)).with("error", Json::Str("the request is too long".to_string())) } else { b.answer(t) };
        writeln!(out, "{reply}")?;
        out.flush()?;
    }
    Ok(())
}
