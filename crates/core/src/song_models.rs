use crate::byml::{Byml, Node};
use crate::pickup_pack::model::ROW_KEYS;
use crate::pickup_pack::rows::{self, ACTOR_INFO};
use crate::error::{Error, Res};
use crate::romfs::Romfs;
use crate::sarc;
use crate::staging::{rewrite, Out};

pub const MODEL: &str = "Item_Harp_Sheik";
pub const DONOR: &str = crate::pickup_pack::model::HARP;
pub const SONGS: [&str; 12] = [
    "ZeldaLullaby",
    "EponaSong",
    "SariaSong",
    "SunSong",
    "TimeSong",
    "StormSong",
    "ForestMinuet",
    "FireBolero",
    "WaterSerenade",
    "SpiritRequiem",
    "ShadowNocturne",
    "LightPrelude",
];

pub fn pack_of(song: &str) -> String {
    format!("Pack/Actor/Item_Melody_{song}.pack.zs")
}

pub fn info_of(song: &str) -> String {
    format!("Component/ModelInfo/Item_Melody_{song}.engine__component__ModelInfo.bgyml")
}

pub fn repoint(raw: &[u8], file: &str, model: &str) -> Res<Vec<u8>> {
    let mut es: Vec<sarc::Entry> = sarc::parse(raw)?;
    let e: &sarc::Entry = sarc::get(&es, file).ok_or(Error::Format("song models: the pack has no model info"))?;
    let mut y: Byml = Byml::parse(&e.data)?;
    for k in ["FmdbName", "ModelProjectName"] {
        if !y.root.set(k, Node::Str(model.to_string())) {
            return Err(Error::Format("song models: the model info lacks a model name"));
        }
    }
    sarc::set(&mut es, file, y.write()?);
    sarc::write(&es)
}

pub fn rows_of(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    let donor: Node = rows::need(&y, DONOR, ACTOR_INFO)?.clone();
    if donor.get("FmdbName").and_then(|v| v.as_str()) != Some(MODEL) {
        return Err(Error::Format("song models: the harp row names another model, the game files differ"));
    }
    let list: &mut Vec<Node> = y.root.as_arr_mut().ok_or(Error::Format("song models: the actor table is not a row list"))?;
    for s in SONGS {
        let id: String = format!("Item_Melody_{s}");
        let r: &mut Node = list.iter_mut().find(|r| rows::id(r) == Some(id.as_str())).ok_or(Error::Format("song models: a song has no actor row"))?;
        for k in ROW_KEYS {
            if let Some(v) = donor.get(k) {
                r.set(k, v.clone());
            }
        }
    }
    y.write()
}

pub fn stage(rf: &mut Romfs) -> Res<Vec<Out>> {
    let mut outs: Vec<Out> = Vec::new();
    for s in SONGS {
        let file: String = info_of(s);
        outs.push(rewrite(rf, &pack_of(s), |raw| repoint(raw, &file, MODEL))?);
    }
    outs.push(rewrite(rf, ACTOR_INFO, rows_of)?);
    Ok(outs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(m: &str) -> Vec<u8> {
        let y: Byml = Byml {
            root: Node::Map(vec![
                ("FmdbName".to_string(), Node::Str(m.to_string())),
                ("ModelProjectName".to_string(), Node::Str(m.to_string())),
            ]),
            version: 2,
            big: false,
        };
        y.write().unwrap()
    }

    #[test]
    fn the_song_rows_take_the_harp_model_fields() {
        let mut list: Vec<Node> = Vec::new();
        let mut harp: Node = Node::Map(Vec::new());
        for k in ROW_KEYS {
            harp.set(k, Node::Str(format!("harp {k}")));
        }
        harp.set("FmdbName", Node::Str(MODEL.to_string()));
        harp.set(rows::ROW_ID, Node::Str(DONOR.to_string()));
        list.push(harp);
        for s in SONGS {
            let mut r: Node = Node::Map(Vec::new());
            r.set("FmdbName", Node::Str("Obj_Rupee".to_string()));
            r.set("ActorName", Node::Str(format!("Item_Melody_{s}")));
            r.set(rows::ROW_ID, Node::Str(format!("Item_Melody_{s}")));
            list.push(r);
        }
        let raw: Vec<u8> = Byml { root: Node::Arr(list), version: 7, big: false }.write().unwrap();
        let y: Byml = Byml::parse(&rows_of(&raw).unwrap()).unwrap();
        let r: &Node = rows::need(&y, "Item_Melody_SunSong", ACTOR_INFO).unwrap();
        assert_eq!(r.get("FmdbName").and_then(|v| v.as_str()), Some(MODEL));
        assert_eq!(r.get("ModelAabbMax").and_then(|v| v.as_str()), Some("harp ModelAabbMax"));
        assert_eq!(r.get("ActorName").and_then(|v| v.as_str()), Some("Item_Melody_SunSong"));
        assert!(rows_of(&Byml { root: Node::Arr(Vec::new()), version: 7, big: false }.write().unwrap()).is_err());
    }

    #[test]
    fn the_model_names_change_and_the_rest_of_the_pack_stays() {
        let f: String = info_of("SariaSong");
        let es: Vec<sarc::Entry> = vec![
            sarc::Entry { name: f.clone(), data: info("Obj_Rupee") },
            sarc::Entry { name: "Other/File.bin".to_string(), data: vec![1, 2, 3] },
        ];
        let raw: Vec<u8> = sarc::write(&es).unwrap();
        let out: Vec<u8> = repoint(&raw, &f, MODEL).unwrap();
        let back: Vec<sarc::Entry> = sarc::parse(&out).unwrap();
        let y: Byml = Byml::parse(&sarc::get(&back, &f).unwrap().data).unwrap();
        assert_eq!(y.root.get("FmdbName").and_then(|v| v.as_str()), Some(MODEL));
        assert_eq!(y.root.get("ModelProjectName").and_then(|v| v.as_str()), Some(MODEL));
        assert_eq!(sarc::get(&back, "Other/File.bin").unwrap().data, vec![1, 2, 3]);
        assert!(repoint(&raw, "missing", MODEL).is_err());
    }
}
