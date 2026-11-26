use crate::error::{Error as E, Res};
use crate::settings::Settings;
use crate::world::{LocId, Placement, World};

pub fn apply(s: &Settings, w: &World, free: &[LocId], base: &mut Placement) -> Res<usize> {
    let mode: &str = s.text("item_pool")?;
    if mode == "balanced" {
        return Ok(0);
    }
    if !matches!(mode, "scarce" | "minimal") {
        return Err(E::Format("item_pool must be balanced, scarce or minimal"));
    }
    let red: usize = w.item("Rupees (20)").ok_or(E::Format("item pool: Red Rupee is missing"))?;
    let mut counts: [usize; 2] = [0, 0];
    let mut changed: usize = 0;
    for l in free {
        let Some(i) = base.get(*l) else {
            continue;
        };
        let k: usize = match w.items.names[i].as_str() {
            "Piece of Heart" | "Piece of Heart (Out of Logic)" | "Piece of Heart (Treasure Chest Game)" => 0,
            "Heart Container" => 1,
            _ => continue,
        };
        counts[k] += 1;
        if mode == "minimal" || counts[k] % 2 == 0 {
            base.put(*l, red);
            changed += 1;
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Value;
    use crate::slice::data::Data;
    use crate::slice::fill::vanilla_place;
    use crate::slice::logic::world;

    #[test]
    fn balanced_preserves_the_entire_placement() {
        let d: Data = Data::load().unwrap();
        let w: World = world(&d).unwrap();
        let mut b: Placement = vanilla_place(&w, &d).unwrap();
        let old: Vec<Option<usize>> = b.item.clone();
        assert_eq!(apply(&Settings::default(), &w, &w.real, &mut b).unwrap(), 0);
        assert_eq!(b.item, old);
    }

    #[test]
    fn scarce_and_minimal_only_replace_selected_health_items() {
        let d: Data = Data::load().unwrap();
        let w: World = world(&d).unwrap();
        let piece: usize = w.item("Piece of Heart").unwrap();
        let heart: usize = w.item("Heart Container").unwrap();
        let red: usize = w.item("Rupees (20)").unwrap();
        let tool: usize = w.item("Boomerang").unwrap();
        let locs: &[LocId] = &w.real[..9];
        let items: [usize; 9] = [piece, heart, piece, heart, piece, tool, piece, heart, heart];
        for (mode, changed, kept) in [("scarce", 3, 4), ("minimal", 7, 0)] {
            let mut s: Settings = Settings::default();
            s.set("item_pool", Value::s(mode)).unwrap();
            let mut b: Placement = Placement::new(w.locs.len());
            for (l, i) in locs.iter().zip(items) {
                b.put(*l, i);
            }
            assert_eq!(apply(&s, &w, &locs[..8], &mut b).unwrap(), changed);
            assert_eq!(locs[..8].iter().filter(|l| b.get(**l) == Some(red)).count(), changed);
            assert_eq!(locs[..8].iter().filter(|l| matches!(b.get(**l), Some(i) if i == piece || i == heart)).count(), kept);
            assert_eq!(b.get(locs[5]), Some(tool));
            assert_eq!(b.get(locs[8]), Some(heart));
            assert_eq!(b.item.iter().filter(|i| i.is_some()).count(), 9);
        }
    }
}
