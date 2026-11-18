# Lose heart on death

Setting `lose_heart_on_death` (number 116), checkbox. Code: `crates/core/src/difficulty/flows.rs` (`lose_heart`).

## Request
"lose heart on death (checkbox) when checked, each time you get a gameover, you lose 1 of the max hearts. down to a min of 3."

After playtesting rc 1: "losing a max-heart on death didn't work. I think you'll need to change the behavior so instead of showing the 'continue from last save / go to title' screen, the behavior is different... maybe it shows the gameover screen then auto-teleports you to the recent checkpoint. is this even possible?"

## Interpretation
Each game over you continue from removes one heart container, never below three. Being revived by a bottled fairy is not a game over.

## Investigation
- The game over scene is the flow `Event/EventFlow/DmF_SY_GameOver.bfevfl.zs`. Choosing Retry runs one event that sets the result to "Retry".
- Max health is game data `PlayerStatus.MaxLife`, in quarter hearts (3 hearts is 12). Flows can add to or compare game data numbers.
- **After your playtest:** the retry itself is `Sequence/Retry.module.ainb`. It reloads the last save, which is why the loss vanished. But it first checks a flag, `Sequence_RequestLastGameOverSave`; when set, the game saves its current state before reloading. That is the game's own "keep what happened" switch.

## Implementation
On Retry the flow now does three things before carrying on:
1. Sets `Sequence_RequestLastGameOverSave`.
2. Checks whether `PlayerStatus.MaxLife` is above 12.
3. If so, subtracts 4.

Choosing "return to title" is untouched. So no teleport was needed, which avoids the fight-state problems the repo's Navi warp ran into.

## How I tested it
- Unit test: the three added events are wired in the right order and the title path is unchanged.
- The game-file test stages the real flow; `flow-loads` is clean.
- rc 1 in the game, by you: **the loss did not stick** (before the save flag was added).

## In the game
rc 2's fix has not been played, and this is the fix I am least sure of. To test:
- Start with 5 or more hearts. Save. Pick up a few rupees so you can tell whether progress is kept. Die, choose Retry.
- Pass: one heart container fewer, and the rupees you picked up are still there.
- Repeat until 3 hearts and die again: it must stay at 3.
- Die with a fairy in a bottle: no heart lost.
- Report where you respawn. I expect the usual checkpoint, since the saved position is not changed.

## Notes
- Side effect by design: with this on, a game over keeps everything you collected since your last save, like the original game.
- If the save flag turns out not to work, the fallback is your idea: revive in place or warp, using the pieces the fairy revival and fall recovery already use in this same flow.
