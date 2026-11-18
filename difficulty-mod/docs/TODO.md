# Difficulty settings: what does not work yet

Newest state first. "Seen" means seen in the running game. Items move to the changelog when they work.

## Broken or unsafe

Nothing known at the moment. Enemies call for backup and the spawn combination were here until rc 12; both are fixed.

## To confirm by hand

| What | State | What to do |
|---|---|---|
| Spawn enemies with L + R + D-pad | Works with scripted button presses | Try it on a real controller. Does a kill free one of the 12 slots? |
| Enemies call for backup | Works in a scripted run; the wait is about 12 seconds, not the 25 intended | Decide: keep 12 seconds or go to 25 |
| Fairies help enemies | No crash; never seen doing its job | Use a fairy near a hurt enemy with the health panel on |
| The six held-back gates | Maps load and guards are there | Kill the guards, see the barrier drop, then switch each gate on |
| Dead Hand in the Hyrule Field hole | Placed further back and his rise distance shortened; not seen since | Drop in and look |
| Grass that gives bugs | Joined the surprise pool; not seen | Cut the clump by the Dead Hand hole |

## Built but held back (not in this release candidate's gate list)

All six have their fix in the code. A scripted run (19 Nov) showed every map loads and the guards are present; killing the guards and the barrier dropping has not been seen. They are switched on one by one as each is seen working.

| Gate | Problem seen | Fix in the code, to be played |
|---|---|---|
| 1 Sacred Forest Meadow | Only one Mad Scrub present; killing it did not open the gate | The gate creates its own Mad Scrub |
| 2 Death Mountain Trail | All three Tektites killed; the gate stayed shut | The gate creates its own Tektite |
| 7 Lost Woods, east passage | Never tried | Try it |
| 9 Lost Woods, the small room | Worked with two Wolfos; the user wanted one guard | One Stalfos for both barriers |
| 11 adult Market | The barrier rose while the player was still at the entrance | Triggers only from the guard's side; the barrier closes the exit behind the player |
| 12 Gerudo Valley | The Gerudo fighter plays her fortress scene and may jail Link | Two Stalfos; the Gerudo fighter is saved for later |

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
