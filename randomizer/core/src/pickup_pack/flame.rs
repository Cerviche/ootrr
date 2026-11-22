//! The Fire Arrow flame on every seed pickup.
//!
//! Seed pickups name the Fire Arrow pickup's effect user (actor::GLINT_USER). Its Wait key is a
//! container that plays the magic aura, the metal shine, the burning core, a light and a far glow
//! (Wait_00_00..Wait_00_04) while Rotate_Obj plays, under the same trigger as the Gold Skulltula
//! token's and the Heart Piece's Wait. All but the shine sit on the bone Effect_0, which only 16
//! of the 913 Item_/Obj_ models have (the arrows, bottles, Deku Sticks, Hookshot, Bombchu); Root
//! is on 912 of them. So the seed rebinds those effects to Root, larger, in the game's effect
//! table (ELink2/elink2.Product.110.belnk.zs). The vanilla Fire Arrow pickup, which no seed
//! places on the ground (slice::pickups::NOT_ON_GROUND), then burns at its model's origin
//! instead of its tip.

use crate::difficulty::elink::{self, Link, Param, TEXT};
use crate::error::{Error as E, Res};
use crate::romfs::Romfs;
use crate::staging::Out;

pub const USER: &str = "Obj_Arrow_Fire";
pub const FROM_BONE: &str = "Effect_0";
pub const TO_BONE: &str = "Root";
/// The keys rebound: the parts of the Wait container.
pub const KEYS: &str = "Wait_";
pub const BONE: u32 = 12;
pub const SCALE: u32 = 13;
/// How much larger the flame burns than on the arrow: one of the effect table's stored values.
pub const SIZE: f32 = 2.0;

/// Rebinds the Wait effects of [`USER`] from [`FROM_BONE`] to [`TO_BONE`] at [`SIZE`]. Returns
/// the new table and how many calls moved.
pub fn rebind(raw: &[u8]) -> Res<(Vec<u8>, usize)> {
    let mut l: Link = Link::parse(raw)?;
    let u: usize = l.user(USER).ok_or_else(|| E::Msg(format!("elink: no effect user {USER}")))?;
    let from: u32 = (TEXT << 24) | l.find_name(FROM_BONE).ok_or_else(|| E::Msg(format!("elink: no bone name {FROM_BONE}")))? as u32;
    let to: u32 = (TEXT << 24) | l.add_name(TO_BONE)? as u32;
    let size: Option<u32> = l.direct_f32(SIZE);
    let mut moved: usize = 0;
    for c in 0..l.users[u].calls.len() {
        let call: &elink::Call = &l.users[u].calls[c];
        if call.asset() == elink::CONTAINER || !l.key(u, c).is_some_and(|k| k.starts_with(KEYS)) {
            continue;
        }
        let Some(p) = l.param_at(call.param_pos()) else {
            return Err(E::Msg(format!("elink: {USER} call {c} points at no parameters")));
        };
        if p.get(BONE) != Some(from) {
            continue;
        }
        let mut pairs: Vec<(u32, u32)> = p.pairs().into_iter().filter(|(b, _)| *b != BONE && (*b != SCALE || size.is_none())).collect();
        pairs.push((BONE, to));
        if let Some(s) = size {
            pairs.push((SCALE, s));
        }
        let pos: u64 = l.add_param(Param::from_pairs(&pairs)?);
        l.users[u].calls[c].set_param_pos(pos);
        moved += 1;
    }
    if moved == 0 {
        return Err(E::Msg(format!("elink: {USER} has no Wait effect on {FROM_BONE} to move")));
    }
    Ok((l.write()?, moved))
}

/// Stages the rebind on top of any earlier edit of the effect table in `late` (the difficulty
/// options edit the same file).
pub fn stage(rf: &mut Romfs, late: &mut Vec<Out>, work: &std::path::Path) -> Res<()> {
    crate::story_skip::stage_late_file(rf, late, work, elink::FILE, |raw| Ok(rebind(raw)?.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Option<Vec<u8>> {
        let p: std::path::PathBuf = std::env::var_os("OOTRR_ROMFS").map(std::path::PathBuf::from)?;
        let mut rf: Romfs = Romfs::open(&p).ok()?;
        rf.read_zs(elink::FILE).ok()
    }

    #[test]
    fn the_fire_arrow_wait_flame_moves_to_root_and_nothing_else_changes() {
        let Some(raw) = game() else {
            eprintln!("skipped: set OOTRR_ROMFS");
            return;
        };
        let was: Link = Link::parse(&raw).unwrap();
        let (out, moved): (Vec<u8>, usize) = rebind(&raw).unwrap();
        assert_eq!(moved, 4, "aura, burning core, light and far glow");
        let now: Link = Link::parse(&out).unwrap();
        assert_eq!(now.write().unwrap(), out);
        let u: usize = now.user(USER).unwrap();
        let bone = |l: &Link, key: &str| -> Option<String> {
            let c: usize = l.find_key(u, key)?;
            l.param_at(l.users[u].calls[c].param_pos())?.get(BONE).and_then(|v| l.text(v)).map(|s| s.to_string())
        };
        for k in ["Wait_00_00", "Wait_00_02", "Wait_00_03", "Wait_00_04"] {
            assert_eq!(bone(&now, k).as_deref(), Some(TO_BONE), "{k}");
            let c: usize = now.find_key(u, k).unwrap();
            let p: &Param = now.param_at(now.users[u].calls[c].param_pos()).unwrap();
            assert_eq!(p.get(SCALE).and_then(|i| now.direct(i as usize)).map(f32::from_bits), Some(SIZE), "{k} is scaled");
            let o: &Param = was.param_at(was.users[u].calls[c].param_pos()).unwrap();
            assert_eq!(p.get(1), o.get(1), "{k} keeps its effect");
        }
        assert_eq!(bone(&now, "Wait_00_01").as_deref(), Some("Skl_Root"), "the metal shine stays");
        assert_eq!(bone(&now, "GetDemo_00").as_deref(), Some(FROM_BONE), "the get demo stays");
        for (i, user) in now.users.iter().enumerate() {
            if i != u {
                assert_eq!(user, &was.users[i]);
            }
        }
        assert_eq!(&now.params[..was.params.len()], &was.params[..], "old parameters stay where calls point at them");
        let size: Option<u32> = now.direct_f32(SIZE);
        assert!(size.is_some(), "the effect table stores the flame size");
    }
}
