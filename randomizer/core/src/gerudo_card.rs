use std::collections::BTreeMap as BM;

use crate::ainb::build::{Spec, QUERY};
use crate::ainb::Ainb;
use crate::error::{Error, Res};
use crate::qol::{edit_plain, Have};
use crate::romfs::Romfs;
use crate::staging::Out;

pub const CARD: &str = "IsGet.Item_Token_Gerudo";
pub const GUARD: &str = "ActorLogicNpcSecurityGuard";
pub const ENABLE: &str = "EnableSecurityGuard";
pub const GET_BOOL: &str = "HoldGameDataGetBoolSimple";
pub const AND: &str = "OperateAND";
pub const OR: &str = "OperateOR";
pub const MULTI: &str = "BoolMulti";
pub const BOOL: usize = 1;
pub const BOOL_IN: usize = 16;
pub const MULTI_COUNT: usize = 0x1c;
pub const TRUE_OUT: i16 = 1;
pub const FALSE_OUT: u16 = 0;

pub const MODULES: [&str; 7] = [
    "Logic/GerudoFortress_Common_178b.logic.module.ainb",
    "Logic/GerudoFortress_Common_1d80.logic.module.ainb",
    "Logic/GerudoFortress_Level_Exterior_01_a401.logic.module.ainb",
    "Logic/GerudoFortress_Level_Room_02_5d06.logic.module.ainb",
    "Logic/GerudoFortress_Level_Room_05_ee1c.logic.module.ainb",
    "Logic/GerudoFortress_Level_Room_06_ade9.logic.module.ainb",
    "Logic/MiniGame_HorsebackArchery_8380.logic.module.ainb",
];

pub const ALWAYS_OFF: [&str; 6] = [
    "Logic/GerudoFortress_Common_58a8.logic.module.ainb",
    "Logic/GerudoFortress_Common_Level_CarpenterSet_01_e139.logic.module.ainb",
    "Logic/GerudoFortress_Common_Level_CarpenterSet_02_9477.logic.module.ainb",
    "Logic/GerudoFortress_Common_Level_CarpenterSet_03_c7f2.logic.module.ainb",
    "Logic/GerudoFortress_Common_Level_CarpenterSet_04_e9a4.logic.module.ainb",
    "Logic/GerudoFortress_Level_Room_06_1561.logic.module.ainb",
];

fn i16_at(b: &mut [u8], o: usize, v: i16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

fn range(a: &Ainb, i: usize) -> (usize, usize) {
    let e = &a.els[i];
    (u16::from_le_bytes([e[0x24], e[0x25]]) as usize, u16::from_le_bytes([e[0x26], e[0x27]]) as usize)
}

fn set_range(a: &mut Ainb, i: usize, base: usize, n: usize) -> Res<()> {
    let b: u16 = u16::try_from(base).map_err(|_| Error::Format("gerudo card: query list too long"))?;
    let c: u16 = u16::try_from(n).map_err(|_| Error::Format("gerudo card: query list too long"))?;
    a.els[i][0x24..0x26].copy_from_slice(&b.to_le_bytes());
    a.els[i][0x26..0x28].copy_from_slice(&c.to_le_bytes());
    Ok(())
}

fn qpos(a: &Ainb, i: usize) -> Res<u16> {
    let p: usize = a.query_pos(i).ok_or(Error::Format("gerudo card: a source is not a query node"))?;
    u16::try_from(p).map_err(|_| Error::Format("gerudo card: too many query nodes"))
}

pub fn queries(a: &Ainb, i: usize) -> Vec<u16> {
    let (s, n) = range(a, i);
    (s..s + n).map(|j| u16::from_le_bytes([a.qids[4 * j], a.qids[4 * j + 1]])).collect()
}

fn give_queries(a: &mut Ainb, i: usize, want: &[u16]) -> Res<()> {
    let base: usize = a.qids.len() / 4;
    for q in want {
        a.qids.extend_from_slice(&q.to_le_bytes());
        a.qids.extend_from_slice(&0u16.to_le_bytes());
    }
    set_range(a, i, base, want.len())
}

pub fn multi_node(a: &mut Ainb, name: &str, seed: &str, srcs: &[(usize, u16)]) -> Res<usize> {
    let first: usize = srcs.first().ok_or(Error::Format("gerudo card: a gate needs a source"))?.0;
    let mut s: Spec = Spec::node(name, seed).linked(BOOL, MULTI, first, 0).constant(BOOL, "UpdateOnce", vec![0; 4]).out(BOOL, "False").out(BOOL, "True");
    s.role = QUERY;
    let id: usize = a.add_node(&s)?;
    let start: usize = a.multi.len() / 8;
    for (el, out) in srcs {
        let e: u16 = u16::try_from(*el).map_err(|_| Error::Format("gerudo card: element index too large"))?;
        a.multi.extend_from_slice(&e.to_le_bytes());
        a.multi.extend_from_slice(&out.to_le_bytes());
        a.multi.extend_from_slice(&[0; 4]);
    }
    let g: usize = a.find_in(id, BOOL, MULTI).ok_or(Error::Format("gerudo card: the gate has no multi input"))?;
    let at: usize = g * BOOL_IN;
    let src: i16 = i16::try_from(-100 - start as i64).map_err(|_| Error::Format("gerudo card: multi table too long"))?;
    let n: i16 = i16::try_from(srcs.len()).map_err(|_| Error::Format("gerudo card: too many gate sources"))?;
    i16_at(&mut a.ins[BOOL], at + 4, src);
    i16_at(&mut a.ins[BOOL], at + 6, n);
    a.els[id][MULTI_COUNT..MULTI_COUNT + 4].copy_from_slice(&(srcs.len() as u32).to_le_bytes());
    let mut want: Vec<u16> = Vec::new();
    for (el, _) in srcs {
        let q: u16 = qpos(a, *el)?;
        if !want.contains(&q) {
            want.push(q);
        }
    }
    give_queries(a, id, &want)?;
    Ok(id)
}

pub fn card_node(a: &mut Ainb, seed: &str) -> Res<usize> {
    let name: Vec<u8> = a.pool.add(CARD).to_le_bytes().to_vec();
    let mut s: Spec = Spec::node(GET_BOOL, seed).constant(0, "GameDataArrayIndex", (-1i32).to_le_bytes().to_vec()).constant(3, "GameDataNameBool", name).out(BOOL, "Bool");
    s.role = QUERY;
    a.add_node(&s)
}

pub fn guards(a: &Ainb) -> Vec<(usize, usize)> {
    a.find_els(GUARD).into_iter().filter_map(|i| a.find_in(i, BOOL, ENABLE).map(|g| (i, g))).collect()
}

pub fn patch(a: &mut Ainb, tag: &str) -> Res<usize> {
    let linked: Vec<(usize, usize, i16, i16)> = guards(a)
        .into_iter()
        .filter_map(|(i, g)| a.input(BOOL, g).map(|x| (i, g, x.src, x.plug)))
        .filter(|(_, _, src, _)| *src >= 0)
        .collect();
    if guards(a).iter().any(|(_, g)| a.input(BOOL, *g).is_some_and(|x| x.src <= -100)) {
        return Err(Error::Format("gerudo card: a guard reads a multi input, the game files differ"));
    }
    if linked.is_empty() {
        return Err(Error::Format("gerudo card: no guard's security is linked, the game files differ"));
    }
    let card: usize = card_node(a, &format!("ootrr gerudo card {tag} read"))?;
    let lacks: usize = multi_node(a, OR, &format!("ootrr gerudo card {tag} lacks"), &[(card, 0)])?;
    let mut gates: BM<(i16, i16), usize> = BM::new();
    let enable: u32 = a.pool.add(ENABLE);
    for (i, g, src, plug) in &linked {
        let gate: usize = match gates.get(&(*src, *plug)) {
            Some(x) => *x,
            None => {
                let from: u16 = u16::try_from(*plug).map_err(|_| Error::Format("gerudo card: bad output index"))?;
                let n: usize = gates.len();
                let x: usize = multi_node(a, AND, &format!("ootrr gerudo card {tag} gate {n}"), &[(*src as usize, from), (lacks, FALSE_OUT)])?;
                gates.insert((*src, *plug), x);
                x
            }
        };
        let at: usize = g * BOOL_IN;
        let to: i16 = i16::try_from(gate).map_err(|_| Error::Format("gerudo card: element index too large"))?;
        i16_at(&mut a.ins[BOOL], at + 4, to);
        i16_at(&mut a.ins[BOOL], at + 6, TRUE_OUT);
        let p = &mut a.params[*i];
        let (n0, s0): (usize, usize) = (p.pl[0][0] as usize, p.pl[0][1] as usize);
        let hit: usize = (s0..s0 + n0)
            .find(|&k| p.plugs[k].a == *src as u32 && p.plugs[k].b == enable)
            .ok_or(Error::Format("gerudo card: a guard has no plug for its security input"))?;
        p.plugs[hit].a = gate as u32;
        let mut want: Vec<u16> = queries(a, *i);
        let q: u16 = qpos(a, gate)?;
        if !want.contains(&q) {
            want.push(q);
            give_queries(a, *i, &want)?;
        }
    }
    Ok(linked.len())
}

pub fn patch_bytes(raw: &[u8], tag: &str) -> Res<Vec<u8>> {
    let mut a: Ainb = Ainb::parse(raw)?;
    patch(&mut a, tag)?;
    let out: Vec<u8> = a.write()?;
    let back: Ainb = Ainb::parse(&out)?;
    if back.write()? != out || !crate::ainb::audit(&back).is_empty() {
        return Err(Error::Format("gerudo card: the patched module does not read back"));
    }
    Ok(out)
}

pub fn stage(rf: &Romfs, have: Have) -> Res<Vec<Out>> {
    MODULES.iter().map(|m| edit_plain(rf, have, m, |raw| patch_bytes(raw, m))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf as PB;

    fn real() -> Option<PB> {
        let p: PB = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Some(p)
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    fn src_of(a: &Ainb, g: usize) -> (i16, i16) {
        let x = a.input(BOOL, g).unwrap();
        (x.src, x.plug)
    }

    fn multi_of(a: &Ainb, i: usize) -> Vec<(u16, u16)> {
        let g: usize = a.find_in(i, BOOL, MULTI).unwrap();
        let x = a.input(BOOL, g).unwrap();
        let start: usize = (-100 - i32::from(x.src)) as usize;
        (start..start + x.plug as usize).map(|j| crate::door_open::multi_entry(a, j).unwrap()).collect()
    }

    #[test]
    fn every_multi_input_node_in_the_game_counts_its_entries_at_0x1c() {
        let Some(root) = real() else {
            return;
        };
        let rf: Romfs = Romfs::open(&root).unwrap();
        for m in MODULES.iter().chain(ALWAYS_OFF.iter()).chain(["Logic/GerudoFortress_Common_35d7.logic.module.ainb"].iter()) {
            let a: Ainb = Ainb::parse(&rf.read(m).unwrap()).unwrap();
            for i in a.find_els(AND).into_iter().chain(a.find_els(OR)) {
                let x = a.input(BOOL, a.find_in(i, BOOL, MULTI).unwrap()).unwrap();
                let want: u8 = if x.src <= -100 { x.plug as u8 } else { 0 };
                assert_eq!(a.els[i][MULTI_COUNT], want, "{m} element {i}");
            }
        }
    }

    #[test]
    fn every_fortress_guard_is_switched_off_by_the_card_and_keeps_its_quest_check() {
        let Some(root) = real() else {
            return;
        };
        let rf: Romfs = Romfs::open(&root).unwrap();
        let outs: Vec<Out> = stage(&rf, &|_: &str| None).unwrap();
        assert_eq!(outs.len(), MODULES.len());
        let mut total: usize = 0;
        for (m, o) in MODULES.iter().zip(&outs) {
            let old: Ainb = Ainb::parse(&rf.read(m).unwrap()).unwrap();
            let new: Ainb = Ainb::parse(&o.z).unwrap();
            assert!(crate::ainb::audit(&new).is_empty(), "{m}");
            let card: Vec<usize> = new.find_els(GET_BOOL).into_iter().filter(|&i| new.in_str(i, "GameDataNameBool") == Some(CARD)).collect();
            assert_eq!(card.len(), 1 + old.find_els(GET_BOOL).into_iter().filter(|&i| old.in_str(i, "GameDataNameBool") == Some(CARD)).count(), "{m}");
            let card: usize = *card.last().unwrap();
            let lacks: Vec<usize> = new.find_els(OR).into_iter().filter(|&i| i >= old.els.len() && multi_of(&new, i) == vec![(card as u16, 0)]).collect();
            assert_eq!(lacks.len(), 1, "{m}");
            for ((i, g), (j, h)) in guards(&old).into_iter().zip(guards(&new)) {
                assert_eq!(i, j, "{m}");
                let (s, p) = src_of(&old, g);
                if s < 0 {
                    assert_eq!(src_of(&new, h), (s, p), "{m}");
                    continue;
                }
                let (gate, out) = src_of(&new, h);
                assert_eq!(out, TRUE_OUT, "{m}");
                assert_eq!(new.el_name(gate as usize), AND, "{m}");
                assert_eq!(multi_of(&new, gate as usize), vec![(s as u16, p as u16), (lacks[0] as u16, FALSE_OUT)], "{m}");
                assert!(queries(&new, i).contains(&(gate as u16)), "{m}");
                assert_eq!(new.els[gate as usize][MULTI_COUNT], 2, "{m}");
                total += 1;
            }
            for i in 0..old.els.len() {
                if old.el_name(i) == GUARD {
                    continue;
                }
                for k in 0..6 {
                    for (g, h) in old.input_ids(i, k).into_iter().zip(new.input_ids(i, k)) {
                        assert_eq!(old.input(k, g).map(|x| (x.src, x.plug)), new.input(k, h).map(|x| (x.src, x.plug)), "{m} element {i}");
                    }
                }
            }
        }
        assert_eq!(total, 17);
        for m in ALWAYS_OFF {
            let a: Ainb = Ainb::parse(&rf.read(m).unwrap()).unwrap();
            let gs: Vec<(usize, usize)> = guards(&a);
            assert!(!gs.is_empty(), "{m}");
            for (_, g) in gs {
                let x = a.input(BOOL, g).unwrap();
                assert!(x.src < 0 && x.val == vec![0; 4], "{m}");
            }
            assert!(patch(&mut a.clone(), m).is_err(), "{m}");
        }
    }
}
