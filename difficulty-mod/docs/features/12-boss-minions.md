# Bosses spawn minions

Setting `boss_minions` (number 121), checkbox. Code: `crates/core/src/difficulty/minions.rs`. Built by a sub-agent on `feature/boss-minions` (`4966a8f`).

## Request
"bosses spawn minions"

## Interpretation
Each dungeon boss fight also has a few weak enemies in the room, chosen to suit the arena.

## Investigation
- **King Dodongo already does this.** His parameter file has `SpawnBabyDodongoParams`: how many Baby Dodongos he calls per roll at each health stage (0, 2, 3 in vanilla).
- For the others, free-standing enemies in dungeon `_Dynamic` maps need no logic or group entries, so placing flying enemies in the boss room is the simplest mechanism.
- Boss intro and outro events use a setting that pauses and hides everything not in the scene, so placed minions should not act during them.
- The warp opens from the boss's own death, not an enemy count (the agent read Gohma's graph in full).

## Implementation
| Boss | Minions |
|---|---|
| Gohma | 3 Keese |
| King Dodongo | Baby Dodongos 2/3/4 per roll, up from 0/2/3 |
| Barinade | 3 Biri |
| Phantom Ganon | 3 Keese |
| Volvagia | 3 Fire Keese |
| Morpha | 3 Keese, above the water |
| Bongo Bongo | 3 Keese, above the drum |
| Twinrova | 2 Fire Keese, 2 Ice Keese |

Placed minions sit in a ring 5 to 8 m around the arena centre, 1.5 to 4 m up. They start no battle music of their own. The build stops with a clear error if a boss is not where it was measured.

Skipped: Ganondorf and Ganon (a soft-lock there costs the whole seed).

## How it was tested
- 20 difficulty tests including a game-file test.
- A real build: all 7 maps hold the minions, Dodongo's values are 2/3/4, `flow-loads` is clean.

## In the game
Never played. To test:
- Start with King Dodongo: it uses the game's own mechanism and is the safest.
- Then Gohma: 3 Keese should be flying around the room.
- Pass: minions present, fight winnable, heart container and warp appear after the boss dies.
- Report: Keese stuck in geometry, minions attacking during the intro, or minions hitting you at the warp afterwards.

## Notes
- Placed minions are not tied to the boss. They are there before the intro, stay after the boss dies, and are probably back on a later visit.
- Keese do not normally exist in the Deku Tree or Forest Temple, so those rely on the game loading the actor on demand.
- A plain Keese placed in mid-air has no vanilla example (vanilla ones hang from a surface); Fire Keese in mid-air do.
- Lesson: check whether the boss already has a spawn mechanism in its own parameters before placing anything.
