use crate::flow_locations::Loc;
use crate::progressive::family_of;
use crate::slice::data::{Data, Item};

pub fn grants_only(l: &Loc) -> bool {
    l.supported && !l.edits.is_empty() && l.edits.iter().all(|e| e.role == "grant")
}

pub fn tiered(l: &Loc, actor: &str) -> bool {
    l.edits.iter().any(|e| e.item != actor)
}

pub fn linked(i: &Item, actor: &str) -> bool {
    i.actor.as_deref() == Some(actor) || i.tiers.iter().any(|t| t == actor) || i.world.iter().any(|w| w == actor)
}

pub fn ok(d: &Data, name: &str) -> bool {
    let Some(l) = d.flow.by_name(name) else {
        return false;
    };
    if crate::flow_extra::trade::is(name) {
        return l.gated && d.vanilla_of(name).is_some_and(|v| l.item.as_deref() != d.item(v).and_then(|i| i.actor.as_deref()));
    }
    if crate::flow_extra::cows::is(name) || crate::flow_extra::frogs::is(name) || crate::flow_extra::potion::is(name) {
        return l.gated && l.item.is_some() && d.vanilla_of(name).is_some();
    }
    if crate::scrubs_b::is_b(l) {
        return crate::scrubs_b::ok(d, l);
    }
    if !(l.supported || l.gated) || l.conf == "guess" || l.edits.is_empty() {
        return false;
    }
    let Some(actor) = l.item.as_deref() else {
        return false;
    };
    let Some(i) = d.vanilla_of(name).and_then(|v| d.item(v)) else {
        return false;
    };
    if family_of(&i.tiers).is_some() {
        return (grants_only(l) || crate::scrubs_a::grants_only(l)) && linked(i, actor) && d.flow_accepts(i);
    }
    if tiered(l, actor) {
        return grants_only(l);
    }
    linked(i, actor)
}

pub fn allows(d: &Data, name: &str, i: &Item) -> bool {
    let Some(l) = d.flow.by_name(name) else {
        return true;
    };
    if family_of(&i.tiers).is_some() {
        return (grants_only(l) && l.cat != "scrub_sale" && d.flow_accepts(i)) || crate::scrubs_a::home(d, name, &i.name);
    }
    d.flow_accepts(i) || d.vanilla_of(name) == Some(i.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Settings, Value};
    use crate::slice::generate::{generate_data, GenOpts, Generated};

    const NEW: [&str; 18] = [
        "Graveyard Dampe Gravedigging Tour",
        "Kak 40 Gold Skulltula Reward",
        "Kak 100 Gold Skulltula Reward",
        "LW Target in Woods",
        "Market Shooting Gallery Reward",
        "Kak Shooting Gallery Reward",
        "Market Bombchu Bowling First Prize",
        "GF HBA 1500 Points",
        "Deku Theater Skull Mask",
        "Deku Theater Mask of Truth",
        "LLR Talons Chickens",
        "Kak 10 Gold Skulltula Reward",
        "Kak 30 Gold Skulltula Reward",
        "DMT Great Fairy Reward",
        "DMC Great Fairy Reward",
        "ZD Diving Minigame",
        "LH Adult Fishing",
        "GC Rolling Goron as Child",
    ];

    const PINNED: [&str; 7] = [
        "GC Darunias Joy",
        "LW Gift from Saria",
        "HF Ocarina of Time Item",
        "DMT Biggoron",
        "GC Medigoron",
        "KF Links House Cow",
        "Kak 20 Gold Skulltula Reward",
    ];

    fn item<'a>(d: &'a Data, n: &str) -> &'a Item {
        d.item(n).unwrap()
    }

    #[test]
    fn the_new_locations_are_free_and_the_risky_ones_stay_pinned() {
        let d: Data = Data::load().unwrap();
        for n in NEW {
            assert!(d.flow_ok(n), "{n}");
        }
        let off: crate::slice::support::Plan = crate::slice::support::plan(&Settings::default()).unwrap();
        for n in PINNED {
            let open: bool = d.flow.by_name(n).is_some_and(|l| crate::flow_extra::open(l, &off));
            assert!(!(d.flow_ok(n) && open), "{n}");
        }
        assert!(d.flow.by_name("GC Rolling Goron as Child").is_some_and(|l| l.supported && grants_only(l) && crate::flow_extra::open(l, &off)));
        for n in ["Kak 20 Gold Skulltula Reward", "DMT Biggoron", "GC Medigoron"] {
            assert!(d.flow_ok(n) && d.flow.by_name(n).is_some_and(|l| l.gated), "{n}");
        }
    }

    fn goron(over: &[(&str, &str)], gone: &[&str]) -> bool {
        let d: Data = Data::load().unwrap();
        let w: crate::world::World = crate::slice::logic::world_with(&d, over).unwrap();
        let mut p: crate::world::Placement = crate::slice::fill::vanilla_place(&w, &d).unwrap();
        let g: Vec<crate::logic_items::ItemId> = gone.iter().map(|n| w.item(n).unwrap()).collect();
        let f: crate::logic_items::ItemId = w.item("Rupees (5)").unwrap();
        for l in &w.real {
            if p.get(*l).is_some_and(|i| g.contains(&i)) {
                p.put(*l, f);
            }
        }
        let s: crate::search::Search = crate::search::explore(&w, &p, &[]);
        s.loc_access(w.loc("GC Rolling Goron as Child").unwrap())
    }

    #[test]
    fn the_child_rolling_goron_needs_a_bomb_bag_and_the_gate_rule_of_goron_city_only() {
        let chus: (&str, &str) = (crate::slice::bombchus::LOGIC_KEY, "true");
        let open: (&str, &str) = ("open_kakariko", "open");
        assert!(goron(&[], &[]));
        assert!(!goron(&[], &["Bomb Bag"]));
        assert!(!goron(&[chus], &["Bomb Bag"]));
        assert!(!goron(&[("logic_child_rolling_with_strength", "true")], &["Bomb Bag"]));
        assert!(!goron(&[], &["Zeldas Letter"]));
        assert!(goron(&[open], &["Zeldas Letter"]));
        assert!(!goron(&[open], &["Bomb Bag"]));
        assert!(goron(&[], &["Bombchus (10)","Bombchus (5)", "Bombchus (20)"]));
        let line: &str = include_str!("../../../../data/logic/World/Overworld.json").lines().find(|l| l.contains("\"GC Rolling Goron as Child\":")).unwrap();
        assert_eq!(line.trim(), "\"GC Rolling Goron as Child\": \"is_child and Bomb_Bag\",");
    }

    #[test]
    fn the_two_fountains_follow_their_flows_and_the_remake_items() {
        let d: Data = Data::load().unwrap();
        for (loc, file, item, vanilla) in [
            ("DMT Great Fairy Reward", "Dm_FairyFountain_Din.bfevfl", "Item_DinFire", "Dins Fire"),
            ("HC Great Fairy Reward", "Dm_FairyFountain_Power.bfevfl", "Item_FairyBless_MP", "Magic Meter"),
        ] {
            let l: &Loc = d.flow.by_name(loc).unwrap();
            assert_eq!(l.edits.len(), 1, "{loc}");
            assert_eq!(l.edits[0].file, file, "{loc}");
            assert_eq!(l.item.as_deref(), Some(item), "{loc}");
            assert_eq!(l.vanilla, vanilla, "{loc}");
            assert_eq!(d.vanilla_of(loc), Some(vanilla), "{loc}");
            assert!(d.flow_ok(loc), "{loc}");
        }
        assert_eq!(d.vanilla_of("DMC Great Fairy Reward"), Some("Magic Meter"));
        let skull: &Loc = d.flow.by_name("Kak 40 Gold Skulltula Reward").unwrap();
        assert_eq!(skull.vanilla, "Bombchus (20)");
        assert_eq!(d.vanilla_of("Kak 40 Gold Skulltula Reward"), Some("Bombchus (20)"));
        assert_eq!(d.item("Bombchus (20)").and_then(|i| i.actor.as_deref()), skull.item.as_deref());
    }

    #[test]
    fn tiered_locations_name_the_lower_tier_and_hold_a_junk_vanilla_item() {
        let d: Data = Data::load().unwrap();
        for n in ["LW Target in Woods", "Market Shooting Gallery Reward", "Kak Shooting Gallery Reward", "Market Bombchu Bowling First Prize", "GF HBA 1500 Points"] {
            let l: &Loc = d.flow.by_name(n).unwrap();
            assert_eq!(l.edits.len(), 2, "{n}");
            assert!(tiered(l, l.item.as_deref().unwrap()), "{n}");
            assert_eq!(d.vanilla_of(n), Some("Rupees (5)"), "{n}");
        }
        assert_eq!(d.flow.by_name("LW Target in Woods").map(|l| l.conf.as_str()), Some("scene"));
    }

    #[test]
    fn family_items_go_only_to_grant_only_flow_locations_with_accepted_tiers() {
        let d: Data = Data::load().unwrap();
        let wallet: &Item = item(&d, "Progressive Wallet");
        assert!(allows(&d, "Kak 50 Gold Skulltula Reward", wallet));
        assert!(allows(&d, "LW Target in Woods", wallet));
        assert!(!allows(&d, "ToT Light Arrows Cutscene", wallet));
        assert!(!allows(&d, "HF Deku Scrub Grotto", wallet));
        assert!(!allows(&d, "Song from Impa", wallet));
        assert!(!allows(&d, "Kak 50 Gold Skulltula Reward", item(&d, "Progressive Strength Upgrade")));
        assert!(!allows(&d, "Kak 50 Gold Skulltula Reward", item(&d, "Progressive Hookshot")));
        assert!(allows(&d, "Some Chest", item(&d, "Progressive Hookshot")));
    }

    #[test]
    fn a_location_always_accepts_its_own_vanilla_item_but_the_item_goes_nowhere_else() {
        let d: Data = Data::load().unwrap();
        let rupees: &Item = item(&d, "Rupees (5)");
        assert!(allows(&d, "LW Target in Woods", rupees));
        assert!(!allows(&d, "Kak 50 Gold Skulltula Reward", rupees));
        let milk: &Item = item(&d, "Bottle with Milk");
        assert!(allows(&d, "LLR Talons Chickens", milk));
        assert!(!allows(&d, "Kak 50 Gold Skulltula Reward", milk));
        let gold: &Item = item(&d, "Rupees (200)");
        assert!(allows(&d, "Kak 100 Gold Skulltula Reward", gold));
        assert!(!allows(&d, "Market Lost Dog", gold));
    }

    #[test]
    fn seeds_with_rewards_on_put_the_new_locations_in_the_pool_and_stay_valid() {
        let d: Data = Data::load().unwrap();
        let mut s: Settings = Settings::default();
        s.set("shuffle_chests", Value::s("all")).unwrap();
        s.set("shuffle_rewards", Value::s("supported")).unwrap();
        for n in 0..2 {
            let g: Generated = generate_data(&d, &s, &format!("flow rules {n}"), &GenOpts::default()).unwrap();
            for l in NEW {
                assert!(g.seed_file.placements.contains_key(l), "{l}");
            }
            for l in PINNED {
                assert!(!g.seed_file.placements.contains_key(l), "{l}");
            }
        }
    }
}
