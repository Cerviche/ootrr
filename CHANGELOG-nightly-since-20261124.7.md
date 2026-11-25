# OOTRR nightly: changes since 0.8.3-nightly.20261124.7

Covers the nightly branch from `dda8087` (the source of nightly 0.8.3-nightly.20261124.7) to the first 0.9.0 nightly: PRs #123 to #127, plus one commit that makes the branch build again after the merges.

**Nightlies are now numbered 0.9.0-nightly.<date>.<run>**: they lead up to 0.9.0 instead of 0.8.3. Builds made from this nightly on are told about 0.9.0 when it is released, and are not offered 0.8.x patch releases.

Location names, logic for many overworld and dungeon checks, the Dampé crash, voided dungeon keys, child and adult trade items, saving after Navi warps, the Hyrule Field sun and a first Archipelago build. Seeds differ from 0.8.3-nightly.20261124.7 because locations were renamed and logic changed.

**Only part of this has been checked in the game** (noted per entry). Back up your saves.

## Location names and logic (#126)

- **Location names match what is in the game.** Chests, freestanding rupees and hearts, Pieces of Heart and Gold Skulltulas are named for the spot they really are at in the remake, and grotto names say which grotto and where. Hyrule Field, Lake Hylia and Lon Lon Ranch are named apart. For example, "HF Tektite Grotto Freestanding PoH" is now "HF Rocks Near Cow Grotto Chest". The old N64 names are gone everywhere (spoiler, tracker, plando); plandos and settings that name locations must use the new names.
- **N64-only locations removed:** 144 checks the remake does not have (freestanding items and four grotto chests) are no longer in the logic.
- **Logic fixes** across Hyrule Field, the grottos, Goron City, Death Mountain, Zora's Fountain, Lon Lon Ranch and the Bottom of the Well, Deku Tree, Dodongo's Cavern, Forest, Fire, Water, Shadow and Spirit Temples, Ice Cavern, Gerudo Training Ground and Ganon's Castle.
- **Excluding locations** no longer makes a seed impossible to generate.

## Fixes (#126)

- **Dampé's first dig (child, Graveyard)** no longer crashes the game. The Heart Piece now comes from the first dig at one of the soils where vanilla hides it, not from any soil.
- **Builds from a .nup or .nx dump** no longer fail with some settings (no heart drops, harp colours, enemy effects, adult trade shuffle, Link the Goron): the unpack now takes the files only those settings read.
- **Dungeon keys and boss keys that vanished** when picked up: the item packs carry their dungeon again. *Reported as "sometimes"; not confirmed fixed in game.*
- **Child and adult trade items** (with Shuffle adult trade items): a child sees only the child items (Weird Egg, Chicken, Zelda's Letter) and an adult only the adult ones. Before, an adult trade item found as a child took the slot of Zelda's Letter or the Weird Egg and blocked the child's quest. A child can switch between child trade items with A on the menu slot, like an adult. *Not checked in game.*
- **The Pocket Egg is no longer a separate item:** Anju's first adult gift is the Pocket Cucco.

## Kakariko Gate open (#124)

- With the gate open, the two checks given at new game (Zelda's Letter and Impa's song) no longer hold a dungeon item. Before, a key there was given with no dungeon credited, while the logic counted it (40 of 200 test seeds with keys anywhere).

## Return with Navi (#123)

- **Saving keeps working** after a Navi trip out of a room that locked its doors behind you (45 such doors across 13 maps). Before, the game stopped autosaving and could not be saved by hand until a game over or a reload. Fixes #71 and #72. *Checked with the scripted game test.*
- A trip never carries a minigame timer into the next map.

## Hyrule Field sun (#122)

- **The sun no longer stays stuck over Death Mountain** in Hyrule Field and Lake Hylia after Saria's gift, mostly with Kakariko Gate open. Saves that already have it are repaired on the next walk into the field. *Checked in game.*

## Longshot (#127)

- The mod is written to `mods/<name>` without the title ID folder, as the latest Longshot loads mods. Hookshot and NX2navi are unchanged.

## Archipelago (#125)

- First Archipelago support: an `ootrr.apworld` for Archipelago 0.6.8, `ootrr ap-export` (logic bundles), `ootrr ap-build` (builds the mod from an `.apootrr` patch), and `ootrr ap-bridge` with a client that sends checks and hands out received items. Items from other worlds show as an Archipelago medallion. Any OOTRR settings string works, not only the presets. See `archipelago/README.md`. *Nothing is checked in game.*

- **Archipelago page in the app** (menu, under Configuration), in four steps. *Set up Archipelago* finds your Archipelago folder, puts the OOTRR world in `custom_worlds` and tells Archipelago where the app is. *Make your player file* saves your player file (a preset, or your own settings) and the world file for a host who does not use the app. *Build your mod* opens your `.apootrr` file and builds into the Configuration page's mod folder. *Play* opens the OOTRR Client with the room address and your name. The app now packs the world file itself, so no Python is needed. *Tried against a stand-in Archipelago folder only: not yet with a real Archipelago install or a real room.*
- `ootrr --help` lists `ap-export`, `ap-build` and `ap-bridge`.

## Settings

- **Enemy randomizer** is shown again on the Difficulty page (it was hidden in 0.8.3-nightly.20261124.7). The page still carries the warning about unexpected behavior.
- **Bombchus in logic is off for everyone.** It stays hidden, and settings strings that ticked it still load, but the logic never counts Bombchus as explosives without a Bomb Bag.

## Build

- The merges left `slice/edit.rs` with two versions of the seed build, so nightly did not compile. Both are kept in one: the Archipelago build hook and the post-build pass from #127 now run for every build, `ap-build` included.
- A typo in the Death Mountain Trail logic (the Eyedrops trade rule, from #126) stopped every seed from generating. Fixed.
