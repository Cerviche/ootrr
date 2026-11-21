use std::collections::{BTreeMap as BM, BTreeSet as BS};

use crate::byml::Byml;
use crate::dungeon_grant::{Grants, Marks};
use crate::error::{Error as E, Res};
use crate::slice::edit::Edited;
use crate::slice::skull_edit::Drop;

pub struct State<'a> {
    pub silver: &'a BM<u64, &'static str>,
    pub silver_used: BS<&'static str>,
    pub silver_hit: BS<u64>,
    pub grants: &'a Grants,
    pub marks: Marks,
    pub skulls: &'a [Drop],
    pub skull_hit: BS<u64>,
    pub qol: crate::qol::Opts,
    pub qol_hits: [usize; 3],
}

impl<'a> State<'a> {
    pub fn new(silver: &'a BM<u64, &'static str>, grants: &'a Grants, skulls: &'a [Drop]) -> State<'a> {
        State { silver, silver_used: BS::new(), silver_hit: BS::new(), grants, marks: Marks::default(), skulls, skull_hit: BS::new(), qol: crate::qol::Opts::default(), qol_hits: [0; 3] }
    }
}

pub type Pass = fn(&str, &[u8], Option<Edited>, &mut State) -> Res<Option<Edited>>;

pub const PASSES: [(&str, Pass); 4] = [("silver rupees", silver), ("dungeon item grants", grants), ("skulltula drops", skulls), ("qol map edits", qol_maps)];

pub type Check = fn(&State) -> Res<()>;

pub const CHECKS: [(&str, Check); 4] = [("every skulltula drop was placed", skulls_found), ("every dungeon item grant was marked", grants_marked), ("every silver rupee was placed", silver_found), ("every qol map edit was made", qol_found)];

pub fn run(file: &str, raw: &[u8], e: Option<Edited>, st: &mut State) -> Res<Option<Edited>> {
    let mut e: Option<Edited> = e;
    for (_, p) in PASSES {
        e = p(file, raw, e, st)?;
    }
    Ok(e)
}

pub fn finish(st: &State) -> Res<()> {
    for (_, c) in CHECKS {
        c(st)?;
    }
    Ok(())
}

fn silver(file: &str, raw: &[u8], e: Option<Edited>, st: &mut State) -> Res<Option<Edited>> {
    crate::slice::silver::apply(file, raw, e, st.silver, &mut st.silver_used, &mut st.silver_hit)
}

fn grants(file: &str, raw: &[u8], e: Option<Edited>, st: &mut State) -> Res<Option<Edited>> {
    crate::slice::dungeon_items::apply(file, raw, e, st.grants, &mut st.marks)
}

fn skulls(file: &str, raw: &[u8], e: Option<Edited>, st: &mut State) -> Res<Option<Edited>> {
    crate::slice::skull_edit::apply(file, raw, e, st.skulls, &mut st.skull_hit)
}

fn skulls_found(st: &State) -> Res<()> {
    crate::slice::skull_edit::all_found(st.skulls, &st.skull_hit)
}

fn grants_marked(st: &State) -> Res<()> {
    crate::dungeon_grant::all_marked(st.grants, &st.marks)
}

fn silver_found(st: &State) -> Res<()> {
    crate::slice::silver::all_found(st.silver, &st.silver_hit)
}

fn has(raw: &[u8], mark: &[u8]) -> bool {
    raw.windows(mark.len()).any(|w| w == mark)
}

fn qol_maps(file: &str, raw: &[u8], e: Option<Edited>, st: &mut State) -> Res<Option<Edited>> {
    let o: crate::qol::Opts = st.qol;
    if !o.maps() {
        return Ok(e);
    }
    let night: bool = o.day_skulls && has(raw, crate::qol::night_skulls::MARK);
    let crow: bool = o.free_scarecrow && has(raw, crate::qol::scarecrow::MARK);
    let tower: bool = o.skip_escape && file == crate::qol::tower_escape::ANCHOR_FILE;
    if !(night || crow || tower) {
        return Ok(e);
    }
    let mut y: Byml = match &e {
        Some(x) => Byml::parse(&x.bytes)?,
        None => Byml::parse(raw)?,
    };
    let mut n: usize = 0;
    if night {
        let k: usize = crate::qol::night_skulls::patch(&mut y);
        st.qol_hits[0] += k;
        n += k;
    }
    if crow {
        let k: usize = crate::qol::scarecrow::patch(&mut y);
        st.qol_hits[1] += k;
        n += k;
    }
    if tower {
        let k: usize = crate::qol::tower_escape::patch_anchor(&mut y);
        st.qol_hits[2] += k;
        n += k;
    }
    if n == 0 {
        return Ok(e);
    }
    let bytes: Vec<u8> = y.write()?;
    Ok(Some(match e {
        Some(mut x) => {
            x.bytes = bytes;
            x
        }
        None => Edited {
            file: file.to_string(),
            old_len: raw.len(),
            bytes,
            hits: Vec::new(),
            used: Vec::new(),
            hearts: Vec::new(),
            wraps: Vec::new(),
            touched: Vec::new(),
        },
    }))
}

fn qol_found(st: &State) -> Res<()> {
    let o: crate::qol::Opts = st.qol;
    let want: [(bool, usize, &str); 3] = [
        (o.day_skulls, crate::qol::night_skulls::COUNT, "night Gold Skulltulas"),
        (o.free_scarecrow, crate::qol::scarecrow::COUNT, "scarecrow spots"),
        (o.skip_escape, 1, "tower preload anchors"),
    ];
    for (i, (on, n, what)) in want.iter().enumerate() {
        if *on && st.qol_hits[i] != *n {
            return Err(E::Msg(format!("qol: expected {n} {what} in the maps, found {}, the game files differ", st.qol_hits[i])));
        }
    }
    Ok(())
}
