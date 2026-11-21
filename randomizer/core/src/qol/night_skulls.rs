use crate::byml::{Byml, Node};

pub const ALWAYS: &str = "gold_skulls_always";
pub const ALWAYS_NUM: u16 = 151;
pub const SUNS: &str = "night_skulls_suns_song";
pub const SUNS_NUM: u16 = 152;
pub const LOGIC_ALWAYS: &str = "gold_skulls_ignore_daytime";
pub const LOGIC_SUNS: &str = "night_skulls_suns_song";
pub const SKULL: &str = "Enemy_Skulltula_Gold";
pub const NIGHT: &str = "Enemy_Skulltula_Gold_Nocturnal";
pub const COUNT: usize = 30;
pub const MARK: &[u8] = b"Enemy_Skulltula_Gold_Nocturnal";

pub const NOTE_ALWAYS: &str = "The Gold Skulltulas that only come out at night are there day and night. Their maps drop Enemy_Skulltula_Gold_Nocturnal from all 30 of them, so they are placed like the day ones, and the logic no longer needs night for them. Experimental, untested in the game.";
pub const NOTE_SUNS: &str = "The logic expects Sun's Song for every Gold Skulltula that only comes out at night, so no check needs one of them before you can play Sun's Song. Has no effect when night Gold Skulltulas are there all day.";

pub fn patch(y: &mut Byml) -> usize {
    let Some(list) = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()) else {
        return 0;
    };
    let mut n: usize = 0;
    for a in list.iter_mut() {
        if a.get("Gyaml").and_then(|g| g.as_str()) != Some(SKULL) {
            continue;
        }
        let Some(d) = a.get_mut("Dynamic") else {
            continue;
        };
        if d.get(NIGHT) != Some(&Node::Bool(true)) {
            continue;
        }
        d.remove(NIGHT);
        n += 1;
        if matches!(d, Node::Map(m) if m.is_empty()) {
            a.remove("Dynamic");
        }
    }
    n
}
