//! The Archipelago page: read an `.apootrr` patch, then build its mod with the folders from the
//! Configuration page. The folder checks are the seed build's own ([`api_build::place`]).

use std::path::PathBuf as PB;
use std::sync::Arc;

use ootrr_core::ap::build::{self as ap_build, Summary};
use ootrr_core::ap::patch::Patch;
use ootrr_core::json::Json;

use crate::api_build::{self, Place};
use crate::body;
use crate::error::{Fail, Hr};
use crate::game_source::{self, Prepared};
use crate::jobs::{Ctl, Jobs};
use crate::pathsafe;

fn refuse(m: &str) -> Fail {
    Fail::new(422, m)
}

fn patch_of(b: &Json) -> Hr<Patch> {
    ap_build::read(body::text(b, "patch_text")?).map_err(|e| refuse(&format!("Patch file: {e}")))
}

fn summary_json(s: &Summary) -> Json {
    Json::obj()
        .with("slot", s.slot.as_str().into())
        .with("seed_name", s.seed_name.as_str().into())
        .with("preset", s.preset.as_str().into())
        .with("settings_string", s.settings.as_str().into())
        .with("locations", Json::Int(s.locations as i64))
        .with("foreign", Json::Int(s.foreign as i64))
        .with("receive", Json::Int(s.receive as i64))
        .with("marker", s.marker.as_str().into())
        .with("mod_name", s.mod_name.as_str().into())
}

/// `POST /api/ap/read {patch_text}`: what the patch holds, without building.
pub fn read(b: &Json) -> Hr<Json> {
    let p: Patch = patch_of(b)?;
    Ok(summary_json(&ap_build::summary(&p)))
}

/// The note left in the mod folder as Spoiler.txt, so "Delete all other seeds" treats an
/// Archipelago mod like a seed.
fn note(s: &Summary) -> String {
    format!(
        "OOTRR Archipelago mod\nSlot: {}\nArchipelago seed: {}\nPreset: {}\nSettings string: {}\n\nThe spoiler of an Archipelago game is on the Archipelago server.\n",
        s.slot, s.seed_name, s.preset, s.settings
    )
}

/// `POST /api/ap/build {patch_text, game_dir, update_dir, include_update, mod_dir, mod_name, host, delete_others, title_logo}`.
pub fn build(jobs: &Arc<Jobs>, b: &Json) -> Hr<Json> {
    let p: Patch = patch_of(b)?;
    let pl: Place = api_build::place(b)?;
    if jobs.running() > 0 || ap_build::running() {
        return Err(Fail::new(409, "Another build is running. Wait for it to finish, then build again."));
    }
    let title: bool = b.get("title_logo").and_then(|x| x.as_bool()) != Some(false);
    let target: String = pathsafe::text(&pl.target);
    let n: usize = p.placements.len();
    let id: u64 = jobs.start(move |c: &Ctl| {
        c.begin(&format!("Patch read, {n} shuffled locations for slot {}", p.slot));
        c.ok(5);
        if let Some(s) = pl.source.step() {
            c.begin(s);
        }
        let game: Prepared = match game_source::prepare(&pl.source, false) {
            Ok(g) => g,
            Err(e) => return fail(c, &e),
        };
        c.ok(8);
        let romfs: PB = game.romfs.clone();
        if title {
            c.begin("Check that no other mod changes the title screen");
            if let Err(e) = ootrr_core::title_logo::preflight(&pl.target, &romfs) {
                return fail(c, &e.to_string());
            }
            c.ok(10);
        }
        c.begin("Check the world with OOTRR's logic and build the mod");
        let (mut built, s) = match ap_build::build(&p, &romfs, &pl.load, &pl.name, pl.host) {
            Ok(x) => x,
            Err(e) => return fail(c, &e.to_string()),
        };
        c.ok(80);
        if title {
            c.begin("Draw the Randomizer line under the title logo");
            match ootrr_core::title_logo::apply(&built.root, &romfs) {
                Ok(notes) => {
                    for n in notes {
                        built.report.push_str(&format!("{n}\n"));
                    }
                }
                Err(e) => return fail(c, &e.to_string()),
            }
        }
        c.ok(90);
        c.begin("Write the slot note");
        if let Err(e) = api_build::put_spoiler(&built.root, Some(&note(&s))) {
            return fail(c, &format!("The mod was written, but {} could not be: {e}", api_build::SPOILER_FILE));
        }
        if pl.with_update {
            c.begin("Add the v1.1.1 update files to the mod");
            match game_source::add_update(&pl.source, &built.root) {
                Ok(n) => built.report.push_str(&format!("Added {n} v1.1.1 update files to the mod. Do not share this mod: it now holds game files.\n")),
                Err(e) => return fail(c, &format!("The mod was written, but the v1.1.1 update files could not be added: {e}")),
            }
        }
        if pl.delete_others {
            c.begin("Delete the other seeds in the mod folder");
            match api_build::delete_others(&built.root) {
                Ok(gone) if gone.is_empty() => built.report.push_str("No other seeds to delete.\n"),
                Ok(gone) => built.report.push_str(&format!("Deleted {} other seeds: {}.\n", gone.len(), gone.join(", "))),
                Err(e) => return fail(c, &format!("The mod was written, but the other seeds could not all be deleted: {e}")),
            }
        }
        c.ok(95);
        c.begin("Mod files written");
        c.ok(100);
        c.finish("done", &built.report);
    })?;
    Ok(Json::obj()
        .with("job_id", Json::Int(id as i64))
        .with("target", target.into())
        .with("writes_files", true.into())
        .with("notice", "Building the Archipelago mod. Only the mod folder is written, the game folder is only read.".into()))
}

fn fail(c: &Ctl, m: &str) {
    c.mark("failed");
    c.finish("failed", m);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &str = r#"{"format":"ootrr-ap-patch","version":1,"game":"OOTRR","preset":"default","settings":"x","seed":"archipelago","seed_name":"1","player":1,"slot":"Link","seed_key":"0123456789ab",
        "placements":{"KF Midos Top Left Chest":"AP: 0123456789ab","KF Midos Top Right Chest":"Slingshot"},
        "foreign":{"KF Midos Top Left Chest":{"item":"Key","player":"Quester","game":"APQuest","progression":true}},
        "remote":[{"key":"aaaaaaaaaaaa","item":"Boomerang","player":"Quester","location":"Top Left Chest"}],"start":[],"location_ids":{}}"#;

    #[test]
    fn reading_a_patch_names_its_slot_and_default_mod() {
        let j: Json = read(&Json::obj().with("patch_text", SMALL.into())).unwrap();
        assert_eq!(j.get("slot").and_then(|x| x.as_str()), Some("Link"));
        assert_eq!(j.get("mod_name").and_then(|x| x.as_str()), Some("ZZ OOTRR AP Link"));
        assert_eq!(j.get("receive").and_then(|x| x.as_i64()), Some(1));
        assert_eq!(j.get("foreign").and_then(|x| x.as_i64()), Some(1));
    }

    #[test]
    fn a_bad_patch_or_bad_folders_are_refused_before_a_job_starts() {
        assert_eq!(read(&Json::obj().with("patch_text", "{}".into())).unwrap_err().status, 422);
        assert!(read(&Json::obj()).is_err());
        let jobs: Arc<Jobs> = Jobs::new();
        let nowhere: String = std::env::temp_dir().join("ootrr_ap_no_such_game").display().to_string();
        let body: Json = Json::obj()
            .with("patch_text", SMALL.into())
            .with("game_dir", nowhere.as_str().into())
            .with("mod_dir", nowhere.as_str().into())
            .with("mod_name", "x".into());
        assert!(build(&jobs, &body).is_err());
        assert_eq!(jobs.running(), 0);
    }

    #[test]
    fn the_note_names_the_slot_and_where_the_spoiler_is() {
        let p: Patch = Patch::parse(SMALL).unwrap();
        let t: String = note(&ap_build::summary(&p));
        assert!(t.contains("Slot: Link") && t.contains("Archipelago server"));
    }
}
