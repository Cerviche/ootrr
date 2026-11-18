#[cfg(test)]
pub(super) mod tests {
    use crate::rules::{eval, Age, Ctx};
    use crate::search::Search;
    use crate::slice::data::Data;
    use crate::slice::logic::{world, world_with};
    use crate::world::{Loc, World};

    fn src() -> &'static str {
        crate::slice::world_files::overworld_text()
    }

    const EPONA: &str = "Epona from Lon Lon Ranch";

    const HBA: [&str; 2] = ["GF HBA 1000 Points", "GF HBA 1500 Points"];

    const HBA_KIT: [(&str, u32); 5] = [("Gerudo Membership Card", 1), ("Epona", 1), ("Ocarina", 1), ("Eponas Song", 1), ("Bow", 1)];

    pub(in crate::slice) fn open(w: &World, name: &str, have: &[(&str, u32)], age: Age) -> bool {
        let mut s: Search = Search::new(w);
        for (n, k) in have {
            s.add(w.item(n).unwrap(), *k);
        }
        let l: &Loc = &w.locs[w.loc(name).unwrap()];
        let c: Ctx = Ctx { cnt: &s.cnt, age: Some(age), tod: 0, region: l.region, q: Some(&s) };
        l.rule.always || (!l.rule.never && eval(&l.rule.node, &c))
    }

    pub(in crate::slice) fn rule_text<'a>(name: &str) -> &'a str {
        let key: String = format!("\"{name}\":");
        let at: usize = src().find(&key).unwrap() + key.len();
        let from: usize = at + src()[at..].find('"').unwrap() + 1;
        let to: usize = from + src()[from..].find('"').unwrap();
        &src()[from..to]
    }

    pub(in crate::slice) fn worlds(d: &Data) -> Vec<World> {
        vec![world(d).unwrap(), world_with(d, &[("time_always_passes", "0")]).unwrap(), world_with(d, &[("ensure_tod_access", "0")]).unwrap()]
    }

    #[test]
    fn the_gravedigging_tour_stays_a_child_check_at_dampe_time() {
        let d: Data = Data::load().unwrap();
        let tour: &str = "Graveyard Dampe Gravedigging Tour";
        assert_eq!(rule_text(tour), "is_child and at_dampe_time");
        let w: World = world(&d).unwrap();
        assert!(open(&w, tour, &[], Age::Child));
        assert!(!open(&w, tour, &[], Age::Adult));
        let slow: World = world_with(&d, &[("time_always_passes", "0")]).unwrap();
        assert!(!open(&slow, tour, &[], Age::Child));
        assert!(open(&slow, tour, &[("Zelda Met", 1)], Age::Child));
        assert!(!open(&slow, tour, &[("Zelda Met", 1)], Age::Adult));
    }

    #[test]
    fn horseback_archery_needs_the_card_an_adult_on_epona_with_her_song_and_a_bow() {
        let d: Data = Data::load().unwrap();
        for w in worlds(&d) {
            for l in HBA {
                assert!(open(&w, l, &HBA_KIT, Age::Adult), "{l}");
                assert!(!open(&w, l, &HBA_KIT, Age::Child), "{l}");
                for (gone, _) in HBA_KIT {
                    let rest: Vec<(&str, u32)> = HBA_KIT.iter().copied().filter(|(n, _)| *n != gone).collect();
                    assert!(!open(&w, l, &rest, Age::Adult), "{l} without {gone}");
                }
            }
        }
        for l in HBA {
            assert!(!rule_text(l).contains("at_day"), "{l}");
            assert!(!rule_text(l).contains("at_night"), "{l}");
        }
    }

    #[test]
    fn epona_needs_an_adult_with_the_ocarina_and_her_song_at_any_hour() {
        let d: Data = Data::load().unwrap();
        for w in worlds(&d) {
            assert!(open(&w, EPONA, &[("Ocarina", 1), ("Eponas Song", 1)], Age::Adult));
            assert!(!open(&w, EPONA, &[("Ocarina", 1), ("Eponas Song", 1)], Age::Child));
            assert!(!open(&w, EPONA, &[("Ocarina", 1)], Age::Adult));
            assert!(!open(&w, EPONA, &[("Eponas Song", 1)], Age::Adult));
            assert!(!open(&w, EPONA, &[], Age::Adult));
        }
        assert_eq!(rule_text("Epona"), "can_play(Eponas_Song) and is_adult");
        assert!(rule_text("Links Cow").contains("at_day"));
    }

    #[test]
    fn the_lab_dive_needs_the_gold_scale() {
        let d: Data = Data::load().unwrap();
        let w: World = world(&d).unwrap();
        for age in [Age::Child, Age::Adult] {
            assert!(!open(&w, "LH Lab Dive", &[], age));
            assert!(!open(&w, "LH Lab Dive", &[("Progressive Scale", 1)], age));
            assert!(open(&w, "LH Lab Dive", &[("Progressive Scale", 2)], age));
        }
        assert!(!open(&w, "LH Lab Dive", &[("Iron Boots", 1), ("Progressive Hookshot", 1)], Age::Adult));
        assert!(rule_text("LH Lab Dive").contains("(Progressive_Scale, 2)"));
    }

    #[test]
    fn the_time_rules_that_were_dropped_were_never_a_restriction_in_any_time_model() {
        let d: Data = Data::load().unwrap();
        let w: World = world_with(&d, &[("time_always_passes", "0"), ("ensure_tod_access", "1")]).unwrap();
        for l in HBA {
            assert!(open(&w, l, &HBA_KIT, Age::Adult), "{l}");
        }
        assert!(open(&w, EPONA, &[("Ocarina", 1), ("Eponas Song", 1)], Age::Adult));
    }
}
