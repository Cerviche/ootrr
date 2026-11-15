use crate::settings::Settings;
use crate::slice::support::Mode;

pub const ROCKS: &str = "shuffle_rocks";
pub const GRASS: &str = "shuffle_grass";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drops {
    pub pots: Mode,
    pub crates: Mode,
    pub hives: Mode,
    pub rocks: Mode,
    pub grass: Mode,
    pub remake: Mode,
}

pub const HIVES: &str = "shuffle_beehives";

fn mode_of(s: &Settings, id: &str, bad: &mut Vec<String>) -> Mode {
    match s.text(id) {
        Ok("off") => Mode::Off,
        Ok("overworld") => Mode::Overworld,
        Ok("dungeons") => Mode::Dungeons,
        Ok("all") => Mode::All,
        Ok(v) => {
            bad.push(format!("{id} is {v}, which is not a known choice"));
            Mode::Off
        }
        Err(e) => {
            bad.push(e.to_string());
            Mode::Off
        }
    }
}

pub fn read(s: &Settings, bad: &mut Vec<String>) -> Drops {
    let hives: Mode = match s.flag(HIVES) {
        Ok(true) => Mode::All,
        Ok(false) => Mode::Off,
        Err(e) => {
            bad.push(e.to_string());
            Mode::Off
        }
    };
    Drops {
        pots: mode_of(s, "shuffle_pots", bad),
        crates: mode_of(s, "shuffle_crates", bad),
        hives,
        rocks: mode_of(s, ROCKS, bad),
        grass: mode_of(s, GRASS, bad),
        remake: mode_of(s, crate::pot_slice::remake::SETTING, bad),
    }
}

impl Drops {
    pub fn off() -> Drops {
        Drops { pots: Mode::Off, crates: Mode::Off, hives: Mode::Off, rocks: Mode::Off, grass: Mode::Off, remake: Mode::Off }
    }

    pub fn any(&self) -> bool {
        self.pots != Mode::Off || self.crates != Mode::Off || self.hives != Mode::Off
    }

    pub fn wild(&self) -> bool {
        self.rocks != Mode::Off || self.grass != Mode::Off
    }

    pub fn notes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if self.pots != Mode::Off {
            out.push("Pots and flying pots hold shuffled items, only the pots whose spot was matched with confidence and only items the game ships as world pickups or whose pickup pack is built. A shuffled pot gives its item once and never refills, its saved row is made permanent. Experimental and untested in the game.".to_string());
        }
        if self.crates != Mode::Off {
            out.push("Crates hold shuffled items, only the few crates matched with confidence. A shuffled crate gives its item once and never refills. Experimental and untested in the game.".to_string());
        }
        if self.hives != Mode::Off {
            out.push("The seven beehives in the covered hole of Kakariko Village hold shuffled items, only refill items such as rupees and ammo. A shuffled beehive gives its item once and never refills, its saved row is made permanent. The remake has no other beehive that drops an item. Experimental and untested in the game.".to_string());
        }
        if self.rocks != Mode::Off {
            out.push("Small liftable rocks hold shuffled items. The adult and the child copy of a rock are two checks. A rock takes only rupees, hearts, ammo and other refills, because a thrown rock can drop its item where it cannot be reached. A shuffled rock gives its item once, its saved row is made permanent. Experimental and untested in the game.".to_string());
        }
        if self.grass != Mode::Off {
            out.push("Grass tufts hold shuffled items. The adult and the child copy of a tuft are two checks. Grass takes only rupees, hearts, ammo and other refills. A shuffled tuft gives its item once and stays cut. Bug grass in the Lost Woods is left alone. Experimental and untested in the game.".to_string());
        }
        if self.remake != Mode::Off {
            out.push("19 pots that only the remake has hold shuffled items, in Dodongo's Cavern, the Fire, Forest, Shadow and Water Temples, the Ice Cavern and, as an adult, the Gerudo Valley storms grotto. They take only rupees, hearts, ammo and other refills, because their logic region comes from the room data and was never checked in the game. A shuffled pot gives its item once, its saved row is made permanent. Experimental and untested in the game.".to_string());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Value;

    #[test]
    fn the_defaults_are_off_and_every_mode_reads() {
        let mut bad: Vec<String> = Vec::new();
        let s: Settings = Settings::default();
        assert_eq!(read(&s, &mut bad), Drops::off());
        assert!(!read(&s, &mut bad).any());
        assert!(bad.is_empty());
        let mut t: Settings = Settings::default();
        t.set("shuffle_pots", Value::s("dungeons")).unwrap();
        t.set("shuffle_crates", Value::s("overworld")).unwrap();
        let d: Drops = read(&t, &mut bad);
        assert_eq!((d.pots, d.crates), (Mode::Dungeons, Mode::Overworld));
        assert_eq!(d.hives, Mode::Off);
        assert_eq!(d.notes().len(), 2);
        t.set(HIVES, Value::Bool(true)).unwrap();
        let h: Drops = read(&t, &mut bad);
        assert_eq!(h.hives, Mode::All);
        assert!(h.any() && h.notes().len() == 3);
        let mut only: Settings = Settings::default();
        only.set(HIVES, Value::Bool(true)).unwrap();
        assert!(read(&only, &mut bad).any());
        assert!(bad.is_empty());
        assert!(Drops::off().notes().is_empty());
        assert!(!d.wild());
    }

    #[test]
    fn rocks_and_grass_read_on_their_own() {
        let mut bad: Vec<String> = Vec::new();
        let mut t: Settings = Settings::default();
        t.set(ROCKS, Value::s("all")).unwrap();
        let d: Drops = read(&t, &mut bad);
        assert_eq!((d.rocks, d.grass, d.pots), (Mode::All, Mode::Off, Mode::Off));
        assert!(!d.any() && d.wild());
        assert_eq!(d.notes().len(), 1);
        t.set(GRASS, Value::s("dungeons")).unwrap();
        let e: Drops = read(&t, &mut bad);
        assert_eq!(e.grass, Mode::Dungeons);
        assert_eq!(e.notes().len(), 2);
        assert!(bad.is_empty());
    }
}
