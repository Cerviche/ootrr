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

pub const RULES: [(&str, Rule); 8] = [
    ("shops", shops),
    ("song spots", songs),
    ("treasure chest game", treasure),
    ("remake overworld chests", remake),
    ("freestanding pickups", freestanding),
    ("pots, crates and other holders", holders),
    ("gold skulltulas", skulls),
    ("world only pickups", world_only),
];

pub fn verdict(s: &Spot) -> Option<bool> {
    RULES.iter().find_map(|(_, r)| r(s))
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
