use crate::bfevfl::{Bf, Event, Flow, Val};
use crate::byml::{Byml, Node};
use crate::error::{Error, Res};

pub const FLOW: &str = "MiniGame_DampeRace";
pub const STATE: &str = "Minigame_DampeRace_TreasureBoxState";
/// The map of Dampé's grave race, where the race goal and its chests are.
pub const GRAVE: &str = "Banc/Graveyard/Graveyard_Adult_Static.bcett.byml.zs";

/// The Heart Piece of the race (Graveyard Dampe Race Freestanding PoH) lies on the ground at the
/// goal: with Dampé (DampePos) and the goal chest it makes a triangle, mirrored from Dampé across
/// the chest on the floor. Copied from the Graveyard's own ground Heart Piece.
pub const HEART: u64 = 7277816997830770001;
pub const HEART_AT: [f32; 3] = [-3.157, 82.6, 75.47];
const HEART_FROM: u64 = 2157466067026784951;

/// A Song of Time block that only stands there as a platform: no Song of Time group, no link.
/// Placed in the race course where the owner stood, raised a little.
pub const BLOCK: u64 = 7277816997830770002;
pub const BLOCK_AT: [f32; 3] = [-21.9615, 84.1, 77.3986];
const BLOCK_ACTOR: &str = "Obj_SongOfTimeBlock_A_01";

/// The grave's dynamic map, which holds the glowing floor boards of the goal room.
pub const GRAVE_DYNAMIC: &str = "Banc/Graveyard/Graveyard_Adult_Dynamic.bcett.byml.zs";
/// The two glowing broken boards (Land_DampeGrave_Floor_A_01) by the ground Heart Piece. Nothing
/// links to them; they go so the item does not sit among them.
pub const BOARDS: [u64; 2] = [12927524056370954305, 4234627562618720374];

pub const NOTE: &str = "One Dampé race: the first race gives the Hookshot chest as before, and the race's Heart Piece lies on the ground at the goal, across from Dampé, with no second race to win. A fast second race shows the rupee chest instead of the Heart Piece chest. A Song of Time block stands in the race course as an extra platform. Always on.";

fn named(f: &Flow, name: &str) -> Res<usize> {
    let hits: Vec<usize> = (0..f.events.len()).filter(|&i| f.events[i].name == name).collect();
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("one dampe race: expected one {name} in {FLOW}, found {}", hits.len()))),
    }
}

fn param<'a>(e: &'a Event, k: &str) -> Option<&'a Val> {
    e.kind.params().and_then(|c| c.get(k))
}

fn is(e: &Event, k: &str, v: &str) -> bool {
    param(e, k) == Some(&Val::Str(v.to_string()))
}

/// A fast second race (Event364, TreasureBoxState = AppearHeartPiece) shows the rupee chest:
/// the Heart Piece is on the ground now, and its chest never appears.
pub fn patch(b: &mut Bf) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|f| f.name == FLOW).ok_or(Error::Format("one dampe race: MiniGame_DampeRace has no flow of its name"))?;
    let heart: usize = named(f, "Event364")?;
    if !(is(&f.events[heart], "GameDataName", STATE) && is(&f.events[heart], "Value", "AppearHeartPiece")) {
        return Err(Error::Format("one dampe race: Event364 no longer shows the Heart Piece chest"));
    }
    if let Some(c) = f.events[heart].kind.params_mut() {
        for (k, v) in c.items.iter_mut() {
            if k == "Value" {
                *v = Val::Str("AppearRupee".to_string());
            }
        }
    }
    Ok(())
}

fn at(p: [f32; 3]) -> Node {
    Node::Arr(p.iter().map(|x| Node::F32(*x)).collect())
}

fn actor(gyaml: &str, hash: u64, pos: [f32; 3], rot: Node, srt: u32) -> Node {
    Node::Map(vec![
        ("Gyaml".to_string(), Node::Str(gyaml.to_string())),
        ("Hash".to_string(), Node::U64(hash)),
        ("Phive".to_string(), Node::Map(vec![("Placement".to_string(), Node::Map(vec![("ID".to_string(), Node::U64(hash))]))])),
        ("Rotate".to_string(), rot),
        ("SRTHash".to_string(), Node::U32(srt)),
        ("Translate".to_string(), at(pos)),
    ])
}

/// Takes the glowing floor boards by the ground Heart Piece out of the grave's dynamic map.
pub fn clear(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()).ok_or(Error::Format("one dampe race: the grave dynamic map has no actors"))?;
    let before: usize = list.len();
    list.retain(|a| !matches!(a.get("Hash"), Some(Node::U64(h)) if BOARDS.contains(h)));
    if before - list.len() != BOARDS.len() {
        return Err(Error::Msg(format!("one dampe race: found {} of the {} floor boards, the game files differ", before - list.len(), BOARDS.len())));
    }
    y.write()
}

/// Adds the ground Heart Piece and the Song of Time block platform to the grave race map.
pub fn place(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let list: &mut Vec<Node> = y.root.get_mut("Actors").and_then(|a| a.as_arr_mut()).ok_or(Error::Format("one dampe race: the grave map has no actors"))?;
    let hash = |a: &Node| match a.get("Hash") {
        Some(Node::U64(h)) => Some(*h),
        _ => None,
    };
    if list.iter().any(|a| matches!(hash(a), Some(h) if h == HEART || h == BLOCK)) {
        return Err(Error::Format("one dampe race: the grave map already holds the new actors"));
    }
    let from: &Node = list.iter().find(|a| hash(a) == Some(HEART_FROM)).ok_or(Error::Format("one dampe race: the Graveyard ground Heart Piece is gone, the game files differ"))?;
    let rot: Node = from.get("Rotate").cloned().unwrap_or_else(|| at([0.0, 0.0, 0.0]));
    let srt: u32 = match from.get("SRTHash") {
        Some(Node::U32(s)) => *s,
        _ => 0,
    };
    list.push(actor("Obj_Heart_Piece", HEART, HEART_AT, rot, srt));
    list.push(actor(BLOCK_ACTOR, BLOCK, BLOCK_AT, at([0.0, 0.0, 0.0]), srt));
    y.write()
}
