
use std::collections::BTreeSet as Set;

use super::model::Schema;
use crate::json::Json;

pub struct Section {
    pub label: &'static str,
    pub ids: &'static [&'static str],
}

pub struct Page {
    pub id: &'static str,
    pub label: &'static str,
    pub heading: &'static str,
    pub sections: &'static [Section],
}

const fn sec(label: &'static str, ids: &'static [&'static str]) -> Section {
    Section { label, ids }
}

pub const RANDOMIZER: &str = "Randomizer";
pub const GAMEPLAY: &str = "Gameplay";

pub const MENU: &[Page] = &[
    Page {
        id: "world",
        label: "World",
        heading: RANDOMIZER,
        sections: &[
            sec("Logic", &["logic_rules", "bombchus_in_logic"]),
            sec("Start", &["starting_age"]),
            sec(
                "Open areas",
                &["open_forest", "open_kakariko", "open_door_of_time", "zora_fountain", "gerudo_fortress", "jabu_open", "gerudo_bridge_open"],
            ),
            sec("Entrances", &["shuffle_entrances"]),
        ],
    },
    Page {
        id: "locations",
        label: "Locations",
        heading: RANDOMIZER,
        sections: &[
            sec("Chests", &["shuffle_chests", "shuffle_remake_chests"]),
            sec(
                "Pots, crates and grass",
                &["shuffle_pots", "shuffle_remake_pots", "shuffle_crates", "shuffle_rocks", "shuffle_grass", "shuffle_beehives"],
            ),
            sec("Ground items and tokens", &["shuffle_freestanding_items", "shuffle_skulltulas"]),
        ],
    },
    Page {
        id: "npcs",
        label: "NPCs and rewards",
        heading: RANDOMIZER,
        sections: &[
            sec(
                "Rewards",
                &["shuffle_rewards", "shuffle_rewards_extra", "shuffle_dungeon_rewards", "shuffle_boss_hearts", "shuffle_treasure_reward", "shuffle_songs"],
            ),
            sec("Business Scrubs", &["shuffle_scrubs", "shuffle_scrubs_a", "shuffle_scrubs_b"]),
            sec("Zora's River frogs", &["shuffle_frog_songs", "shuffle_frog_bug_heart"]),
            sec("Other", &["shuffle_trade_checks", "shuffle_potion_shop", "shuffle_cows", "shuffle_cows_other"]),
        ],
    },
    Page {
        id: "shops",
        label: "Shops",
        heading: RANDOMIZER,
        sections: &[
            sec("Shopsanity", &["shuffle_shops", "shopsanity_prices"]),
        ],
    },
    Page {
        id: "dungeon",
        label: "Dungeon items",
        heading: RANDOMIZER,
        sections: &[
            sec("Keys and maps", &["shuffle_smallkeys", "shuffle_bosskeys", "shuffle_mapcompass", "shuffle_hideoutkeys"]),
            sec("Silver rupees", &["shuffle_silver_rupees"]),
            sec(
                "Ganon's Boss Key",
                &["shuffle_ganon_bosskey", "ganon_bosskey_stones", "ganon_bosskey_medallions", "ganon_bosskey_rewards", "ganon_bosskey_tokens", "ganon_bosskey_hearts"],
            ),
        ],
    },
    Page {
        id: "goals",
        label: "Goals",
        heading: RANDOMIZER,
        sections: &[
            sec("Rainbow Bridge", &["bridge", "bridge_stones", "bridge_medallions", "bridge_rewards", "bridge_tokens", "bridge_hearts"]),
            sec("Ganon's Trials", &["trials_random", "trials"]),
            sec("Triforce Hunt", &["triforce_hunt", "triforce_count_per_world", "triforce_goal_per_world"]),
        ],
    },
    Page {
        id: "starting",
        label: "Starting inventory",
        heading: RANDOMIZER,
        sections: &[sec("Start", &["starting_hearts", "start_reward"]), sec("Songs", &["starting_songs"]), sec("Items", &["starting_items"])],
    },
    Page { id: "excluded", label: "Excluded locations", heading: RANDOMIZER, sections: &[sec("Excluded locations", &["disabled_locations"])] },
    Page {
        id: "timesavers",
        label: "Time savers",
        heading: GAMEPLAY,
        sections: &[
            sec("Cutscenes", &["skip_intro", "skip_cutscenes", "skip_story_scenes", "faster_chests"]),
            sec("Side quests", &["skip_epona_race", "skip_mask_quest", "anju_one_cucco", "big_poe_one", "plant_beans"]),
            sec("Dungeons", &["deku_bridge_safe"]),
        ],
    },
    Page {
        id: "tweaks",
        label: "Tweaks",
        heading: GAMEPLAY,
        sections: &[sec("Time of day", &["time_always_passes", "early_beds"]), sec("Link and Navi", &["navi_return", "bunny_hood"])],
    },
    Page { id: "hints", label: "Hints", heading: GAMEPLAY, sections: &[sec("Hints", &["gossip_hints", "chests_match_contents", "misc_hints"])] },
];

pub fn problems(sch: &Schema) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut placed: Set<&str> = Set::new();
    let mut pages: Set<&str> = Set::new();
    for p in MENU {
        if !pages.insert(p.id) {
            out.push(format!("menu page {} is listed twice", p.id));
        }
        for s in p.sections {
            for id in s.ids {
                if !placed.insert(id) {
                    out.push(format!("setting {id} is placed in the menu more than once"));
                }
            }
        }
    }
    for s in &sch.settings {
        if !placed.contains(s.id.as_str()) {
            out.push(format!("setting {} has no place in the settings menu (settings/menu.rs)", s.id));
        }
    }
    out
}

pub fn menu_json(sch: &Schema) -> Json {
    let have: Set<&str> = sch.settings.iter().map(|s| s.id.as_str()).collect();
    let mut placed: Set<&str> = Set::new();
    let mut out: Vec<Json> = Vec::new();
    for p in MENU {
        let mut secs: Vec<Json> = Vec::new();
        for s in p.sections {
            let ids: Vec<&str> = s.ids.iter().copied().filter(|x| have.contains(x) && placed.insert(x)).collect();
            if !ids.is_empty() {
                secs.push(Json::obj().with("label", s.label.into()).with("ids", Json::strs(ids.into_iter())));
            }
        }
        if !secs.is_empty() {
            out.push(page_json(p.id, p.label, p.heading, secs));
        }
    }
    let left: Vec<&str> = sch.settings.iter().map(|s| s.id.as_str()).filter(|x| !placed.contains(x)).collect();
    if !left.is_empty() {
        let sec: Json = Json::obj().with("label", "Not placed yet".into()).with("ids", Json::strs(left.into_iter()));
        out.push(page_json("other", "Other", RANDOMIZER, vec![sec]));
    }
    Json::Arr(out)
}

fn page_json(id: &str, label: &str, heading: &str, secs: Vec<Json>) -> Json {
    Json::obj().with("id", id.into()).with("label", label.into()).with("heading", heading.into()).with("sections", Json::Arr(secs))
}

#[cfg(test)]
mod tests {
    use super::super::schema;
    use super::*;

    #[test]
    fn every_setting_has_exactly_one_place_in_the_menu() {
        assert_eq!(problems(schema()), Vec::<String>::new());
    }

    #[test]
    fn exported_menu_lists_each_setting_once_and_skips_unknown_ids() {
        let sch: &Schema = schema();
        let j: Json = menu_json(sch);
        let mut seen: Vec<String> = Vec::new();
        for p in j.as_arr().unwrap() {
            for s in p.get("sections").and_then(|x| x.as_arr()).unwrap() {
                for id in s.get("ids").and_then(|x| x.as_arr()).unwrap() {
                    seen.push(id.as_str().unwrap().to_string());
                }
            }
        }
        let mut want: Vec<String> = sch.settings.iter().map(|s| s.id.clone()).collect();
        seen.sort();
        want.sort();
        assert_eq!(seen, want);
    }
}
