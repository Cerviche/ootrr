use crate::byml::{Byml, Node};

pub const SETTING: &str = "free_scarecrow";
pub const NUM: u16 = 154;
pub const LOGIC_KEY: &str = "scarecrow_behavior";
pub const LOGIC_ON: &str = "free";
pub const ACTOR: &str = "Npc_Pierre_CallOcarina";
pub const PREFIX: &str = "Appear_SCARECROW_";
pub const OCARINA: &str = "IsGet.Item_Ocarina_Fairy";
pub const COUNT: usize = 16;
pub const MARK: &[u8] = b"Appear_SCARECROW_";

pub const NOTE: &str = "Start with Scarecrow's Song: Pierre stands at all 16 of his hookshot spots as soon as you own an Ocarina, so there is no song to record or play. Each Npc_Pierre_CallOcarina that waited on its Appear_SCARECROW flag waits on IsGet.Item_Ocarina_Fairy instead, and the logic treats the Scarecrow's Song as known (OoTR's Free Scarecrow's Song). Bonooru and the adult Pierre at Lake Hylia still talk as usual. Experimental, untested in the game.";

pub fn patch(y: &mut Byml) -> usize {
    let Some(list) = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()) else {
        return 0;
    };
    let mut n: usize = 0;
    for a in list.iter_mut() {
        if a.get("Gyaml").and_then(|g| g.as_str()) != Some(ACTOR) {
            continue;
        }
        let Some(p) = a.get_mut("Presence") else {
            continue;
        };
        if p.get("FlagName").and_then(|f| f.as_str()).is_some_and(|f| f.starts_with(PREFIX)) {
            p.set("FlagName", Node::Str(OCARINA.to_string()));
            p.set("IsNegation", Node::Bool(false));
            n += 1;
        }
    }
    n
}
