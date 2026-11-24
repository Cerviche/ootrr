# 72. Hyrule Field sun stuck over Death Mountain (the "sunrise bug")

No setting of its own. The bug shows with **World > Kakariko Gate: Open** (and in a few other routes, below); the fix is in the always-on story cutscene skip (Saria's gift) plus a repair for saves made before it.

## Request

Reported on the OOTRR Discord (OOTRR 0.8.1 on Longshot 3.0.0): adult at Lake Hylia, Sun's Song at night, clear sky, and the sun came up on the wrong side, behind Death Mountain. Shooting where the sun should be still started the Fire Arrow scene. The report came with a video, which pinned it down. The task: find the cause and fix it with data edits only, so the flag can never stay stuck, and repair saves that already have it.

## Interpretation

The sun the player sees in Hyrule Field and Lake Hylia must follow the clock at every hour after the first walk into the field, whatever the settings, on new saves and on saves made with the bug.

## Investigation (from the game files)

- Hyrule Field's world parameters (`WorldMgr/WorldParam/HyruleField_Child`, Adult inherits it) have one `ConditionEnvTimePalette` row: while the saved flag `HyruleField_ChangeENV` is true, a clear sky uses the palette table `MainField_Bluesky_First` (cloudy stays `MainField_Cloudy`). That table is `MainField_Bluesky` plus `IsForceMainLight: true`, `MainLight_Azimuth: -215`, `MainLight_Elevation: 18`: the sun is pinned to one direction at every hour, day and night. Lake Hylia is part of the field map, and from the Fire Arrow tablet that direction (bridge camera yaw 145, 18 degrees up) frames Death Mountain: the reporter's picture. The normal palette follows the clock (`MainField` `IsSolarMotion: true`, `SunRiseAzimuth: 90`, sunrise 04:00 in the field, sunset 19:40).
- That palette row is the only reader of the flag found in the game files (Event, AI, Logic and GameData were searched).
- Vanilla writes the flag twice, both in `Event/EventFlow/Goto_HyruleCastle`: `Event218` sets it true after Saria's bridge scene (`Dm_Kokiri_0155`), just before Link is sent into the field; `Event217` clears it in the first field owl (entry `Goto_HyruleCastle_Kaepora_00`: `Event196` ... `Dm_HrlField_0000`, `Dm_HrlField_0005`, then `Event217`, a 1-frame `Event220` wait, `Event221` `EventWorldManagerController.Event_RebuildConditionEnvTimePalette`, then `Dm_HrlField_0010`). So in vanilla the pinned sun lasts only through the field's opening scene.
- The owl starts from `EventRequestActor` 893417770466804315 in `HyruleField_Child_Static` (request `HyruleField_Goto_HyruleCastle_Kaepora`), at the field end of the forest bridge (area 6015224556942575734 at 158.9 162.7 285.1). Its logic (`Logic/HyruleField_Child_Prefab_de5f`) deletes the area once the `Goto_HyruleCastle` quest is past `Step01`, and reads `HavePlayedEvent.Dm_HrlField_0010`.
- Saria's bridge trigger is area 2646691965227689495 in `KokiriForest_Child_Static` (-116.57 114.15 -38.09), retired once `Dm_Kokiri_0150` has played (`Logic/KokiriForest_Child_Level_All_02_ddcd`). It does not depend on the quest step.

What OOTRR did before this fix:
- `story_skip::patch_saria_give` (always on) cuts the bridge scene to an item get and writes the scene's leftovers itself: `Event17` -> `OOTRRSariaGive...` (`Dm_Kokiri_0150` played, `Dm_Kokiri_0155` played, **`HyruleField_ChangeENV` = true**) -> the item get. The vanilla `Event218` is no longer reachable from the bridge entry.
- With Kakariko Gate Open the new game marks the whole castle story as done (`start_state::KAKARIKO_FLAGS`, including `HavePlayedEvent.HyruleField_Goto_HyruleCastle_Kaepora` and `Dm_HrlField_0010`, and the `Goto_HyruleCastle` quest `Complete`), so the owl never plays and nothing ever clears the flag: from Saria's gift on, every clear day and night in the field and at the lake has the pinned sun.
- `start_state::keep_field_clock` (applied with Kakariko Open or Time always passes) turns `Event218` into a clear, but `Event218` is dead code under the story skip, and the gift wrote true afterwards anyway. Its comment (and the one on `FIELD_FIRST_VISIT`) said the flag is cleared by Impa's send off and freezes the clock; the files say the owl clears it and it picks the palette. Both comments are corrected.
- The owl cut (`patch_field_owl`: `Dm_HrlField_0005` and `0010` become their played flags) keeps `Event217` and the rebuild, so with Kakariko Closed the owl still cleared the flag on the first walk into the field.

History (from the repo):
- 8d12ecb (13 Nov, "Fixed Time Cycle") added `start_state::keep_field_clock` and `stage_field_clock`. Its doc comment says Saria's bridge scene "marked the field as first visited and froze the clock on every new game", so the write (`Event218`) "is turned into a clear". At that point `patch_saria_give` sent the item get on to `Event218` (the event after the gift scene), so with an open gate the gift ended in the clear.
- b4fdf84 (19 Nov, "Shorter story cutscenes and short item get text") rewrote `patch_saria_give`: the item get now ends the event (`NONE`) and the chain itself writes `set_bool(f, SARIA_AFTER, true)`. Its message says the bridge trigger "sets its event bool, both scene flags and the Hyrule Field palette flag". `Event218` is no longer reachable, so this undid 8d12ecb's clear for an open gate and wrote true for every gate setting. That is the regression this fix reverses.

## The flag through a playthrough

"Owl plays" = Kakariko Gate Closed and Link enters the field through the forest bridge before the `Goto_HyruleCastle` quest moves past `Step01`. The time savers (`skip_intro`, `skip_cutscenes`, `faster_chests`) and Time always passes do not change any cell: none of them writes the flag, the owl's played flags or the quest step.

| Point in the game | Before, Kakariko Closed, owl plays | Before, Kakariko Open | Before, owl played before the gift, or retired by the quest step | After the fix, any settings |
|---|---|---|---|---|
| New game | 0 (never written) | 0 (written 0 with the owl marked played) | 0 | 0 |
| Saria's gift on the bridge | 1 | 1 | 1 | **0** (the gift writes 0) |
| Walking into the field (owl area) | owl: 0 and palette rebuilt | no owl: 1 | no owl: 1 | owl (if it plays): 0 and palette rebuilt; no owl: 0 |
| Rest of the game, clear sky | clock sun | **pinned sun** | **pinned sun** | clock sun |
| A save made with the bug (flag 1), loaded in the fixed build | - (the owl will clear it) | 1 until the next item window, then 0 | 1 until the next item window, then 0 (owl played, or `Goto_HyruleCastle` past `Step01`) | - |

The third column: the owl area sits at the field end of the bridge, so a player who reaches the field another way (Lost Woods to Zora's River or Goron City, warps), walks back through the owl area into the forest and only then crosses the bridge gets the gift after the owl. And with the gate Closed, a player who moves the `Goto_HyruleCastle` quest past `Step01` before ever crossing the owl area retires the owl (its logic deletes the area) while both owl flags stay 0; the repair checks the quest step too, so that save is repaired as well. Not reproduced in the game; it follows from the trigger data above.

Known gap: an old save with the flag 1, both owl flags 0 and the quest at `Step01` or before keeps the pinned sun until the owl plays, as in vanilla (the owl then clears it). The repair waits for the owl on purpose there, to keep the vanilla first-visit look.

Cost of the fix: with Kakariko Closed, the field's opening scene (`Dm_HrlField_0000`, when it plays) is lit by the clock instead of the fixed first-morning light. Everything after it is as in vanilla.

## Implementation

1. `story_skip::patch_saria_give`: the gift chain writes `HyruleField_ChangeENV` false instead of true. Nothing in a fixed build writes it true any more (checked by a walk of every event reachable from the bridge entry).
2. Repair for saves made with the bug (`field_palette.rs`, staged in the late stage right after Saria's gift): the shared item-get entry `GetPouchContent:GetPouchContent` (every chest and gift item runs it, as `starter_bags.rs` also relies on: the `ByActorName`, `ForShowPickUpWindow`, `ForTreasureBox` and `ForGetInEvent` entries all call it, and `Dm_SY_GetInEvent` calls those) now starts with four questions and one write: if `HyruleField_ChangeENV` is true and the owl is marked played (`HavePlayedEvent.Dm_HrlField_0010` or `HavePlayedEvent.HyruleField_Goto_HyruleCastle_Kaepora`) or can no longer play (`Goto_HyruleCastle_IsAfter_Step01_Exp`, the game's own expression for the quest step the owl's logic checks, asked with `EventQueryIsGameDataBoolExp` as `open_world/guard.rs` already does for other quests), set `HyruleField_ChangeENV` false; every branch then goes on to the entry's old first event (`Event11`, `EventTriggerSetForGetWindow`). Nothing branches on the written value, so no wait is needed. It uses the flow's existing `GameDataActor` (only its query and action lists grow, the way debug names and starter bags already extend this flow), adds no actor and no call into another flow file, so no preload list changes.
   - No palette rebuild in the repair: `Event_RebuildConditionEnvTimePalette` would need a new `EventWorldManagerController` actor in a flow that runs in every map, which has not been tried. The flag only matters when a map builds its palette, so the repaired value takes effect at the next map load of Hyrule Field (leaving a grotto, a house, Kakariko or any other area). If the repair runs while Link is in the field itself (a field chest or a scrub), the pinned sun stays until he leaves and comes back.
   - The owl conditions keep the vanilla first-visit look for an old save that is still between the gift and an owl that will play.
3. Comments corrected in `start_state.rs` (`FIELD_FIRST_VISIT`, `KAKARIKO_FLAGS`) and on `patch_saria_give`.

Default seeds and settings strings are unchanged (no setting was added or changed). Every build's staged `Goto_HyruleCastle` (one value) and `GetPouchContent` (five events, and the query `EventQueryIsGameDataBoolExp` on its `GameDataActor` if the flow did not list it yet) differ from upstream main on purpose; nothing else does.

## Tests

`OOTRR_ROMFS` = `<game romfs>`.
- `field_palette::tests` (in `field_palette.rs`): on a synthetic flow, all sixteen combinations of the flag, the two owl flags and the quest-step expression end on the entry's old start, and only "flag true and any of the three" writes the clear; the step question uses `EventQueryIsGameDataBoolExp` and the actor lists it once; a second patch and a flow without the entry are refused; the output parses back. On the real `GetPouchContent`: the same sixteen walks end on the real old start (`Event11`) and write the clear in the same cases, every vanilla event, actor count, other entry start and preload list is unchanged, the file round-trips, and `Goto_HyruleCastle_IsAfter_Step01_Exp` is a `BoolExp` in `data/flags.json`.
- `field_palette::tests::a_build_with_the_other_item_window_edits_runs_the_repair_first`: a full build with `debug_location_names` and `skip_first_pickup_text` on (short item text is always on), the other features that edit `GetPouchContent`: the built entry still starts with the repair, the sixteen walks write the clear in the same cases and all end on one event, and a second patch of the built flow is refused.
- `story_skip::tests::the_real_saria_gift_clears_the_field_palette_flag_and_never_sets_it`: after `patch_bridge` and `patch_saria_give` on the real `Goto_HyruleCastle`, the only write of the flag reachable from `KokiriForest_ArriveLostWood` (following every next, case, fork and join) is the gift's false; the owl entry still reaches its one clear.
- `cargo test -p ootrr-core story_skip`, `field_palette`, `difficulty`, `settings`, then `cargo test --workspace`: all pass (final version, 24 Nov: ootrr-core 1614 passed, 6 ignored, the other crates 15, 51, 248, 77, 1, 1 and 235; first version: ootrr-core 1613 passed, 6 ignored, which takes in `difficulty`, `settings`, `story_skip` and the default-seed, pin and settings-string tests, all unchanged; the other crates 15, 51, 248, 77, 1, 1 and 235 passed).
- Tester builds (the reporter's settings string and seed plus the time savers, built into empty folders: upstream main c139d4e and this branch's commit): `ootrr flow-loads <game romfs> <mod romfs>` reports "every event of the edited flows lists the flows it calls" for both. The two spoilers are identical, and the two mods (541 files each) differ in exactly three files: `GetPouchContent.bfevfl.zs`, `Goto_HyruleCastle.bfevfl.zs` and the resource size table. Dumped: on main `OOTRRSariaGive214` writes `HyruleField_ChangeENV` true (the bug, as the investigation said); on the fix it writes false, and the `GetPouchContent` entry starts at `OOTRRFieldSun114` (flag?) -> `115` (`Dm_HrlField_0010` played?) -> `116` (owl request played?) -> `117` (clear), every branch ending on the old `Event11` (index 6). (That was the first version; the repair now asks the quest-step expression after the second owl flag, so it is five events.)
- Content check: `python tools\audit.py randomizer` and `python tools\audit.py difficulty-mod`: FAIL 0 both; the 8 WARNs in `randomizer` are in files this fix does not touch.

## Played in the game (24 Nov, by script, build faaf0ca)

A tester ran the plan on the first version of the fix (two owl conditions, no quest-step condition), with the reporter's settings plus the time savers, against upstream main c139d4e:
- Saria's gift: the flag read **1 on the bug build, 0 on the fix build**.
- Repair (flag set 1 with the owl marked played, then a Mido chest): **fix 0, bug 1**. Repeated as adult: 1, then 0 after the chest on the fix build.
- Control (owl flags set 0, flag 1, then the next chest): the flag **stayed 1 on the fix build** (redone cleanly). The bug build's control run was tainted (its starting state was not clean), so it proves nothing either way.
- The game carried on after the gift and after each chest (no freeze, Link moved).
- Pictures at the Lake Hylia Fire Arrow tablet (a person looked at them): with the flag on, a sun sat left of Death Mountain at 22:00, at dawn and at noon, and there was none in the east at dawn; with the flag off, and on the fix build after the repair and a map hop, a plain night sky at 22:00 and the sun low in the east at dawn.

Not played yet: the quest-step condition added after the review, the default settings (gate Closed), the repair through a non-chest item get, and the repair with location names and first-pickup text on. The plan below covers them.

## How to test it in the game

A tester plan exists outside the repo. The short version, with two mods built from the reporter's settings (Kakariko Gate Open, Kokiri Forest Open) plus the time savers (`skip_intro`, `skip_cutscenes` all, `faster_chests`, `starting_hearts` 20), one from upstream main (bug) and one from this branch (fix):
1. New game on each. Before the bridge, `flag get HyruleField_ChangeENV`: 0 on both.
2. Cross Saria's bridge (warp to -116.6 114.6 -38.1), take the gift, clear the dialogs, read the flag: **1 on the bug build, 0 on the fix build.**
3. Repair (fix build): `flag set HyruleField_ChangeENV 1` (what an old save holds; the owl flags are already set by Kakariko Open), then open one of Mido's chests (Kokiri Forest, about -50.3 112.3 -42.0): the flag reads 0 after the window. Control: the same on the bug build stays 1; and on the fix build with `HavePlayedEvent.Dm_HrlField_0010` and `HavePlayedEvent.HyruleField_Goto_HyruleCastle_Kaepora` set to 0 first, the flag stays 1.
3b. Quest-step repair (fix build): both owl flags 0, `Step_Goto_HyruleCastle` written past `Step01` (for example `Step02`), flag 1, then a chest: the flag reads 0.
3c. Default settings (gate Closed, a third mod): the gift writes 0, the first walk into the field from the bridge still plays the owl (its played flag turns 1), the flag stays 0, Link is free afterwards.
3d. A non-chest item get (a shop purchase or an NPC gift) after setting the flag 1 with the owl played: 0 afterwards. And once on a fix build with `debug_location_names` and `skip_first_pickup_text` on: a chest and a ground item both still clear it.
4. Look (needs a clear sky; the bridge cannot set the weather): as adult at the Fire Arrow tablet in Lake Hylia (-189 132.3 515), at night and at dawn, camera toward Death Mountain (yaw 145, 18 degrees up). Bug build after step 2: a sun disc over Death Mountain at every hour. Fix build: none at night, the sun low in the east at dawn. The lake's sky stays rainy until `SageOfWater_EscapeFromWaterTemple` is set.

## Notes for the next feature

- `ConditionEnvTimePalette` rows in a map's WorldParam switch the sky palette by a saved flag; a palette table with `IsForceMainLight` pins the sun. Search the WorldParams for a flag before guessing what it does.
- A story cut that writes a scene's leftovers itself must also write the clear that a later, skippable scene would have done, or not write the set at all.
- `GetPouchContent:GetPouchContent` is the one step every windowed item get shares: a cheap place for a one-time repair that must run for every player.
- A quest step can be asked in a flow through the game's own `<Quest>_IsAfter_<Step>_Exp` expressions (`EventQueryIsGameDataBoolExp`); `data/flags.json` lists them as `BoolExp`.
- Before calling a skipped scene's write a bug, read the history: the clear here was added on purpose (8d12ecb) and lost in a later rewrite (b4fdf84).

## MR Drafts

### Initial MR Draft
**Title:** Improved World > Kakariko Gate Fix: the Hyrule Field sun no longer sticks over Death Mountain

With Kakariko Gate Open, Saria's gift set the flag the game uses for Link's first walk into Hyrule Field and the owl that clears it never played, so on clear days Hyrule Field and Lake Hylia drew the sun pinned in the north-east at every hour (reported on the OOTRR Discord). Saria's gift now clears that flag, and the shared item-get flow clears it on saves that already have it once the owl is marked played or can no longer play (it takes effect at the next field load). Data edits only; no setting changes.

This restores the clear that 8d12ecb ("Fixed Time Cycle") added for an open gate and that the Saria's gift rewrite in b4fdf84 bypassed.

Played by script on the first version (24 Nov): the gift writes 1 on main and 0 here, the repair clears an old save's flag after a chest, and pictures at Lake Hylia show the sun back in the east at dawn. The quest-step repair, the default settings and non-chest item gets are not played yet. Every build's `Goto_HyruleCastle` and `GetPouchContent` change slightly on purpose (hunks outside `field_palette.rs`: one value in `story_skip.rs`, one stage call in `slice/edit.rs`, comments in `start_state.rs`); default seeds and settings strings are unchanged.

For details about implementation and testing, see difficulty-mod/docs/features/72-field-sun-palette-fix.md.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
