use crate::settings::{Cond, Setting, Settings, Value};

pub const SETTING: &str = "shuffle_shops";
pub const PRICES: &str = "shop_prices";
pub const MAX: &str = "shop_price_max";
pub const MIN: &str = "shop_price_min";
pub const ITEMS: &str = "shop_deal_items";
pub const STACKS: &str = "stacks";
pub const UPGRADES: &str = "upgrades";
pub const HEARTS: &str = "hearts";
pub const SONGS: &str = "songs";
pub const GEAR: &str = "shop_deal_gear";
pub const LOW: i32 = 5;
pub const HIGH: i32 = 995;

pub const NOTE: &str = "Shop shelves are shuffled. The shop goods trade places between the eight shelf shops, each keeps its own price, and the logic needs a bigger wallet for anything above 200 rupees. Experimental, untested in the game.";
pub const DEALS: &str = "Special deals sell a random item once, through a new shelf actor per item and price. Only items that cannot be held twice go on a deal, so a second copy of a shop is turned away with the bag full message. The Potion Shops get no deals. Experimental, only one Kokiri Shop deal is tested in the game.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Off,
    Shuffle,
    Deals(u8),
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prices {
    Vanilla,
    Affordable,
    Weighted,
    Uniform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opts {
    pub level: Level,
    pub prices: Prices,
    pub min: i32,
    pub max: i32,
    pub kinds: Kinds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Kinds {
    pub stacks: bool,
    pub upgrades: bool,
    pub hearts: bool,
    pub songs: bool,
    pub gear: bool,
}

impl Kinds {
    pub fn once(&self) -> bool {
        self.stacks || self.hearts
    }
}

impl Default for Opts {
    fn default() -> Opts {
        Opts { level: Level::Off, prices: Prices::Weighted, min: 10, max: 200, kinds: Kinds::default() }
    }
}

impl Opts {
    pub fn on(&self) -> bool {
        self.level != Level::Off
    }

    pub fn deals(&self) -> bool {
        matches!(self.level, Level::Deals(_) | Level::Random)
    }

    pub fn notes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if self.on() {
            out.push(NOTE.to_string());
        }
        if self.deals() {
            out.push(DEALS.to_string());
        }
        out
    }
}

pub const DEAL_LEVELS: [&str; 5] = ["deals_1", "deals_2", "deals_3", "deals_4", "random"];

fn with_deals(s: Setting) -> Setting {
    s.dep(SETTING, Cond::In(DEAL_LEVELS.iter().map(|v| Value::s(v)).collect()))
}

pub fn settings() -> Vec<Setting> {
    vec![
        Setting::choice(58, SETTING, "shuffle", "Shop shelves", "off")
            .tip("Whether the shelves of the eight shelf shops sell shuffled goods or random items. The Happy Mask Shop stays as it is. Prices above 200 rupees need a bigger wallet in the logic. Experimental, none of these edits are tested in the game yet except one Kokiri Shop deal.")
            .opt("off", "Vanilla", "Every shop sells its usual goods.")
            .opt("shuffle", "Shuffled goods", "The usual shop goods move between the shops and keep their prices, nothing new is sold.")
            .opt("deals_1", "1 special deal", "One slot in each shop sells a random item once, the other slots sell shuffled shop goods.")
            .opt("deals_2", "2 special deals", "Two slots in each shop sell a random item once, the other slots sell shuffled shop goods.")
            .opt("deals_3", "3 special deals", "Three slots in each shop sell a random item once, the other slots sell shuffled shop goods.")
            .opt("deals_4", "4 special deals", "Four slots in each shop sell a random item once, the other slots sell shuffled shop goods.")
            .opt("random", "Random deals", "Each shop gets 0 to 4 special deals, the Kokiri Shop at least one."),
        with_deals(
            Setting::choice(59, PRICES, "shuffle", "Special deal prices", "weighted")
                .tip("How the price of a special deal is chosen. Shop goods always keep their own price.")
                .opt("vanilla", "Vanilla", "A deal costs what its slot cost before.")
                .opt("affordable", "Affordable", "Every deal costs 10 rupees.")
                .opt("weighted", "Weighted", "Prices between the lowest and the highest price, leaning to the middle.")
                .opt("uniform", "Uniform", "Every price between the lowest and the highest price is equally likely."),
        ),
        with_deals(Setting::number(60, MAX, "shuffle", "Highest deal price", 200, LOW as i64, HIGH as i64).tip("The highest price a special deal can have, in steps of 5 rupees. Above 200 the logic asks for the Adult's Wallet, above 500 for the Giant's Wallet. Kokiri Shop deals never cost more than 200, so the Kokiri Sword and other early items stay in reach.")),
        with_deals(
            Setting::multi(62, ITEMS, "shuffle", "More items on deals")
                .tip("Special deals always take items that cannot be held twice. These classes can be added. Ammo, refills and hearts only go to the shops with one keeper (the two Bazaars and the Bombchu Shop), since only there one purchase closes the slot for good. Experimental, untested in the game.")
                .opts(&[(STACKS, "Ammo and refills, sold once. The Bazaars and the Bombchu Shop then keep their deals for these and for hearts"), (UPGRADES, "Upgrades: the shelf shows the first tier and the purchase gives the next tier the player needs"), (HEARTS, "Pieces of Heart and Heart Containers, sold once in the Bazaars and the Bombchu Shop"), (SONGS, "Songs on a harp model, when songs can go anywhere")]),
        ),
        with_deals(
            Setting::toggle(63, GEAR, "shuffle", "Bow, Slingshot, Bomb Bag and Magic on deals", false).tip("Lets the Bow, the Slingshot, the Bomb Bag and the Magic Meter stand on special deals. The shelf shows the first item of the family. The purchase checks what the player owns, like a progressive chest. A player without the item gets it with what the chest or the Great Fairy adds, the small quiver and 30 arrows, the small bullet bag and 30 seeds, 20 bombs, or the first magic bar. A player who owns a tier gets the next one instead: the bigger quiver, bullet bag or bomb bag, or the double magic bar. Experimental. Untested in the game, including whether the shop sells the deal to a player who already owns the first item."),
        ),
        with_deals(Setting::number(61, MIN, "shuffle", "Lowest deal price", 10, LOW as i64, HIGH as i64).tip("The lowest price a special deal can have, in steps of 5 rupees.").max_by(MAX)),
    ]
}

fn level(v: &str) -> Option<Level> {
    match v {
        "off" => Some(Level::Off),
        "shuffle" => Some(Level::Shuffle),
        "deals_1" => Some(Level::Deals(1)),
        "deals_2" => Some(Level::Deals(2)),
        "deals_3" => Some(Level::Deals(3)),
        "deals_4" => Some(Level::Deals(4)),
        "random" => Some(Level::Random),
        _ => None,
    }
}

fn prices(v: &str) -> Option<Prices> {
    match v {
        "vanilla" => Some(Prices::Vanilla),
        "affordable" => Some(Prices::Affordable),
        "weighted" => Some(Prices::Weighted),
        "uniform" => Some(Prices::Uniform),
        _ => None,
    }
}

pub fn read(s: &Settings, bad: &mut Vec<String>) -> Opts {
    let mut o: Opts = Opts::default();
    match s.text(SETTING) {
        Ok(v) => match level(v) {
            Some(l) => o.level = l,
            None => bad.push(format!("{SETTING} is {v}, which is not a known choice")),
        },
        Err(e) => bad.push(e.to_string()),
    }
    if !o.deals() {
        return o;
    }
    match s.text(PRICES) {
        Ok(v) => match prices(v) {
            Some(p) => o.prices = p,
            None => bad.push(format!("{PRICES} is {v}, which is not a known choice")),
        },
        Err(e) => bad.push(e.to_string()),
    }
    match (s.int(MIN), s.int(MAX)) {
        (Ok(a), Ok(b)) => {
            o.min = (a as i32).clamp(LOW, HIGH);
            o.max = (b as i32).clamp(LOW, HIGH);
            if o.min > o.max {
                bad.push(format!("{MIN} is {a}, above {MAX} {b}"));
            }
        }
        (Err(e), _) | (_, Err(e)) => bad.push(e.to_string()),
    }
    match s.items(ITEMS) {
        Ok(v) => {
            for k in v {
                match k.as_str() {
                    STACKS => o.kinds.stacks = true,
                    UPGRADES => o.kinds.upgrades = true,
                    HEARTS => o.kinds.hearts = true,
                    SONGS => o.kinds.songs = true,
                    x => bad.push(format!("{ITEMS} includes {x}, which is not a known class")),
                }
            }
        }
        Err(e) => bad.push(e.to_string()),
    }
    match s.flag(GEAR) {
        Ok(v) => o.kinds.gear = v,
        Err(e) => bad.push(e.to_string()),
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Value;

    #[test]
    fn off_by_default_and_every_level_reads() {
        let mut bad: Vec<String> = Vec::new();
        let o: Opts = read(&Settings::default(), &mut bad);
        assert!(bad.is_empty() && !o.on() && !o.deals() && o.notes().is_empty());
        let mut s: Settings = Settings::default();
        for (v, l) in [("shuffle", Level::Shuffle), ("deals_1", Level::Deals(1)), ("deals_4", Level::Deals(4)), ("random", Level::Random)] {
            s.set(SETTING, Value::s(v)).unwrap();
            let o: Opts = read(&s, &mut bad);
            assert_eq!(o.level, l);
            assert!(o.notes()[0].contains("Experimental"));
        }
        assert!(bad.is_empty(), "{bad:?}");
    }

    #[test]
    fn prices_are_read_only_for_deals() {
        let mut s: Settings = Settings::default();
        s.set(SETTING, Value::s("deals_2")).unwrap();
        s.set(PRICES, Value::s("uniform")).unwrap();
        s.set(MAX, Value::Int(300)).unwrap();
        s.set(MIN, Value::Int(40)).unwrap();
        let mut bad: Vec<String> = Vec::new();
        let o: Opts = read(&s, &mut bad);
        assert_eq!((o.prices, o.min, o.max), (Prices::Uniform, 40, 300));
        s.set(SETTING, Value::s("shuffle")).unwrap();
        assert_eq!(read(&s, &mut bad).prices, Prices::Weighted);
        assert!(bad.is_empty(), "{bad:?}");
    }
}
