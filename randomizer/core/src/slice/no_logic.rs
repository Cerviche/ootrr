use std::fmt::Write as W;

use crate::error::{Error as E, Res};
use crate::logic_items::ItemId;
use crate::rng::Rng;
use crate::settings::Settings;
use crate::slice::fit::{Fit, Grid};
use crate::slice::watch::Watch;
use crate::world::{LocId, Placement, World};

pub const NOTE: &str = "No Logic: item access and beatability were not checked. This seed may be impossible to finish. No verified playthrough is available.";

pub fn enabled(s: &Settings) -> Res<bool> {
    Ok(s.text("logic_rules")? == "no_logic")
}

pub fn fill(w: &World, base: &Placement, free: &[LocId], allow: &dyn Fn(ItemId, LocId) -> bool, rng: &mut Rng, fixed: &[(LocId, ItemId)], watch: &Watch) -> Res<Placement> {
    watch.check()?;
    let mut out: Placement = base.clone();
    let mut pool: Vec<ItemId> = Vec::with_capacity(free.len());
    for l in free {
        pool.push(out.get(*l).ok_or_else(|| E::Msg(format!("No Logic: {} has no pool item", w.locs[*l].name)))?);
        out.item[*l] = None;
    }
    let mut locked: Vec<LocId> = Vec::new();
    for (l, i) in fixed {
        if !free.contains(l) || locked.contains(l) || !allow(*i, *l) {
            return Err(E::Msg(format!("plando: {} cannot hold {} under these settings", w.locs[*l].name, w.items.names[*i])));
        }
        let at: usize = pool.iter().position(|x| x == i).ok_or_else(|| E::Msg(format!("plando: {} has no copy left to place at {}", w.items.names[*i], w.locs[*l].name)))?;
        pool.swap_remove(at);
        out.put(*l, *i);
        locked.push(*l);
    }
    let mut open: Vec<LocId> = free.iter().copied().filter(|l| !locked.contains(l)).collect();
    rng.shuffle(&mut pool);
    rng.shuffle(&mut open);
    let g: Grid = Grid::new(&pool, &open, allow);
    let mut fit: Fit = Fit::new(&g);
    if let Some(short) = fit.short(w) {
        return Err(E::Msg(crate::slice::fill::short_text(&short)));
    }
    let mut items: Vec<usize> = (0..pool.len()).collect();
    let mut spots: Vec<usize> = (0..open.len()).collect();
    rng.shuffle(&mut items);
    for p in items {
        watch.tick()?;
        rng.shuffle(&mut spots);
        let at: usize = spots.iter().position(|j| fit.take(p, *j)).ok_or(E::Format("No Logic: no compatible location remains"))?;
        let j: usize = spots.swap_remove(at);
        out.put(open[j], pool[p]);
    }
    Ok(out)
}

pub fn spoiler(w: &World, got: &Placement, free: &[LocId]) -> String {
    let mut out: String = format!("{NOTE}\n\nPlacements\n");
    for l in free {
        if let Some(i) = got.get(*l) {
            let _ = writeln!(out, "  {}: {}", w.locs[*l].name, crate::spoiler::shown(&w.items.names[i]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Value, seed::SeedFile};
    use crate::slice::data::Data;
    use crate::slice::generate::{GenOpts, Generated, generate_data};
    use crate::world::Spec;

    fn world(text: &str) -> World {
        World::build(&Spec::from_bundle(text).unwrap()).unwrap()
    }

    const LOCKED: &str = r#"{
        "goal": "Win",
        "items": [{"name":"Sword","advancement":true}],
        "regions": [{"region_name":"Root","locations":{"Locked Chest":"Sword"},"events":{"Win":"Sword"}}]
    }"#;

    #[test]
    fn an_item_can_be_locked_behind_itself_and_fixed_there() {
        let w: World = world(LOCKED);
        let l: LocId = w.loc("Locked Chest").unwrap();
        let i: ItemId = w.item("Sword").unwrap();
        let mut base: Placement = Placement::new(w.locs.len());
        base.put(l, i);
        let got: Placement = fill(&w, &base, &[l], &|_, _| true, &mut Rng::from_seed(1), &[(l, i)], &Watch::idle()).unwrap();
        assert_eq!(got.get(l), Some(i));
        assert!(!crate::search::explore(&w, &got, &[]).won());
        assert!(crate::slice::fill::fill(&w, &base, &[l], &|_, _| true, &mut Rng::from_seed(1), 1, &[(l, i)], false).is_err());
    }

    #[test]
    fn matching_preserves_counts_restrictions_and_fixed_placements() {
        let w: World = world(r#"{"goal":"Win","items":[{"name":"Sword","advancement":true},"Boots","Rupee"],"regions":[{"region_name":"Root","locations":{"A":"False","B":"False","C":"False"},"events":{"Win":"False"}}]}"#);
        let a: LocId = w.loc("A").unwrap();
        let b: LocId = w.loc("B").unwrap();
        let c: LocId = w.loc("C").unwrap();
        let sword: ItemId = w.item("Sword").unwrap();
        let boots: ItemId = w.item("Boots").unwrap();
        let rupee: ItemId = w.item("Rupee").unwrap();
        let mut base: Placement = Placement::new(w.locs.len());
        base.put(a, sword);
        base.put(b, boots);
        base.put(c, rupee);
        let allow = |i: ItemId, l: LocId| (i != sword || l == b) && (l != b || i == sword);
        for n in 0..12 {
            let got: Placement = fill(&w, &base, &[a, b, c], &allow, &mut Rng::from_seed(n), &[(c, boots)], &Watch::idle()).unwrap();
            assert_eq!(got.get(b), Some(sword));
            assert_eq!(got.get(c), Some(boots));
            assert_eq!(got.get(a), Some(rupee));
            assert_eq!(got.item.iter().filter(|i| i.is_some()).count(), 3);
        }
        assert!(fill(&w, &base, &[a, b, c], &allow, &mut Rng::from_seed(1), &[(a, sword)], &Watch::idle()).is_err());
        assert!(fill(&w, &base, &[a, b, c], &|_, _| false, &mut Rng::from_seed(1), &[], &Watch::idle()).is_err());
    }

    #[test]
    fn generated_seeds_reload_build_actors_and_keep_excluded_plando() {
        let d: Data = Data::load().unwrap();
        for (mode, wide) in [("balanced", false), ("minimal", true)] {
            let mut s: Settings = Settings::default();
            s.set("logic_rules", Value::s("no_logic")).unwrap();
            s.set("item_pool", Value::s(mode)).unwrap();
            if wide {
                s.set("shuffle_freestanding_items", Value::s("all")).unwrap();
                s.set("shuffle_skulltulas", Value::s("matched")).unwrap();
                s.set("shuffle_songs", Value::s("anywhere")).unwrap();
            }
            s.set("disabled_locations", Value::set(&["KF Midos Top Left Chest"])).unwrap();
            let o: GenOpts = GenOpts { spoiler: true, plando: vec![crate::slice::plando::parse("KF Midos Top Left Chest=Rupees (20)").unwrap()], ..GenOpts::default() };
            let g: Generated = generate_data(&d, &s, "no logic integration", &o).unwrap();
            let seed: SeedFile = SeedFile::parse(&g.seed_file.to_text()).unwrap();
            assert_eq!(seed.placements.get("KF Midos Top Left Chest").map(|x| x.as_str()), Some("Rupees (20)"));
            assert!(enabled(&seed.settings().unwrap().settings).unwrap());
            assert!(g.spheres.is_empty());
            assert!(g.spoiler.as_ref().unwrap().contains(NOTE));
            assert!(g.report.contains(NOTE));
            assert!(!g.report.contains("Logic: OoTR glitchless"));
            assert!(!crate::slice::chest_targets::actors_for(&d, &seed).unwrap().is_empty());
            let again: Generated = generate_data(&d, &s, "no logic integration", &o).unwrap();
            assert_eq!(seed.to_text(), again.seed_file.to_text());
        }
    }

    #[test]
    fn cancellation_is_honoured() {
        let w: World = world(LOCKED);
        let l: LocId = w.loc("Locked Chest").unwrap();
        let mut base: Placement = Placement::new(w.locs.len());
        base.put(l, w.item("Sword").unwrap());
        let watch: Watch = Watch::new(Some(std::time::Duration::ZERO), None);
        assert!(fill(&w, &base, &[l], &|_, _| true, &mut Rng::from_seed(1), &[], &watch).unwrap_err().to_string().contains(crate::slice::watch::OVER));
    }
}
