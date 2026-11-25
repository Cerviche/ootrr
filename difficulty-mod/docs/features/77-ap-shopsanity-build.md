# 77. Archipelago: a slot with Shopsanity builds

Upstream bug fix for Archipelago slots. No new setting; settings strings and default seeds are unchanged.

## Requested

A player reported on Discord (25 Nov, web and exe) that an Archipelago slot with Shopsanity on is refused at the step "Check the world with OOTRR's logic and build the mod" with `this seed turns shop shuffle on but its file has no shop table, generate it again with this build`. The report also names "dungeon shuffle"; see "Not reproduced".

## Cause

- Every mod build checks that a seed file's shop table matches its settings (`shops::seed::check`). Settings with `shuffle_shops` on and an empty table are refused.
- `ap::patch::check` rebuilds the seed file from the `.apootrr` patch. It ran `prepare` again and dropped the shop layout, then made the file with `SeedFile::new`, which starts with an empty shop table. Nothing filled it. The plain generator fills it in `slice/generate.rs` (`fill_deals`, then `shops::seed::entries`).
- The patch holds placements only, and the apworld has no shop code. The layout is a pure function of the settings and the seed, so `check` gets the same layout the apworld's logic bundle was made from.
- Reproduced from the command line on nightly 2be61b88 (export, patch, `ootrr ap-build`): `shuffle_shops=shuffle` and `deals_1` fail with the message above; Shopsanity with dungeon item shuffles (`any_dungeon`) fails the same way; the same slot without Shopsanity builds.

## The fix

In `randomizer/core/src/ap/patch.rs`, `check()`:

1. Keeps the layout from `prepare`.
2. After the seed file is made, fills the deals from the placements and copies the layout into the seed file's shop table, the same two steps as the plain generator.

`shops::seed::check` is unchanged. Follow-up, not in this change: a patch that puts another player's item on a shop deal is not refused. The apworld never writes one (shop rows take the player's own items only); a hand-edited or mismatched patch could, and the foreign label would become the shelf goods. With Shopsanity off the layout is `None` and the function behaves as before.

## Tested

- Unit tests in `ap/patch.rs` (`cargo test -p ootrr-core ap::patch`, 3 of 3 pass with the fix): for `deals_1` and `shuffle`, a solo patch made from the generator's placements checks with a shop table equal to the generator's (64 rows) and passes `shops::seed::check`; a default slot has an empty table.
- Without the fix (fix hunk reverted in the worktree, tests kept): the shop-table test fails with an empty table against 64 rows at `deals_1` (run with an earlier fourth test for a foreign item on a deal, since dropped from this change; the patch-parsing and default-slot tests passed). Restored afterwards.
- Command line (scripted, no game, build from this branch): `ootrr ap-build` exit 0 for `control_off`, `shuffle_shops=shuffle`, `deals_1`, `deals_1` plus `any_dungeon` keys; each Shopsanity build reports "64 shelf slots read back, 0 problems"; the `deals_1` build reports 8 special deals. With the generator's two Ice Traps on deals swapped back to the deals' goods the build reports 8 special deals through 6 new shelf actors, and `flow-loads` on it finds every called flow listed. Dungeon entrance shuffle still exits 1 with `entrance shuffle is not supported yet`.
- Default output: a default-settings `ootrr build` and an `ootrr ap-build` of a default-settings slot, each made with the nightly 2be61b88 exe and this branch's exe, give the same 296 and 294 files byte for byte (SHA-256 of every file).
- Full workspace tests (`cargo test --workspace` with the game files set, through the build gate): 2401 passed, 0 failed, 13 ignored.
- Not played. The shelf edits are the existing Shopsanity feature; buying a deal in an AP-built mod in the game was not done.

## Not reproduced

- "Dungeon shuffle" in the report. Dungeon entrance shuffle is refused on purpose at Archipelago generation (`entrance shuffle is not supported yet`), so it cannot produce the shop-table message. Dungeon item shuffles build fine alone. The reporter's files were not seen.
- The app page and the browser version were not run; they call the same `ap::build::build`.

## How to test it in the game

1. Make an Archipelago slot whose settings string has Shopsanity on (`deals_1`), with the apworld or `ootrr ap-export` and a solo `.apootrr`.
2. `ootrr ap-build <patch> <game> <load> <name>`: it writes the mod; before the fix it exits 1 with the shop-table message.
3. Start the game with the mod, walk into the Kokiri Shop and buy the deal.

## MR Drafts

### Initial MR Draft
**Title:** Improved Archipelago > Shopsanity Fix: Slots with Shopsanity build

Building the mod for an Archipelago slot with "Shopsanity" on stopped with "this seed turns shop shuffle on but its file has no shop table". The slot's seed file was rebuilt from the patch without its shop table. The build now fills the table the same way the normal generator does. Checked by command line builds of `deals_1`, `shuffle` and a default slot, and by unit tests.
