# Bosses spawn minibosses

Setting `boss_minibosses` (number 120), checkbox. Code: `crates/core/src/difficulty/minibosses.rs`. Built by a sub-agent on `feature/boss-minibosses` (`fd20c72`).

## Request
"bosses spawn minibosses"

## Interpretation
Each dungeon boss fight also has one strong enemy in the room. I told the agent a solid result for a few bosses beats a fragile one for all.

## Investigation
- The boss heart shuffle in the repo already knows the boss rooms, which was the starting point.
- Each boss room is run by a logic graph. In all five covered rooms the heart container and exit warp are created from "boss defeated" flags, never from an enemy count, so a live extra enemy should not block them.
- The game has 193 logic nodes of the simple "create this actor, delete it on a signal" kind, which gives an exact shape to copy.

## Implementation
Two files per boss:
- **Dungeon map (`_Static`):** one new enemy in the boss's room, added to the group that runs the room.
- **Room logic graph:** one new node that has the enemy present from entry and deletes it on the same signal that creates the heart container.

| Boss | Miniboss |
|---|---|
| Queen Gohma | Stalfos (child version) |
| King Dodongo | Lizalfos |
| Barinade | Stalfos (child version) |
| Phantom Ganon | Stalfos (weak) |
| Twinrova | Dinolfos |

Skipped: Volvagia, Morpha and Bongo Bongo (no safe floor), Ganondorf and Ganon (scripted), and the boss rush copies.

## How it was tested
- 21 difficulty tests including a game-file test per boss room.
- A real build: all five maps and graphs carry the edit alongside the boss heart edit, the graph audit is clean, `flow-loads` is clean.

## Result of the scripted load test (18 Nov)
- With this setting on, **Dodongo's Cavern went to a black screen** in rc 4 (reproduced with this setting alone; the cavern loaded with every other setting on). No crash line in the log; the game kept running but Link never appeared.
- The Deku Tree, Jabu-Jabu, Forest Temple and Spirit Temple loaded with it on.
- King Dodongo was removed from the list in rc 5. Which of his three edited files is at fault (the added Lizalfos, the room logic graph, or the map file) was not isolated.
- Loading is all that was checked: nobody has seen a miniboss in a boss room yet.

## In the game
Never played by hand. To test:
- Turn this on alone and go to Gohma (shortest route).
- Pass: a Stalfos is in the room, the fight is winnable, and after Gohma dies the Stalfos vanishes and the heart container and blue warp appear.
- Report: a crash entering the dungeon (the graph edit is the suspect), the miniboss attacking during the intro cutscene, or it standing in a bad place (King Dodongo's lava ring, Barinade's arena).

## Notes
- This is the first time the repo adds a whole node to a logic graph, and the first appended actor in a `_Static` map. If loading the dungeon crashes, that is why.
- The miniboss is in the room before the intro triggers. Whether cutscenes pause it is inferred, not seen.
- In Jabu, Forest and Spirit the delete signal is a quest step. If a seed starts past that step, the miniboss never appears.
