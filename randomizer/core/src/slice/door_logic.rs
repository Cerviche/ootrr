#[cfg(test)]
mod tests {
    use crate::logic_items::ItemId;
    use crate::rules::Age;
    use crate::search::{explore, Search};
    use crate::slice::data::Data;
    use crate::slice::logic::world;
    use crate::world::{Placement, World};

    const ROOM: &str = "Dodongos Cavern Upper Lizalfos";
    const BRIDGE: &str = "Dodongos Cavern Far Bridge";

    fn without_bow(w: &World, d: &Data) -> Placement {
        let mut p: Placement = crate::slice::fill::vanilla_place(w, d).unwrap();
        let bow: ItemId = w.item("Bow").unwrap();
        let f: ItemId = w.item("Rupees (5)").unwrap();
        for l in &w.real {
            if p.get(*l) == Some(bow) {
                p.put(*l, f);
            }
        }
        p
    }

    fn adult_at(w: &World, p: &Placement, more: &[&str], region: &str) -> bool {
        let extra: Vec<ItemId> = ["Hover Boots"].iter().chain(more.iter()).map(|n| w.item(n).unwrap()).collect();
        let s: Search = explore(w, p, &extra);
        s.region_reached(w.region(region).unwrap(), Some(Age::Adult))
    }

    #[test]
    fn the_upper_lizalfos_room_opens_only_from_the_eye_switch_door() {
        let d: Data = Data::load().unwrap();
        let w: World = world(&d).unwrap();
        let p: Placement = without_bow(&w, &d);
        assert!(adult_at(&w, &p, &[], BRIDGE));
        assert!(!adult_at(&w, &p, &[], ROOM));
        assert!(adult_at(&w, &p, &["Bow"], ROOM));
    }
}
