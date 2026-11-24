# 60. Kakariko Gate open: no dungeon item handed out at new game

A fix for upstream, on branch `fix/new-game-dungeon-item` (based on `nightly`). No new setting: it changes what the existing Kakariko Gate setting (`open_kakariko`) lets the fill place.

## Request

With the Kakariko gate open, the items placed at "HC Zeldas Letter" and "Song from Impa" are handed out when a new game starts as a plain pouch add. A dungeon item there (for example a small key with Small Keys set to Anywhere) arrives with no dungeon credited, while the logic counts it. Make a small fix branch for upstream, with the numbers before and after.

## What happens

With `open_kakariko` = `open` the castle visit is skipped, and the items at its two checks (`start_state::KAKARIKO_LOCATIONS`) are given in the new game's `OpeningEvent` by `PouchSystemActor.EventTriggerAddOrRemovePouchContent` with the item's actor (`slice/edit.rs` `skipped_items`, written by `start_state.rs`). A dungeon item's actor is shared by every dungeon (`Item_Dungeon_SmallKey`, `Item_Dungeon_BossKey`, `Item_Dungeon_Map`, `Item_Dungeon_Compass`), so that add names no dungeon. Every other path that hands out a dungeon item writes the dungeon's own values instead: a gift scene through `gift_keys.rs` (small keys through `dungeon_credit.rs`: `DungeonInfo.<dungeon>.SmallKeyNum` +1, `SmallKeyTotalNum` +1, `IsGetSmallKeyOnce`; boss keys, maps and compasses through `dungeon_grant.rs`), and "start with" maps and boss keys through `slice/dungeon_items.rs` `start_flags`. The generic gift routine already refuses a bare pouch add of an `Item_Dungeon_*` actor (`slice/flow_rules.rs` `gift_form`); the new-game path did not.

So the logic counted a key the player never gets in its dungeon, and a seed can need it to be beaten.

### How often

Settings: default, plus Kokiri Forest open, Kakariko Gate open, Small Keys Anywhere. Seed names `r-0` to `r-199`:

| | Seeds with a dungeon item at one of the two checks |
|---|---|
| plain nightly (`dda8087`, `ootrr generate` with the reference build, spoilers counted) | 40 of 200 (9, 11, 12 and 8 per block of 50) |
| this branch (test `open_gate_seeds_*`) | 0 of 200 |

Every kind can land there. Measured with the old rule during development (not a committed test): with Small Keys, Boss Keys, Maps and Compasses and Ganon's Boss Key all Anywhere and songs shuffled anywhere, 20 of 40 seeds (`r-0` to `r-39`) put one there (15 small keys, 2 boss keys, 2 maps, 2 compasses). Ganon's Boss Key was seen there once, in a separate exploration run without the song shuffle (1 of 40); the build-refusal test covers that key. With keys in their own dungeon, 0 of 20. Small keys of the Thieves' Hideout can only stay vanilla on nightly, and Silver Rupee items only go into chests, so neither can reach these two checks. With the gate closed (the default) the two checks are not given at new game at all.

### Evidence from the built data

On plain nightly (`dda8087`) with the settings above, seed `r-10` puts "Small Key (Forest Temple)" at "HC Zeldas Letter". Its built mod's `OpeningEvent` (`ootrr bfevfl-dump`) sets `IsGet.Item_Dungeon_SmallKey` = true and adds `Item_Dungeon_SmallKey` (Num 1) to the pouch, and none of its 160 dumped lines writes a `DungeonInfo` value. An earlier build of the same code (recorded in the SoH Standard preset work, seed `r-2`, a Shadow Temple key) showed the same, and its `Dm_HrlCastle_0050`, the gift scene the skip never plays, had the credit a gift makes (`DungeonInfo.ShadowTemple.SmallKeyNum` +1, `SmallKeyTotalNum` +1, `IsGetSmallKeyOnce`). The test `nightly_seed_r_10_key_at_zeldas_letter_is_refused` pins this placement: the old rule's test (an item actor or tiers) accepts the Forest Temple key there, the fix refuses it both in the fill (`allowed`) and at build (`skipped_items`), and the start-state ops for that key add `Item_Dungeon_SmallKey` and write 0 `DungeonInfo` values.

Not seen in the game. Nobody has played a seed with a key at one of these checks; the bug is shown from the generator and the built data only.

## The fix

- `slice/place_rules.rs`, rule `skipped`: at the two checks a new game hands out, an item that belongs to a dungeon (`Item.dungeon` set: small keys, boss keys, maps, compasses, Ganon's Boss Key) is refused, as items with no actor already were (`slice::edit::given_at_start`).
- `slice/edit.rs`, `skipped_items`: a seed that still has such an item there (an old seed file, a hand-edited one) stops the build with "HC Zeldas Letter holds Small Key (Shadow Temple), which cannot be given when a new game starts".
- A plando entry that puts a dungeon item there is refused with the generator's usual plando message.

Both changes act only when the gate is open, which is off by default, so default seeds and the default build are unchanged. With the gate open and keys, boss keys, maps or compasses set to Anywhere, Overworld or Same region (whichever of those each setting offers), every seed text gives a different seed than before; with them in their own dungeon, seeds are unchanged.

### Why refuse instead of credit

The other fix would credit the dungeon at new game: the same writes the gift scene makes, added to `OpeningEvent`. It keeps the pool unrestricted (two more spots for keys), but it adds new game-data writes (a small key's count and flags, each boss key, map and compass flag, and the line that names the dungeon) that need a game test, and the new game has no get window to show which dungeon it was. Refusing is smaller, matches what `flow_rules::gift_form` already does for the other bare pouch adds, and only takes two checks away from dungeon items.

### Other things handed out at new game

| Item kind | Verdict | Evidence |
|---|---|---|
| Small keys, boss keys, maps, compasses, Ganon's Boss Key | Needs more than a pouch add: now refused at the two checks | Their actor is shared by every dungeon; every other give path writes `DungeonInfo` values (`gift_keys.rs`, `dungeon_credit.rs`, `dungeon_grant.rs`) |
| Thieves' Hideout small keys | Refused too (they have a dungeon) | They can only stay vanilla on nightly, so they never reach these checks |
| Silver Rupee items | Already refused (no actor), and only placed in chests | `slice/silver.rs` `at`; `items.json` gives them no actor |
| Treasure Chest Game keys | Not in the shuffle on nightly | No setting shuffles them |
| Gold Skulltula Token | Plain pouch add is enough | The House of Skulltula's reward branches read `PouchData.Item_Token_Gold.StockNum` (10, 20, 30, 40, 50, 100 in `GatheringTokenGold`); the token branch of `GetPouchContent` only copies that count into `UI_TokenGoldNumAfterGetting` for the get window |
| Songs (with songs shuffled anywhere) | Plain pouch add, the same as the unshuffled default | Song from Impa's own Zelda's Lullaby is already given this way when the gate is open |
| Ocarinas and other progressive items | Plain pouch add; two copies step through the tiers | Existing test `two_progressive_ocarinas_given_at_the_start_become_both_tiers` |
| Bow, Slingshot | Pouch add; the bag comes at the next item get | `starter_bags.rs`; ammo with no bag turns into rupees in `skipped_items` |
| Filled bottles, Gerudo Membership Card, Magic Meter, Double Defense, Heart Container | Left allowed; the same bare pouch add the gift scenes and Saria's gift use for them (`flow_rules::gift_takes`), plus their `IsGet` flag (`start_state::add_is_get`), which is what the game's own checks read (`IsGet.Item_Token_Gerudo`, `IsGet.Item_FairyBless_MP`, `IsGet.Item_FairyBless_DefenseDouble`) | Not checked in the game at new game; outside this fix |
| `skip_child_zelda` | Logic only, tied to the same open gate | `slice/support.rs` |
| Other new-game state (Deku Tree open, intro, owls, Temple of Time) | Flags and quest steps only, no items | `story_skip::deku`, `story_skip::start`, `start_state::first_visit` |
| Starting items (`starting_items`) | No dungeon items or tokens in its list | `settings/list.rs` |

## Tests

Offline only (`OOTRR_ROMFS` set where a test reads game files):

- `slice::place_rules::tests::open_gate_seeds_*` (4 blocks of 50, `r-0` to `r-199`): counts the dungeon items at the two checks, prints and asserts 0, and checks each seed's new-game items build. The count before the fix (40 of 200) is measured with nightly `dda8087`'s own build, above.
- `nightly_seed_r_10_key_at_zeldas_letter_is_refused`: the placement nightly's `r-10` makes (Forest Temple small key at "HC Zeldas Letter"): the old rule accepts it, the fix refuses it in `allowed` and in `skipped_items`, a Gold Skulltula Token is still allowed there, and the start state for the key writes 0 `DungeonInfo` values.
- `with_every_dungeon_item_anywhere_none_lands_at_the_new_game_checks`: every dungeon item kind Anywhere, songs anywhere, 40 seeds: 0.
- `keys_in_their_own_dungeon_never_reach_the_new_game_checks`: open gate, keys in their dungeon, 20 seeds: 0.
- `a_dungeon_item_at_a_new_game_check_stops_the_build`: a small key, a boss key, a map, a compass and Ganon's Boss Key at "HC Zeldas Letter" each stop the start state with the message; a Gold Skulltula Token there is given as `Item_Token_Gold`.
- `a_plando_key_at_a_new_game_check_is_refused`: refused with the gate open, accepted with it closed.

Results on this branch (nightly `dda8087` plus this fix): `cargo test -p ootrr-core place_rules` 9 passed; `generate` 63 passed; `settings` 113 passed; `cargo test --workspace` 2347 passed, 0 failed, 9 ignored. The upstream workflow's test step (`cargo test --release --locked -p ootrr-web -p ootrr -- stdio options_parse jobs::`, no game files) passes; the change touches no platform code.

Not tested by anyone: upstream's wasm build (`cargo build --release --locked -p ootrr --target wasm32-wasip1`) and the parity script (`site/test/parity.mjs`), because no wasm target is installed here; nothing in the game.

Output compared with the reference build of nightly `dda8087`:

- Default settings, seed `cmp-seed`: the seed file is identical and the built mod is identical, all 277 files.
- Kakariko Gate open, keys in their own dungeon: the seed files of `r-0` to `r-199` are identical, and the built mod of `cmp-seed` is identical, all 281 files.
- Kakariko Gate open, Small Keys Anywhere: all 200 seed files differ (the fill's choice of spot for every dungeon item no longer includes the two checks, so the whole shuffle moves), and none of them has a dungeon item at the two checks. A seed built by this branch passes `ootrr flow-loads`. Nightly's own `r-10` seed file, which has the Forest Temple key at "HC Zeldas Letter", is refused by this branch's build with the message above.

## How to test it in the game

Nothing new to see when it works: no key appears at the two checks. To see the old bug, use plain nightly with Kokiri Forest open, Kakariko Gate open and Small Keys Anywhere, seed `r-10` (its spoiler puts the Forest Temple small key at "HC Zeldas Letter"): start a new game, go to the Forest Temple and check whether the first locked door opens with a key that came from that check. Expected on nightly, from the data: it does not, the dungeon's key count stays 0. With this branch the same settings never put a key there, and building nightly's `r-10` seed file stops with the message above.

## MR Drafts

### Initial MR Draft
**Title:** Improved World > Kakariko Gate Fix: no dungeon items handed out at new game

With the Kakariko Gate open, whatever is placed at Zelda's Letter and the Song from Impa is given when a new game starts as a plain item add, which credits no dungeon, so a small key (or boss key, map or compass) placed there was counted by the logic but never reached its dungeon (40 of 200 seeds with Small Keys Anywhere, from the generator and the built data; not seen in the game). Dungeon items are no longer placed at those two checks, and a seed file that still has one there is refused at build. Both changes act only when the gate is open, which is off by default, so default seeds and the default build are unchanged; with the gate open and keys, boss keys, maps or compasses set to Anywhere, Overworld or Same region, every seed text gives a different seed than before. Based on nightly.

For details about implementation and testing, see difficulty-mod/docs/features/60-new-game-dungeon-item-fix.md.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
