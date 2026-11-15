use crate::error::{Error as E, Res};
use crate::logic_items::ItemId;
use crate::search::explore;
use crate::settings::seed::{SeedFile, ShopEntry};
use crate::shops::price::wallets;
use crate::slice::data::Data;
use crate::slice::starts::rebuild;
use crate::spoiler::{playthrough, Spoiler};
use crate::world::{Placement, World};

pub const WALLET: &str = "Progressive Wallet";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Audit {
    pub won: bool,
    pub bought: usize,
    pub top: i64,
    pub bad: Vec<String>,
}

pub fn audit(d: &Data, seed: &SeedFile) -> Res<Audit> {
    let (w, place): (World, Placement) = rebuild(d, seed)?;
    let won: bool = explore(&w, &place, &[]).won();
    let sp: Spoiler = playthrough(&w, &place)?;
    let wallet: ItemId = w.item(WALLET).ok_or(E::Format("the logic has no Progressive Wallet"))?;
    let mut held: u8 = w.start.iter().filter(|(i, _)| *i == wallet).map(|(_, n)| *n as u8).sum();
    let mut out: Audit = Audit { won, ..Audit::default() };
    for sphere in &sp.spheres {
        for (l, _) in sphere {
            let name: &str = &w.locs[*l].name;
            let Some(e) = seed.shops.get(name) else {
                continue;
            };
            let e: &ShopEntry = e;
            out.bought += 1;
            out.top = out.top.max(e.price);
            let need: u8 = wallets(e.price as i32);
            if need > held {
                out.bad.push(format!("{name} sells {} for {} rupees in the playthrough with only {held} wallet upgrades", e.item, e.price));
            }
        }
        held = held.saturating_add(sphere.iter().filter(|(_, i)| *i == wallet).count() as u8);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Settings, Value};
    use crate::slice::generate::{generate_data, GenOpts, Generated};

    fn with(pairs: &[(&str, &str)]) -> Settings {
        let mut s: Settings = Settings::default();
        for (k, v) in pairs {
            s.set(k, Value::s(v)).unwrap();
        }
        s
    }

    #[test]
    fn shop_seeds_generate_on_every_level_win_and_never_ask_more_rupees_than_the_wallets_hold() {
        let d: Data = Data::load().unwrap();
        for lv in ["shuffle", "deals_1", "deals_4", "random"] {
            let mut s: Settings = with(&[("shuffle_shops", lv), ("shuffle_chests", "all")]);
            if lv != "shuffle" {
                s.set("shop_price_max", Value::Int(700)).unwrap();
                s.set("shop_deal_items", Value::set(&["stacks", "upgrades", "hearts", "songs"])).unwrap();
                s.set("shuffle_songs", Value::s(if lv == "deals_4" { "own" } else { "anywhere" })).unwrap();
            }
            for n in 0..2 {
                let g: Generated = generate_data(&d, &s, &format!("audit {lv} {n}"), &GenOpts { spoiler: true, ..GenOpts::default() }).unwrap_or_else(|e| panic!("{lv} {n}: {e}"));
                assert_eq!(g.seed_file.shops.len(), 64);
                let back: SeedFile = SeedFile::parse(&g.seed_file.to_text()).unwrap();
                let a: Audit = audit(&d, &back).unwrap();
                assert!(a.won, "{lv} {n}");
                assert!(a.bad.is_empty(), "{lv} {n}: {:?}", a.bad);
                assert!(g.spoiler.unwrap().contains("Shops, "));
                for (loc, e) in &back.shops {
                    if e.deal {
                        assert!(crate::shops::rules::shelf_ok(&d, &e.item), "{loc} {}", e.item);
                        assert_eq!(back.placements.get(loc), Some(&e.item));
                    } else {
                        assert!(!back.placements.contains_key(loc));
                    }
                }
            }
        }
    }


    #[test]
    fn the_same_seed_text_gives_the_same_shops_and_the_main_fill_does_not_follow_the_shop_draw() {
        let d: Data = Data::load().unwrap();
        let a: Generated = generate_data(&d, &with(&[("shuffle_shops", "shuffle")]), "same", &GenOpts::default()).unwrap();
        let b: Generated = generate_data(&d, &with(&[("shuffle_shops", "shuffle")]), "same", &GenOpts::default()).unwrap();
        assert_eq!(a.seed_file.to_text(), b.seed_file.to_text());
        let off: Generated = generate_data(&d, &Settings::default(), "same", &GenOpts::default()).unwrap();
        assert!(off.seed_file.shops.is_empty());
        assert!(!off.seed_file.to_text().contains("\"shops\""));
        assert!(!off.report.contains("Shop shelves"));
    }
}
