# Enemies call for backup

Setting `enemy_backup` (number 126), checkbox. Code: `crates/core/src/difficulty/backup.rs`. Built by a sub-agent on `feature/enemy-backup` (`f3f674c`).

## Request
"enemes call for backup. if you take too long defeating an enemy, they can call summon another enemy"

## Interpretation
After a fight has lasted a while, the enemy summons one more enemy near itself, with limits so it cannot snowball. I suggested 20 to 30 seconds and at most once per enemy or a long cooldown.

## Investigation
- **"In a fight":** the enemy currently has a valid target (`QueryTargetIsValid`), the shape Gohma's own battle graph uses.
- **Spawning:** the game's "actor holder" component lets one actor own and create another. Vanilla examples: Bari holds Biri, big Leevers hold Leevers, Floormasters hold small ones, and a fishing lure creates its hat on demand with the exact node sequence used here.
- **What to summon:** a helper that does not run the shared enemy graph, so it can never summon in turn. Every Keese variant runs that graph, so there is no Keese helper. The choice depends on the caller (table below), and the Peahat larva (`Enemy_Peahat_Larva_Independent`, the standalone larva from child Hyrule Field) is the fallback.
- **Once per enemy:** two per-actor bool keys, `OotrrBackupEligible` (starts true) and `OotrrBackupUsed` (starts false), are added to the shared `EnemyCommon` blackboard table in every caller's pack. Vanilla precedent: `ExecuteBlackboardSetBool` with `ExecuteOnce` and `IsIgnoreNotExistsError` writes the `EnemyCommon` key `IsStunLightEnable` in the Tektite graph, and `QueryBlackboardGetBoolNoError` reads keys in the Amos, Gibdo and Lizalfos graphs.
- **Bosses out:** the excluded actors get no `Eligible` key and no holder of their own. A missing key reads false, so their graph stops at the first check.

## Implementation
- The shared enemy graph (same hook as regen) gains 24 nodes. Per frame: if `Eligible` is false, do nothing. Otherwise: if the target is valid and a created helper sleeps in the holder, wake it 1.5 m above the caller; else if the holder is empty and `Used` is false, run a 750-frame timer (about 25 seconds), then request the helper and set `Used`. `Used` is set at the same moment as the request and is never cleared, so there is no second call even after the helper dies. The holder nodes sit below the `Eligible` check, so an excluded actor never touches a missing `Backup` entry.
- Excluded: `Enemy_DungeonBoss_*`, `Enemy_MiddleBoss_*` (Dark Link, Nabooru's Iron Knuckle, minibosses), `Enemy_IronKnuckle_*`, `Enemy_DeadHand*`, `Enemy_PhantomGanon*` and `Enemy_GhostPicturePerformer_*` (Phantom Ganon, his horse, the performers), `Enemy_Guay_Boss`, `Enemy_KotakeKoumeBattleManager`. 33 of the 123 actors that run the graph are excluded. A few boss packs carry a parent enemy's parameter file that does gain the holder slot, but without the `Eligible` key it is never used.
- 81 callers get an `ActorHolderRef` (65 own a new shared holder, 10 existing holders gain a `Backup` entry) and the two keys in their common blackboard table. Three new shared holder files, one per helper.
- Extra AI memory for the 123 actors that run the graph, and a size table entry for each new file.

| Caller group | Helper | Why |
|---|---|---|
| Bari, Biri, Stinger (both), Shell Blade, Tentacles (Jabu-Jabu natives) | `Enemy_Shabom` | The game places Shabom only in Jabu-Jabu's Belly, where these callers live; it floats and does not run the shared graph. |
| Freezard, Fixed Freezard, Gibdo, ReDead (3), Wallmaster, Like Like, Floormaster (3) (Ice Cavern, Shadow and Spirit Temple natives) | `Enemy_FlyingPot` | The game places flying pots in the Ice Cavern, Shadow and Spirit Temples (and Gerudo Fortress); it does not run the shared graph. |
| Everything else, including Octorok and Blue Tektite | `Enemy_Peahat_Larva_Independent` | Fallback: flies, runs no shared graph, known to exist as a standalone actor. |

A holder file shared by callers of different groups falls back to the larva.

## How it was tested
- Difficulty tests with node shapes compared against the lure, Dodongo, Gohma, Twinrova, Tektite and Amos graphs, plus checks that every helper is outside the shared graph and every excluded actor lacks the `Eligible` key.
- A real build; the built mod has the patched graphs, new file, holder entries and memory growth.

## In the game
Never played, and this is the riskiest feature. To test:
- Turn this on alone. Find one ordinary enemy (a Deku Baba or Stalfos), stay close without killing it for 40 seconds.
- Pass: a Peahat larva appears above it. Kill the larva and keep fighting: no second one may come.
- First check: that the map loads at all, then that the helper actor is created outside the map it normally lives in (Shabom outside Jabu-Jabu, flying pot in the Ice Cavern, Shadow or Spirit Temple).
- Fight a boss or miniboss for a minute: nothing may appear.
- Report: a crash when enemies load, nothing happening, or helpers piling up.

## Notes
- It rests on untested assumptions: that the holder component switches on just from being added to an enemy's parameters, that a missing blackboard key reads false without an error, and that a new key in the common blackboard table needs nothing else.
- Helpers are created on demand, so their packs must load in maps that never placed them. Unknown until played.
- The 25 seconds assumes 30 counted frames per second.
- What happens to the helper when its caller dies is unknown.

## Red enemies (session fixes, 18 Nov, code only)
The user saw enemies tinted red with regen, fairy and backup on (screenshot of a Deku Baba in Kokiri Forest). Findings from the game files:
- The damage colour is made by one native node, `ExecuteSetEnemyDamageColorAndLight`, alone in `AI/EnemyCommon.common.reactiondamagecolor.module.ainb`. In the game's own file that node is the whole graph: no parameters, no inputs, no outputs, no links. Its colour logic is in game code, so the graph cannot be read for a cause.
- Regen, fairy and backup each wrap the old root in a Parallel (`EndPolicy` 0, `ResultPolicy` 0, the same values the game's own caller graphs use) with the old root as its first child. With all three the node sits three Parallels deep. Its own bytes and the link order are unchanged (new test `the_vanilla_damage_color_node_is_untouched_after_regen_fairy_and_backup`). Nothing writes a colour or damage key.
- Only regen writes to the life component (`OneShotLifeSetOwnLife`, add 1 every 150 frames, clamped at the maximum). The game uses that node only in the player's and NPCs' event graphs, never in an enemy graph.
- So no edit was found that changes the colour path, and none was made. Two causes remain:
  1. Regen: a heal, even one that changes nothing, may count as a life change that the colour node reads as a hit, giving a flash every 150 frames.
  2. The wrapper: the colour node runs as a child of a Parallel and not as the module root, and may behave differently there.
- Not caused by the speed setting (it scales keys ending in Speed and Interval only) or by the gate edit (the gate does not touch the guards' components).
- To tell them apart, build three mods with one setting each and look at a Deku Baba: `enemy_backup` only (wrapper but no heal), `enemy_regen` only (wrapper and heal), `enemy_speed` 200 only. Red with backup only means cause 2; red only with regen means cause 1; red with neither means another feature.
