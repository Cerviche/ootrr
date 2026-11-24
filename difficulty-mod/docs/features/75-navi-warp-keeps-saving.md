# 75. Return with Navi: a trip out of a locked room no longer turns saving off

Upstream bug fix for the setting Return with Navi (setting 78, `navi_return`). No new setting; the setting's default and the settings strings are unchanged. Based on `origin/nightly` (dda8087, which carries the same test fixes as #120 and #121, so the upstream CI passes without them).

## Requested

Upstream issue #71: after entering the Deku Tree side room with the Deku Scrub and using Navi to warp out, the game no longer autosaves and cannot be saved by hand. Upstream issue #72, "Navi warps can disable saving" (maintainer, no text). Fixes #71 and #72.

## Cause

- The scrub room's door is a locking shutter (`Obj_SwitchShutter_A_01`, placement hash `11355963437042053245` in `InsideTheDekuTree_Static`) whose placement carries `IsDisableSaveAndWarp: true`. When it locks behind Link its AI graph enters the game safety section `Common_ShutterEnemy` (node `OneshotEnterGameSafetySection`), which turns manual saves, autosaves and item warps off. The same graph leaves the section (`OneshotLeaveGameSafetySection`) when the lock is released, that is when the room is beaten.
- Return with Navi changed the map with a bare `GameSystemActor.EventReserveBancChange`. The door is unloaded with the map, so its "leave" never ran, and nothing in the map change stops the section. Saving stayed off until a game over and Try Again, loading a save, or going back to the title.
- 45 locking doors with that key exist in 13 maps (Deku Tree 1, Dodongo's Cavern 4, Jabu-Jabu 2, Forest Temple 3, Fire Temple 3, Water Temple 1, Shadow Temple 6, Spirit Temple 6, Ice Cavern 1, Gerudo Training Ground 6, Ganon's Castle 8, Graveyard child 2 and adult 2). Navi offers a trip from every one of them.
- The boss-room list in `STAY` covers the boss rush stages only. The story's own rooms are inside the dungeon maps, so a map name cannot tell them apart.

## The fix

In `randomizer/core/src/qol/navi_return.rs`, `offer()`: after the player answers yes, the trip now runs, in order,

1. `GameSystemActor.EventTriggerLeaveGameSafetySection` (no parameters; the game's own action for ending a section, used the same way before a map change in `Dm_SY_BancChangeGo:InsideGanonCaslte_Dungeon_EscapeGo`),
2. `ChallengeSystemActor.EventTriggerMiniGameTimerStop`,
3. `ChallengeSystemActor.EventTriggerUnshowTimerUI` (both without parameters, in the order the game's own flows use, for example `SubEvent_GetSwordBigGoron`),
4. the existing `GameSystemActor.EventReserveBancChange`.

The three actions are added to actors the patched flow already has (`GameSystemActor`, `ChallengeSystemActor`); no new actor, no new file, no new setting. The "quiet while a minigame timer counts" check from #100 stays in front of the question, so Navi still never ends a race or a boss fight halfway. The timer stop and hide are a safety net so a trip never carries a timer display into the next map (the sibling report is #74, see feature 76). The `STAY` comment now says what the trip does instead of claiming the state cannot be undone.

What changes in a default build: Return with Navi defaults to "everywhere", so stock settings include this flow, and only this flow (`Event/EventFlow/NaviHint.bfevfl.zs`) changes: 3 more events per question (the flow holds 12 dungeon questions in "dungeons" mode and 14 in "everywhere" mode). The only other file that changes is the size table (`System/Resource/ResourceSizeTable...rsizetable.zs`), for the flow's larger size; checked 25 Nov by hashing the default build against nightly dda8087 (286 files, these two differ). With the setting off, the build is unchanged (the stage returns no files).

## Risks, as read and not tested

- Leaving a section when none is running is expected to do nothing (the game's own escape flow calls it with no check) [inferred]; the lobby check below covers it.
- Stopping and hiding the timer when none runs is expected to do nothing [inferred, not tested].
- The section's settings row keeps the default "save when it ends" [inferred]: leaving may write an autosave with Link still inside the locked room, just before the map change. A reload of that save then starts in the room with the door locked again; Navi still works there. Not tested.
- Leaving a quest section (boss approaches without a timer, the Gerudo carpenter fight) through Navi ends that section early. Before this fix the same trip left saving off for the session. Not tested.
- After the trip the room's enemy is still alive; coming back, the door locks and the section starts again. Expected.

## Tested

- Reproduction of the bug in the game (nightly 80eb5be, investigator run 24 to 25 Nov, a scripted manual save through the pause menu): in the Deku Tree scrub room, after a Navi trip out, 0 save files changed and the Save entry was greyed; a baseline save at the door and after a Navi trip from the lobby changed 3 files; game over and Try Again brought saving back.
- Unit tests (`cargo test -p ootrr-core navi_return`): every map change the patch adds is reached only through the leave, stop and hide actions in that order (walked for every dungeon map, both age answers, both modes); the original events of the flow are unchanged; the event count is `DUNGEONS.len() + 12 * 6 + 1`; the existing boss room, timer and second-run tests still pass.
- Game-file test: the vanilla `Dm_SY_BancChangeGo` flow still holds `EventTriggerLeaveGameSafetySection`, `SubEvent_GetSwordBigGoron` still holds the two timer actions, and the staged `NaviHint` flow holds all three. A game update that renames one fails the build's tests, not the player's save.
- The fix itself in the game (scripted, 25 Nov, build 030, the same test that reproduced the bug): after the Navi trip out of the Deku Tree scrub room the manual save wrote the save (3 files changed) and the Save entry was not greyed. A Navi trip from the lobby, and trying to save inside the locked room, behave as before. Other locked rooms were not played.

## How to test it in the game

Build with Return with Navi on ("everywhere"), `skip_intro`, `skip_cutscenes` all and `faster_chests`. Child save at the lobby side of the Deku Tree scrub room door.

1. Pause menu, System tab, Save: works (baseline).
2. Walk in; the door locks. Save is refused inside (the game's own rule, unchanged).
3. Press D-pad Right, answer yes. Link arrives at the dungeon entrance.
4. Open the pause menu: the Save entry is available and a manual save works (before the fix: greyed).
5. Also try Navi from the lobby (no locked room) and then save: unchanged.
6. Walk back in: the door locks again, Save is refused again; leave with Navi again, Save works.
7. Close and start the game again on the newest save: Link is at the entrance, or in the room with Navi still working.
8. Repeat in another locked room, for example the Dodongo's Cavern door at room 4.
9. By hand: beat the scrub and leave by the door, then save (the vanilla exit, must be unchanged).

The scripted version is the investigator harness's `fix` mod beside `navi` and `control`; its plan is `playtest/inv71/PLAN.md`.

## Research numbers

Locking doors with `IsDisableSaveAndWarp`: 45 in 13 maps (list above). Events added: 3 per question. Section settings row: `Common_ShutterEnemy`, name hash 1509064940, `IsCheckSafetyArea` and `IsDisableGrab` false, nothing else written.

## MR Drafts

### Initial MR Draft
**Title:** Improved World > Return with Navi Fix: a trip out of a locked room no longer turns saving off

Using Return with Navi to leave a locked enemy room (for example the Deku Tree scrub room) left the game refusing manual saves and autosaves until a game over or reload. The trip now leaves the game's safety section, and stops and hides any minigame timer, before it changes the map. Reproduced in the game on nightly before the fix (0 save files changed after the trip, Save greyed); the fix itself is not yet confirmed in the game.

Only the Navi flow changes (3 more events per question); with the setting off nothing changes. Merged with nightly dda8087, which fixed nightly's own tests. Base: nightly.

Fixes #71, #72.

For details about implementation and testing, see difficulty-mod/docs/features/75-navi-warp-keeps-saving.md.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

### 2nd MR Draft
**Title:** Improved World > Return with Navi Fix: a trip out of a locked room no longer turns saving off

Using Return with Navi to leave a locked enemy room (for example the Deku Tree scrub room) left the game refusing manual saves and autosaves until a game over or reload. The trip now leaves the game's safety section, and stops and hides any minigame timer, before it changes the map. Tested in the game by script: on nightly the trip out of the Deku Tree scrub room left Save greyed (0 save files changed); with the fix the same test wrote the save (3 files) and Save was not greyed. A trip from the lobby and saving inside the locked room behave as before. Other locked rooms were not played.

Only the Navi flow changes (3 more events per question), and with it the size table; with the setting off nothing changes. Merged with nightly dda8087, which fixed nightly's own tests. Base: nightly.

Fixes #71, #72.

For details about implementation and testing, see difficulty-mod/docs/features/75-navi-warp-keeps-saving.md.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
