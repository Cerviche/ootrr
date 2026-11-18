# Difficulty settings: what does not work yet

Newest state first. "Seen" means seen in the running game. Items move to the changelog when they work.

## Broken or unsafe: leave these settings off

| What | What happens | Next step |
|---|---|---|
| Enemy regen, fairies help enemies, enemies call for backup | With all three on, killing a gate guard crashed the game (once; the same kill is fine with only gates on). None of the three has been seen doing its job | Find the crash (the suspect is how their logic is now hosted in other scripts); then test each alone |
| Spawn enemies with L + R + D-pad | Nothing happens on any button combination. An earlier version crashed; that crash is fixed | Find which link fails: reading the controller from Link's script, or Link creating an enemy |
| Gate guards look red | Enemies the game already places, once adopted as gate guards, are tinted bright red (the Deku Babas on the Deku Tree path). The gate works | Find what the gate does to an adopted guard |

## Built but held back (not in this release candidate's gate list)

| Gate | Problem seen | Planned fix |
|---|---|---|
| 1 Sacred Forest Meadow | Only one Mad Scrub present; killing it does not open the gate | The gate creates its own guard |
| 2 Death Mountain Trail | All three Tektites killed; the gate stays shut | The gate creates its own guard |
| 7 Lost Woods, east passage | Not tried | Try it |
| 11 adult Market | The barrier rose while the player was still at the entrance, on the far side | Trigger only on the guard's side; the barrier closes the exit behind the player |
| 12 Gerudo Valley | The Gerudo fighter plays her fortress scene when beaten and may jail Link | Two Stalfos instead; fix the Gerudo fighter later |

Other gate changes wanted: one Stalfos for the two Lost Woods barriers (two guards is too many); the Hyrule Field barrier half a unit lower; every gate triggers only from the guard's side.

## Works, with a known limit

- **Lose heart on death:** the loss lasts for the play session; quitting and continuing restores the hearts.
- **No ammo drops with shuffled shops:** a seed can end up with no source of arrows. Planned change: "limited ammo", where ammo still drops in boss fights.
- **Dead Hand in the Hyrule Field hole** rises right by the entrance, although he is placed at the back of the room.
- **Pots in the Market guard house** give no surprises.
- **Bosses spawn minibosses:** King Dodongo is left out; adding a miniboss there hung his dungeon.

## Never seen in the game

Enemy speed, more enemies (the slider), boss minibosses and minions beyond loading, enemies respawn sooner, Zora's Domain ambush, the ten Like Likes at Lake Hylia, fake pots.

## Ideas saved for later

- Grass that normally gives bugs joins the surprise pool.
- The Gerudo fighter as a gate guard.
- Mido as a visible blocker.
- A bomb-proof, glowing wall was replaced by the purple barrier; nothing further planned.
