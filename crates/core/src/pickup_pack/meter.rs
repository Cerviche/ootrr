use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::pickup_pack::rows::{find, need, POUCH_INFO};

pub const NONE: [&str; 2] = ["Item_DinFire", "Item_Arrow_Ice"];
pub const KIND: &str = "ExpansionAvility";
pub const AMOUNT: &str = "MPMax";
pub const RAISE: &str = "MPMaxUP";

pub fn applies(who: &str) -> bool {
    NONE.contains(&who)
}

pub fn raises(r: &Node) -> bool {
    r.get(KIND).and_then(|v| v.as_str()) == Some(RAISE)
}

pub fn strip(r: &mut Node) -> bool {
    if !raises(r) {
        return false;
    }
    r.remove(KIND);
    r.remove(AMOUNT);
    true
}

fn row<'a>(y: &'a mut Byml, who: &str) -> Res<&'a mut Node> {
    y.root
        .as_arr_mut()
        .ok_or(E::Format("actor table: the root is not a row list"))?
        .iter_mut()
        .find(|r| r.get("__RowId").and_then(|v| v.as_str()) == Some(who))
        .ok_or_else(|| E::Msg(format!("{who} has no row in {POUCH_INFO}")))
}

pub fn apply(y: &mut Byml, name: &str, identity: &str) -> Res<()> {
    for who in [name, identity] {
        strip(row(y, who)?);
    }
    Ok(())
}

pub fn audit(was: &Byml, now: &Byml, name: &str, identity: &str) -> Res<String> {
    for who in [name, identity] {
        if raises(need(now, who, POUCH_INFO)?) || need(now, who, POUCH_INFO)?.get(AMOUNT).is_some() {
            return Err(E::Msg(format!("audit: {who} still carries a magic raise")));
        }
    }
    let mut want: Node = need(was, identity, POUCH_INFO)?.clone();
    strip(&mut want);
    if need(now, identity, POUCH_INFO)? != &want {
        return Err(E::Msg(format!("audit: the row of {identity} changed in more than the magic raise")));
    }
    if let Some(old) = find(was, name) {
        let mut w: Node = old.clone();
        strip(&mut w);
        if need(now, name, POUCH_INFO)? != &w {
            return Err(E::Msg(format!("audit: the row of {name} changed in more than the magic raise")));
        }
    }
    Ok(format!("magic raise: {name} and {identity} carry no {KIND} and no {AMOUNT}, no other column changed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pickup_pack::rows::tests::{row, table};
    use crate::pickup_pack::rows::{insert, pouch_row};

    fn s(x: &str) -> Node {
        Node::Str(x.to_string())
    }

    fn sample() -> Byml {
        let mut y: Byml = table(vec![
            row("Item_DinFire", &[(KIND, s(RAISE)), (AMOUNT, Node::I32(24)), ("PouchCategory", s("Material"))]),
            row("Item_FairyBless_MP", &[(KIND, s(RAISE)), (AMOUNT, Node::I32(24)), ("PouchSpecialDeal", s("FairyBlessMP"))]),
        ]);
        let new: Node = pouch_row(find(&y, "Item_DinFire").unwrap(), "Obj_DinFire");
        insert(&mut y, new).unwrap();
        y
    }

    #[test]
    fn only_the_spell_and_the_ice_arrow_lose_the_raise() {
        assert!(applies("Item_DinFire") && applies("Item_Arrow_Ice"));
        for keep in ["Item_FairyBless_MP", "Item_FairyBless_MPDouble", "Item_Arrow_Fire", "Item_Arrow_Light", "Item_FaroreWind", "Item_NayruLove", "Item_GlassTruth"] {
            assert!(!applies(keep), "{keep}");
        }
    }

    #[test]
    fn the_new_row_and_the_shipped_row_lose_the_raise_and_nothing_else() {
        let was: Byml = sample();
        let mut y: Byml = was.clone();
        apply(&mut y, "Obj_DinFire", "Item_DinFire").unwrap();
        for who in ["Obj_DinFire", "Item_DinFire"] {
            let r: &Node = find(&y, who).unwrap();
            assert_eq!(r.get(KIND), None, "{who}");
            assert_eq!(r.get(AMOUNT), None, "{who}");
            assert_eq!(r.get("PouchCategory"), Some(&s("Material")), "{who}");
        }
        assert_eq!(find(&y, "Item_FairyBless_MP"), find(&was, "Item_FairyBless_MP"));
        assert!(audit(&was, &y, "Obj_DinFire", "Item_DinFire").unwrap().contains("no other column changed"));
        assert!(audit(&was, &was, "Obj_DinFire", "Item_DinFire").is_err());
        apply(&mut y, "Obj_DinFire", "Item_DinFire").unwrap();
        assert!(audit(&y, &y, "Obj_DinFire", "Item_DinFire").is_ok());
    }

    #[test]
    fn a_missing_row_is_refused() {
        let mut y: Byml = sample();
        assert!(apply(&mut y, "Obj_Gone", "Item_DinFire").is_err());
        assert!(apply(&mut y, "Obj_DinFire", "Item_Gone").is_err());
    }

    #[test]
    fn the_meter_items_keep_their_raise() {
        let mut y: Byml = sample();
        assert!(!applies("Item_FairyBless_MP"));
        apply(&mut y, "Obj_DinFire", "Item_DinFire").unwrap();
        assert!(raises(find(&y, "Item_FairyBless_MP").unwrap()));
    }
}
