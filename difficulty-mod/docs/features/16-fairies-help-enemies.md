# Fairies help enemies too

Setting `fairy_helps_enemies` (number 125), checkbox. Code: `crates/core/src/difficulty/fairy.rs` and `flows.rs` (`fairy_used`). Built by a sub-agent on `feature/fairy-helps-enemies` (`df5f039`).

## Request
"fairies heal / respawn enemies. if a fairy gets used, all the enemies also heal, and the most recently defeated enemy gets resapwned"

## Interpretation
"Used" means releasing a bottled fairy to heal, or being revived by one on death. Two halves: (a) every living enemy nearby heals to full, (b) the last enemy you defeated comes back. I told the agent (b) was probably much harder and a solid (a) alone was acceptable.

## Investigation
- **(a)** needs a signal from the fairy events to every enemy. Events can set a game data flag; enemy AI can read one. The flag chosen is `MiniGame_Diving_Test`: it exists, is unsaved, defaults to false, and nothing in the game refers to it.
- "Heal to full" is the same heal node regen uses, in mode 1. The player's "heal to max" event is wired exactly that way.
- **(b)** A defeated enemy is deleted, so none of its own logic is left to run. No event action or AI node can name "the last one beaten" or recreate a placed enemy. A "this room's enemies come back" variant had no safe hook either.

## Implementation
- **Events raise the flag:** one added event in the release-fairy flow and one in the fairy revival chain of the game over flow.
- **Enemies poll it:** the shared enemy graph (the one regen uses) gains 15 nodes. While the flag is set, a living enemy heals to full. A 30-frame timer then clears the flag.
- Enemies clear the flag, not the event, because it is unknown whether enemy AI runs during the event.
- **(b) is not implemented.**

## How it was tested
- 26 difficulty tests, with node shapes compared against game-authored nodes.
- A real build: both flows carry the new event, `flow-loads` is clean, and with lose-heart and no-bottles also on each flow carries all edits.

## In the game
Never played. To test:
- Turn this on with enemy health at 3x. Damage a Stalfos or Lizalfos, then release a bottled fairy nearby.
- Pass: the enemy takes a full set of hits again.
- Also die with a fairy in a bottle next to a damaged enemy.
- Touching a wild fairy should not heal enemies.

## Notes
- If no enemy is awake when the fairy is used, the flag stays set and the next enemy to wake heals instead. A reload clears it.
- Not covered: Beamos and other enemies that do not run the shared graph.
- Must be applied after regen in the build (it wraps whatever it finds; regen expects the original graph).
- For (b), the control bridge's memory access or a code patch would be the only routes I can see.
