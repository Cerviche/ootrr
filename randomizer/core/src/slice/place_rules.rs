use crate::logic_items::ItemId;
use crate::slice::data::Data;
use crate::slice::pickups as pu;
use crate::slice::support::{Plan, Songs};
use crate::world::{LocId, World};

pub struct Spot<'a> {
    pub d: &'a Data,
    pub w: &'a World,
    pub p: &'a Plan,
    pub it: ItemId,
    pub l: LocId,
    pub name: &'a str,
    pub item: &'a str,
}

pub type Rule = fn(&Spot) -> Option<bool>;

pub const RULES: [(&str, Rule); 16] = [
    ("ganon's boss key on the light arrows cutscene", lacs_key),
    ("dungeon rewards at the end of dungeons", end_of_dungeons),
    ("excluded locations", excluded),
    ("junk only spots", junk),
    ("silver rupees", silver),
    ("trade items", trade),
    ("skipped locations", skipped),
    ("gated npc rewards keep their vanilla items", gated),
    ("shops", shops),
    ("song spots", songs),
    ("treasure chest game", treasure),
    ("remake overworld chests", remake),
    ("freestanding pickups", freestanding),
    ("pots, crates and other holders", holders),
    ("gold skulltulas", skulls),
    ("world only pickups", world_only),
];

/// The rules the settings ask for: they stay when every item may go everywhere.
pub const SETTING_RULES: [(&str, Rule); 7] = [
    ("ganon's boss key on the light arrows cutscene", lacs_key),
    ("dungeon rewards at the end of dungeons", end_of_dungeons),
    ("excluded locations", excluded),
    ("silver rupees", silver),
    ("adult trade items", adult_trade),
    ("skipped locations", skipped),
    ("song spots", songs),
];

/// The rules that keep a seed safe however freely items move: spots the logic or the game is
/// unsure of (unsure Gold Skulltulas in matched mode, the unsure remake chests and heart spots)
/// keep progression out, the Treasure Chest Game reward takes only what its game can show, a
/// spot that exists once per age only takes items a second copy does no harm, and a boss heart
/// only takes an upgrade it can stand as a pickup of, and pot junk (the Recovery Heart and the
/// other items no chest holds) stays out of chests and NPC gifts. Only their refusals count; a
/// spot they pass still goes through the build checks.
pub const SAFETY_RULES: [(&str, Rule); 7] = [
    ("junk only spots", junk),
    ("treasure chest game", treasure),
    ("remake overworld chests", remake),
    ("unsure freestanding spots", unsure_free),
    ("spots that exist once per age", copies),
    ("upgrades at boss hearts", heart_upgrade),
    ("pot junk in chests and gifts", pot_junk),
];

pub fn safety_refuses(s: &Spot) -> bool {
    SAFETY_RULES.iter().any(|(_, r)| r(s) == Some(false))
}

fn unsure_free(s: &Spot) -> Option<bool> {
    (s.p.world.on() && pu::keeps_out(s.d, s.name, s.item)).then_some(false)
}

fn copies(s: &Spot) -> Option<bool> {
    (!crate::slice::copies::fits(s.d, s.name, s.item)).then_some(false)
}

/// An upgrade at a boss heart stands as the ground pickup of the tier slice::tier_order fixes
/// there. Without a pickup for every tier the build falls back to a heart chest, which has no
/// tier wrapper and drops the pool's actor instead of that tier (slice::chest_targets).
fn heart_upgrade(s: &Spot) -> Option<bool> {
    let heart: bool = s.d.cat.by_name(s.name).is_some_and(|c| c.kind == "bossheart");
    (heart && s.d.family(s.item).is_some() && !crate::slice::key_spots::placeable(s.d, s.item, crate::slice::key_spots::Spot::Free)).then_some(false)
}

/// Items that only stand in the world (pots, the ground, skulltulas) or sell on a shelf. Their
/// chest form is a guess (data/items.json) and untested in the game, so chests and NPC gifts keep
/// them out, as before every item could go everywhere.
fn pot_junk(s: &Spot) -> Option<bool> {
    let junk: bool = pu::WORLD_ONLY.contains(&s.item) || s.d.pots.only.contains(s.item);
    let gift: bool = s.d.flow.by_name(s.name).is_some() || s.d.cat.by_name(s.name).is_some_and(|c| c.kind == "chest");
    (junk && gift && !crate::shops::is_loc(s.name)).then_some(false)
}

pub fn setting_verdict(s: &Spot) -> Option<bool> {
    SETTING_RULES.iter().find_map(|(_, r)| r(s))
}

/// The adult trade chain keeps its items at home: the sequence is not shuffled.
fn adult_trade(s: &Spot) -> Option<bool> {
    let child: [&str; 2] = ["Item_WeirdEgg", "Item_ZeldaLetter"];
    s.d.item(s.item)
        .filter(|i| i.actor.as_deref().is_some_and(|a| crate::flow_extra::trade::chain(a) && !child.contains(&a)))
        .is_some_and(|i| !crate::flow_extra::trade::keeps(s.d, s.name, i))
        .then_some(false)
}

pub fn verdict(s: &Spot) -> Option<bool> {
    RULES.iter().find_map(|(_, r)| r(s))
}

fn lacs_key(s: &Spot) -> Option<bool> {
    if s.p.keys.ganon != crate::slice::support::Keep::Lacs {
        return None;
    }
    if s.name == crate::lacs_key::LOC {
        return Some(s.item == crate::lacs_key::ITEM);
    }
    (s.item == crate::lacs_key::ITEM).then_some(false)
}

/// Dungeon Rewards = End of dungeons: a Stone or Medallion only goes to a boss or Rauru reward
/// event, and those events only take a Stone or Medallion. A fitting pair is left to the other rules.
fn end_of_dungeons(s: &Spot) -> Option<bool> {
    if !s.p.rewards.dungeon_own {
        return None;
    }
    let item: bool = s.d.item(s.item).is_some_and(|i| i.kind == "DungeonReward");
    let spot: bool = s.d.flow.by_name(s.name).is_some_and(|l| l.cat == crate::slice::support::DUNGEON_REWARD_CAT);
    (item != spot).then_some(false)
}

fn excluded(s: &Spot) -> Option<bool> {
    (!s.p.excluded.is_empty() && s.w.items.adv[s.it] && s.p.excluded.iter().any(|e| *e == s.name)).then_some(false)
}

fn junk(s: &Spot) -> Option<bool> {
    crate::slice::junk_spots::refuses(s.d, s.w, s.p, s.it, s.l).then_some(false)
}

fn silver(s: &Spot) -> Option<bool> {
    crate::slice::silver::at(s.d, s.w, s.p, s.it, s.l)
}

fn trade(s: &Spot) -> Option<bool> {
    s.d.item(s.item).is_some_and(|i| !crate::flow_extra::trade::keeps(s.d, s.name, i)).then_some(false)
}

fn skipped(s: &Spot) -> Option<bool> {
    (crate::slice::edit::skipped_locations(s.p).contains(&s.name) && !s.d.item(s.item).is_some_and(|i| i.actor.is_some() || !i.tiers.is_empty())).then_some(false)
}

fn gated(s: &Spot) -> Option<bool> {
    (s.p.rewards.extra && s.d.item(s.item).is_some_and(|i| !crate::flow_extra::fits(s.d, s.name, i))).then_some(false)
}

fn shops(s: &Spot) -> Option<bool> {
    crate::shops::rules::at(s.d, &s.p.shops, s.p.songs == Songs::Anywhere, s.name, s.item)
}

fn songs(s: &Spot) -> Option<bool> {
    crate::slice::song_spots::gate(s.d, s.p, s.name, s.item)
}

fn treasure(s: &Spot) -> Option<bool> {
    crate::slice::treasure::at(s.p.treasure, s.name, s.item)
}

fn remake(s: &Spot) -> Option<bool> {
    (s.p.remake && crate::slice::remake_overworld::keeps_out(s.d, s.name, s.item)).then_some(false)
}

fn freestanding(s: &Spot) -> Option<bool> {
    if !s.p.world.on() {
        return None;
    }
    pu::at(s.d, s.name, s.item).or_else(|| pu::keeps_out(s.d, s.name, s.item).then_some(false))
}

fn holders(s: &Spot) -> Option<bool> {
    crate::pot_slice::rules::at(s.d, s.name, s.item)
}

fn skulls(s: &Spot) -> Option<bool> {
    if s.p.skulls == crate::slice::skulls::Mode::Off {
        return None;
    }
    crate::slice::skulls::accepts(s.d, s.w, s.it, s.l)
}

fn world_only(s: &Spot) -> Option<bool> {
    (pu::WORLD_ONLY.contains(&s.item) || s.d.pots.only.contains(s.item)).then_some(false)
}
