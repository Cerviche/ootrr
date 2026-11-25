//! Builds the OOTRR mod of one Archipelago slot from its `.apootrr` patch. Shared by
//! `ootrr ap-build` and the app's Archipelago page.
//!
//! The other players' items are registered as extra items of the build data for the length of
//! one build only: [`Extras`] clears them (and the loose chest counts) when the build ends, so a
//! long-running app goes back to plain seed builds afterwards. One Archipelago build runs at a
//! time ([`LOCK`]), and [`running`] tells the app's plain builds to wait.

use std::path::Path as P;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::error::{Error as E, Res};
use crate::modpack::Host;
use crate::romfs::Romfs;
use crate::slice::data::Data;
use crate::slice::edit::Built;
use crate::staging::Out;

use super::item::Foreign;
use super::mailbox::Record;
use super::markers;
use super::patch::{self, Patch};

static LOCK: Mutex<()> = Mutex::new(());
static RUNNING: AtomicBool = AtomicBool::new(false);

/// True while an Archipelago build has its extra items registered.
pub fn running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

struct Extras;

impl Extras {
    fn on() -> Extras {
        RUNNING.store(true, Ordering::SeqCst);
        Extras
    }
}

impl Drop for Extras {
    fn drop(&mut self) {
        crate::slice::data::clear_extra_items();
        RUNNING.store(false, Ordering::SeqCst);
    }
}

/// What a patch holds, read without building.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub slot: String,
    pub seed_name: String,
    pub preset: String,
    pub settings: String,
    /// Shuffled locations of this slot.
    pub locations: usize,
    /// Locations of this slot that hold another player's item.
    pub foreign: usize,
    /// Items this slot receives from the other worlds, plus its starting items.
    pub receive: usize,
    /// The save flag a new game sets, which the client checks to find the right game.
    pub marker: String,
    /// The mod name used when none is given.
    pub mod_name: String,
}

pub fn summary(p: &Patch) -> Summary {
    Summary {
        slot: p.slot.clone(),
        seed_name: p.seed_name.clone(),
        preset: p.preset.clone(),
        settings: p.settings.clone(),
        locations: p.placements.len(),
        foreign: p.foreign.len(),
        receive: p.remote.len() + p.start.len(),
        marker: markers::seed_flag(&p.seed_key),
        mod_name: default_name(&p.slot),
    }
}

/// `ZZ OOTRR AP <slot>`, the slot made safe for a folder name.
pub fn default_name(slot: &str) -> String {
    let t: String = slot.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).take(24).collect();
    format!("ZZ OOTRR AP {}", if t.is_empty() { "slot" } else { t.as_str() })
}

/// Reads a patch's text.
pub fn read(text: &str) -> Res<Patch> {
    if text.len() > 64 << 20 {
        return Err(E::Msg("archipelago patch: the file is too large".to_string()));
    }
    Patch::parse(text)
}

/// Checks the patch against OOTRR's own logic and writes its mod to `load/<name>`.
pub fn build(p: &Patch, romfs: &P, load: &P, name: &str, host: Host) -> Res<(Built, Summary)> {
    let _one = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _extras: Extras = Extras::on();
    let foreign: Vec<Foreign> = p.foreign_list();
    let mut d: Data = Data::load()?;
    let before: usize = d.items.len();
    super::item::register(&mut d, &foreign)?;
    crate::slice::data::set_extra_items(d.items[before..].to_vec());
    let d: Data = Data::load()?;
    let c: patch::Checked = patch::check(&d, p)?;
    if !c.won {
        return Err(E::Msg(c.notes.join(" ")));
    }
    let recs: Vec<Record> = p.records(&d)?;
    crate::slice::data::set_extra_dungeon_items(
        recs.iter()
            .filter_map(|r| match &r.style {
                super::mailbox::Style::Dungeon(_, k, dg) => Some((*k, *dg)),
                _ => None,
            })
            .collect(),
    );
    let key: String = p.seed_key.clone();
    let mut hook = |rf: &mut Romfs, work: &P, outs: &mut Vec<Out>| -> Res<()> {
        super::stage(rf, outs, work, &key, &recs)?;
        super::item::stage(rf, work, outs, &foreign)
    };
    let mut b: Built = crate::slice::edit::build_for_with(&c.seed, romfs, load, name, host, Some(&mut hook))?;
    let s: Summary = summary(p);
    b.report.push_str(&format!(
        "Archipelago: slot {} of {}, {} items to receive, {} items for other players, seed marker {}\n",
        s.slot, s.seed_name, s.receive, foreign.len(), s.marker
    ));
    Ok((b, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_names_are_safe_folder_names() {
        assert_eq!(default_name("Link"), "ZZ OOTRR AP Link");
        assert_eq!(default_name("a/b c"), "ZZ OOTRR AP a_b_c");
        assert_eq!(default_name(""), "ZZ OOTRR AP slot");
        assert_eq!(default_name(&"x".repeat(40)).len(), "ZZ OOTRR AP ".len() + 24);
    }

    #[test]
    fn a_summary_counts_what_the_patch_holds() {
        let p: Patch = Patch::parse(
            r#"{"format":"ootrr-ap-patch","version":1,"game":"OOTRR","preset":"default","settings":"x","seed":"archipelago","seed_name":"1","player":1,"slot":"Link","seed_key":"0123456789ab",
            "placements":{"KF Midos Top Left Chest":"AP: 0123456789ab","KF Midos Top Right Chest":"Slingshot"},
            "foreign":{"KF Midos Top Left Chest":{"item":"Key","player":"Quester","game":"APQuest","progression":true}},
            "remote":[{"key":"aaaaaaaaaaaa","item":"Boomerang","player":"Quester","location":"Top Left Chest"}],"start":[],"location_ids":{}}"#,
        )
        .unwrap();
        let s: Summary = summary(&p);
        assert_eq!((s.locations, s.foreign, s.receive), (2, 1, 1));
        assert_eq!(s.mod_name, "ZZ OOTRR AP Link");
        assert_eq!(s.marker, markers::seed_flag("0123456789ab"));
    }
}
