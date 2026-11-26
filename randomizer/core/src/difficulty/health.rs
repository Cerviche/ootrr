use crate::difficulty::{NORMAL, Opts, times};
use crate::settings::model::Setting;

pub const BOSS: &str = "boss_health";
pub const MINI: &str = "miniboss_health";

pub fn setting(num: u16, id: &str, label: &str) -> Setting {
    let mut out: Setting = Setting::choice(num, id, "difficulty", label, "follow_enemy")
        .opt("follow_enemy", "Follow enemy health", "Use the Enemy health multiplier, including in older presets.")
        .tip("Choose an independent health multiplier from 0.5x to 5x, or follow Enemy health. Health tops out at 99. Scripted health is unchanged. Experimental, untested in the game.");
    for n in (50..=500).step_by(25) {
        out = out.opt(&format!("pct_{n}"), &format!("{}x", times(n)), "");
    }
    out
}

pub fn percent(text: &str) -> i64 {
    text.strip_prefix("pct_").unwrap_or("").parse::<i64>().ok().filter(|n| (50..=500).contains(n) && n % 25 == 0).unwrap_or(0)
}

pub fn multiplier(stem: &str, mini_lizalfos: bool, o: &Opts) -> i64 {
    let n: i64 = if stem.starts_with("Enemy_DungeonBoss_") {
        o.boss_health
    } else if stem.starts_with("Enemy_MiddleBoss_") || stem.starts_with("Enemy_IronKnuckle") || stem.starts_with("Enemy_DeadHand") || (mini_lizalfos && stem == "Enemy_Lizalfos") {
        o.miniboss_health
    } else {
        0
    };
    if n == 0 { o.health } else { n }
}

pub fn active(o: &Opts) -> bool {
    o.health != NORMAL || (o.boss_health != 0 && o.boss_health != NORMAL) || (o.miniboss_health != 0 && o.miniboss_health != NORMAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_and_shared_lizalfos_use_independent_values() {
        let o: Opts = Opts { health: 150, boss_health: 300, miniboss_health: 50, ..Opts::default() };
        for s in ["Enemy_DungeonBoss_Gohma", "Enemy_DungeonBoss_Ganon", "Enemy_DungeonBoss_Koume"] {
            assert_eq!(multiplier(s, false, &o), 300);
        }
        for s in ["Enemy_MiddleBoss_DarkLink", "Enemy_MiddleBoss_Poe_Meg", "Enemy_IronKnuckle", "Enemy_DeadHand"] {
            assert_eq!(multiplier(s, false, &o), 50);
        }
        assert_eq!(multiplier("Enemy_Lizalfos", true, &o), 50);
        assert_eq!(multiplier("Enemy_Lizalfos", false, &o), 150);
        assert_eq!(multiplier("Enemy_Stalfos", false, &o), 150);
        let old: Opts = Opts { health: 250, ..Opts::default() };
        assert_eq!(multiplier("Enemy_DungeonBoss_Gohma", false, &old), 250);
        assert_eq!(multiplier("Enemy_MiddleBoss_DarkLink", false, &old), 250);
        assert!(!active(&Opts { boss_health: 100, miniboss_health: 100, ..Opts::default() }));
        assert!(active(&Opts { boss_health: 200, ..Opts::default() }));
    }
}
