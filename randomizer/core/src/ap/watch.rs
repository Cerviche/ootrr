use crate::ainb::build::Spec;
use crate::ainb::graft::{root, set_root, SIMULTANEOUS};
use crate::ainb::Ainb;
use crate::error::{Error, Res};

use super::mailbox::{DISPATCH, PENDING};

pub const MAIN: &str = "Sequence/MainUpdate.module.ainb";
pub const SELECTOR: u16 = 7;
const S32: usize = 0;
const BOOL: usize = 1;
const STR: usize = 3;

fn s32(v: i32) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

fn b(v: bool) -> Vec<u8> {
    (v as u32).to_le_bytes().to_vec()
}

fn text(a: &mut Ainb, s: &str) -> Vec<u8> {
    a.pool.add(s).to_le_bytes().to_vec()
}

fn simultaneous<'a>(seed: &'a str) -> Spec<'a> {
    Spec::composite(SIMULTANEOUS, seed).prop(S32, "EndPolicy", s32(0)).prop(S32, "ResultPolicy", s32(0))
}

fn watcher(a: &mut Ainb, recv: &str, event: &str, key: &str) -> Res<usize> {
    let seed = |n: &str| format!("ootrr-ap#{key}#{n}");
    let (s1, s2, s3, s4, s5, s6) = (seed("query"), seed("request"), seed("clear"), seed("idle"), seed("both"), seed("select"));
    let name: Vec<u8> = text(a, recv);
    let q: usize = a.add_node(
        &Spec::query("SeqQueryGetNonStructGameDataBool", &s1)
            .idle(S32, "GameDataIndex", s32(-1))
            .constant(BOOL, "IsDirect", b(true))
            .constant(STR, "GameDataFullName", name.clone())
            .out(BOOL, "Value"),
    )?;
    let ev: Vec<u8> = text(a, event);
    let req: usize = a.add_node(
        &Spec::node("SeqOneShotRequestEvent", &s2)
            .prop(BOOL, "ExecuteEveryFrame", b(false))
            .idle(BOOL, "IsBancParamValue", b(false))
            .constant(STR, "EventName", ev),
    )?;
    let clear: usize = a.add_node(
        &Spec::node("SeqOneShotSetNonStructGameDataBool", &s3)
            .prop(BOOL, "ExecuteEveryFrame", b(false))
            .idle(S32, "GameDataIndex", s32(-1))
            .constant(BOOL, "Value", b(false))
            .constant(STR, "GameDataFullName", name),
    )?;
    let idle: usize = a.add_node(&Spec::node("SeqExecuteGenericDummyAction", &s4))?;
    let both: usize = a.add_node(&simultaneous(&s5).kid(req, "").kid(clear, ""))?;
    let mut sel: Spec = Spec::composite(SELECTOR, &s6)
        .prop(S32, "CalculateTiming", s32(0))
        .prop(BOOL, "ChildFrameSync", b(false))
        .prop(BOOL, "InputValue", b(false))
        .prop(BOOL, "IsNoSelectWhenChildBusy", b(false))
        .linked(BOOL, "Input", q, 0)
        .kid(both, "True")
        .kid(idle, "False");
    sel.link_extra = 8;
    a.add_node(&sel)
}

pub fn add_dispatch(a: &mut Ainb) -> Res<usize> {
    let old: usize = root(a).ok_or(Error::Format("watchers: MainUpdate has no root"))?;
    let w: usize = watcher(a, PENDING, DISPATCH, "dispatch")?;
    let top: Spec = simultaneous("ootrr-ap#root").kid(old, "").kid(w, "");
    let new: usize = a.add_node(&top)?;
    set_root(a, new)?;
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ainb::audit;
    use crate::romfs::Romfs;

    #[test]
    fn the_real_main_update_keeps_its_old_root_under_a_new_one_and_reads_back() {
        let p = ootrr_game::test_romfs::path();
        if !p.join("Pack").join("ZsDic.pack.zs").is_file() {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            return;
        }
        let rf: Romfs = Romfs::open(&p).unwrap();
        let raw: Vec<u8> = rf.read(MAIN).unwrap();
        let before: Ainb = Ainb::parse(&raw).unwrap();
        let mut a: Ainb = before.clone();
        assert_eq!(add_dispatch(&mut a).unwrap(), 1);
        assert!(audit(&a).is_empty(), "{:?}", audit(&a));
        let out: Vec<u8> = a.write().unwrap();
        let back: Ainb = Ainb::parse(&out).unwrap();
        assert!(audit(&back).is_empty());
        assert_eq!(back.els.len(), before.els.len() + 6 + 1);
        let top: usize = root(&back).unwrap();
        assert_eq!(top, back.els.len() - 1);
        assert_eq!(back.parent_of(0).unwrap().0, top);
        let shown: String = crate::ainb::show::show(&back);
        assert!(shown.contains(&format!("in str GameDataFullName = \"{PENDING}\"")));
        assert!(shown.contains(&format!("in str EventName = \"{DISPATCH}\"")));
        assert_eq!(shown.matches("SeqOneShotRequestEvent").count(), before_requests(&before) + 1);
    }

    fn before_requests(a: &Ainb) -> usize {
        crate::ainb::show::show(a).matches("SeqOneShotRequestEvent").count()
    }
}
