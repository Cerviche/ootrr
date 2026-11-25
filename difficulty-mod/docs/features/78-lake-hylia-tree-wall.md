# 78. Lake Hylia tree Gold Skulltula: the wall waits for the Longshot chest

Bug fix. No new setting; settings strings are unchanged.

## Requested

The logic says the Gold Skulltula in the tree at Lake Hylia (`LH GS Tree`: Longshot, and night) is reachable once you hold the Longshot. In the game an invisible hookshot wall in front of it stays until the Longshot chest in the Water Temple has been opened, whatever that chest holds.

## Cause

- The tree has an invisible wall (`FldObj_AirWall_Hookshot`) 5.9 units from the token, in both `HyruleField_Adult_Dynamic` and `HyruleField_Child_Dynamic`. Its presence is negated `WaterTemple_Room07_00`: it is there until that flag is set. By the `<Scene>_RoomNN_NN` pattern of chest flags this is the Longshot chest's opened flag (inferred from the flag list, not seen in the game).
- With the Longshot shuffled, the chest can hold something else and the Longshot can be found elsewhere. Logic then expects the token reachable while the wall blocks it. If the route to that chest needs something placed at the token, the seed cannot be finished.
- Of the 25 hookshot walls in the game this is the only one with a presence flag; the other 24 are always there, in the original game too, and are left alone.

## The fix

New file `randomizer/core/src/lake_tree_wall.rs`, staged right after the Water Temple gate (`water_gate.rs`). In both Dynamic files it rewrites the wall's presence flag from `WaterTemple_Room07_00` to the always-true expression used by the gate fix, so the wall never appears. The adult file is edited by both; `edit_staged` chains the two edits. Each edit checks the actor's hash, name, negation and old flag, and refuses a second application.

Applied always, not only when items are shuffled: with vanilla placement the chest gives the Longshot, so by the time the player can hook the tree the wall would have gone anyway; the only change is that it is gone earlier. A condition on a setting would need a second code path for no gain.

The logic rule stays `can_use(Longshot)`. Whether the plain Hookshot reaches the token with the wall gone is not known; the rule stays conservative until someone sees it.

A build report line is added (`NOTE`).

## Tested

Commit: see the branch head.

- Unit tests (`cargo test -p ootrr-core lake_tree_wall`, game files set): both Dynamic files carry the wall with the chest flag before the edit and the always-true flag after, the actor count is unchanged, a second application is refused; the adult file staged after the water gate keeps both edits and is staged once; the stage is in the stage list. Results and the without-fix run are in the report for this change.
- Default output: see "Default output".
- Not played. Nothing here has been seen in the game.

## Default output

Every build changes: the two Dynamic files differ from nightly (the adult one also by the water gate edit, already in nightly) and the build report gets one line. No other file changes. Seeds and settings strings are unchanged.

## How to test it in the game

Plan: `playtest/lh-tree-wall/PLAN.md` (not yet written; see the report). By hand: adult, night (or Gold Skulltulas at any time on), Longshot, Water Temple Longshot chest not opened. Stand at Lake Hylia below the tree and hook the token. Before the fix the hook is stopped by the wall; after it the token is collected. Then the same with the Hookshot only, to decide whether the logic may be relaxed.

## MR Drafts

### Initial MR Draft
**Title:** Improved Logic > Gold Skulltula Fix: Lake Hylia tree token without the Water Temple chest

The Gold Skulltula in the tree at Lake Hylia is counted as reachable once you have the Longshot (and it is night). In the game an invisible wall in front of it stays until the Longshot chest in the Water Temple has been opened, even if the Longshot was found somewhere else.

With the Longshot shuffled, you could hold the Longshot and still be unable to hook that token, and a seed could need it.

This PR removes that wall, so the token can be hooked whenever the logic says it can.

For details about implementation and testing, see difficulty-mod/docs/features/78-lake-hylia-tree-wall.md.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
