# Difficulty settings: what does not work yet

Newest state first. "Seen" means seen in the running game. Items move to the changelog when they work.

## Broken or unsafe

Nothing known at the moment. Enemies call for backup and the spawn combination were here until rc 12; both are fixed.

## To confirm by hand

| What | State | What to do |
|---|---|---|
| Spawn enemies with L + R + D-pad | Works with scripted button presses, also with every other setting on (rc 13); did nothing on a real controller in rc 12 | Try it on a real controller. Does a kill free one of the 12 slots? |
| Enemies call for backup | Reworked and seen by script (rc 13): each kind calls its own helper after 10 to 25 seconds | Review the pairings in enemy-table.md; play it |
| Gohma's Stalfos and Keese | Created when the door shuts; the dungeon loads | Walk through the boss door: nothing before the door shuts, then they appear |
| Zora's Domain ambush | Map loads, Skulltulas present | Walk the cave |
| Dodongo guard on Death Mountain Trail | Stands at his spot | Fight him; the barrier should drop |
| Guays over child Hyrule Field | Flying by day and night | Decide if that is the amount wanted |
| Fairies help enemies | No crash; never seen doing its job | Use a fairy near a hurt enemy with the health panel on |
| The held-back gate 7 | Maps load and guards are there | Walk the Lost Woods east passage, kill the guard, see the barrier drop |
| Dead Hand in the Hyrule Field hole | Placed further back and his rise distance shortened; not seen since | Drop in and look |
| Grass that gives bugs | Joined the surprise pool; not seen | Cut the clump by the Dead Hand hole |

## Built but held back (not in this release candidate's gate list)

Gates 2, 9 and 12 were seen working and are released (gate 2 now has a Dodongo guard that is not played yet). Gate 7 is held; gates 1 and 11 are released with their fixes. A scripted run (19 Nov) showed every map loads and the guards are present; killing the guards and the barrier dropping has not been seen. They are switched on one by one as each is seen working.

| Gate | Problem seen | Fix in the code, to be played |
|---|---|---|
| 1 Sacred Forest Meadow | Only one Mad Scrub present; killing it did not open the gate | The gate creates its own Mad Scrub |
| 7 Lost Woods, east passage | Never tried | Try it |
| 11 adult Market | The barrier rose while the player was still outside it (19 Nov) | Trigger starts 3.5 units past the wall on the guard side (round 3); to be played |

## Works, with a known limit

- **Lose heart on death:** the loss lasts for the play session; quitting and continuing restores the hearts.
- **No ammo drops with shuffled shops:** a seed can end up with no source of arrows. Planned change: "limited ammo", where ammo still drops in boss fights.
- **Dead Hand in the Hyrule Field hole** rises right by the entrance, although he is placed at the back of the room.
- **Pots in the Market guard house** give no surprises.
- **Bosses spawn minibosses:** King Dodongo is left out; adding a miniboss there hung his dungeon.

## Never seen in the game

Enemy speed by hand (measured by script: most enemies ignore it, see enemy-speed.md), more enemies (the slider; a 10x rework is in progress), boss minibosses and minions beyond loading, enemies respawn sooner, Zora's Domain ambush, the ten Like Likes at Lake Hylia, fake pots.

## Ideas saved for later

- Grass that normally gives bugs joins the surprise pool.
- The Gerudo fighter as a gate guard.
- Mido as a visible blocker.
- A bomb-proof, glowing wall was replaced by the purple barrier; nothing further planned.
