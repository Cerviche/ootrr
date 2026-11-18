# More enemies

Setting `more_enemies` (number 119), choice: Off, Half again, Double. Code: `crates/core/src/difficulty/extras.rs`. Built by a sub-agent on `feature/more-enemies` (`d1c7b84`).

## Request
"more enemies (spawn extras)"

## Interpretation
Extra copies of enemies that are already placed in a map, standing near the originals. The agent made it a three-way choice.

## Investigation
- Enemies are entries in the map files (`Banc/...bcett.byml.zs`). 1,503 `Enemy_*` actors are placed.
- The main risk is soft-locks: a room that opens when its enemies die. The agent searched all 1,824 logic graphs and found rooms count specific linked enemies (through `Obj_EnemyCounter_*` and enemy groups), never "everything in the room".
- So a copy that nothing links to is never counted.

## Implementation
An enemy gets a copy only if all of these hold:
- its type is on a list of 21 ordinary enemies (Keese, Octorok, Tektite, ReDead, Dodongo, Like Like, Deku Baba and so on);
- nothing anywhere refers to it;
- it has no links, rails, name or event ties;
- it is not in the boss rush maps.

The copy is appended to the same map with fresh ids, 1.0 to 1.5 m away at the same height. Placement is seeded from the seed text, so the same seed gives the same copies. It runs before the boss features so their additions are not doubled.

## How it was tested
- 19 difficulty tests, including one against your game files: originals untouched, copy ids unique, distance limit, same result every run.
- Real builds, Double against Off: identical item placements; exactly 23 map files and the size table differ.
- "Double" adds 221 enemies. See `<builds>\rc-4\entity-maps\` for where.

## In the game
Never played. To test:
- Turn on Double alone. Go to Hyrule Field as adult (35 added), Dodongo's Cavern (28) or Fire Temple (26).
- Pass: clearly more enemies than usual, and every door that opens on "defeat all enemies" still opens.
- Look for copies stuck in walls, floating, or over pits; they are placed without collision data. Use the entity maps to know where to look.
- Reload a room after killing a copy: it will probably be back, because copies have no save flag.

## Notes
- Skipped on purpose: bosses, minibosses, Gold Skulltulas, Deku Scrubs, anything in an enemy group (555 enemies), and solid or wall-mounted enemies (Armos, Beamos, Skulltulas).
- Up to 35 extra actors in one map is untested for memory.
- With the object map site or parsed collision, placement could be checked and improved.
