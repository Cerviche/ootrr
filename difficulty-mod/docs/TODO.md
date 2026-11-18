# Difficulty settings: what does not work yet

Newest state first. "Seen" means seen in the running game. Items move to the changelog when they work.

## Broken or unsafe: leave these settings off

| What | What happens | Next step |
|---|---|---|
| Enemies call for backup | **Crashes the game.** Scripted test, run twice (18 Nov): with backup on, alone or with the others, the game crashes within seconds of an enemy appearing near Link; with regen alone, fairies alone, or nothing on, the same test never crashes. Its helper has never been seen | Fix how backup stores its two flags on enemies (a fix was started and is saved in `playtest
ext-fixes-wip.patch`), then rerun `playtest\crashtest` |
| Enemy regen, fairies help enemies | No crash in the scripted test (spawn an enemy, Din's Fire, 30 s nearby). Neither has been seen doing its job: enemy health cannot be read yet | Show enemy health in the Longshot debug panel (in progress), then check healing |
| Spawn enemies with L + R + D-pad | Nothing happens on any button combination. An earlier version crashed; that crash is fixed | Find which link fails: reading the controller from Link's script, or Link creating an enemy |
| Gate guards look red | The Deku Baba guarding the Deku Tree path looks bright red. Nothing in the gate code tints a guard, and created guards are not red, so this may be how that plant looks | Compare with the same plant in a game with no difficulty settings |

## Built but held back (not in this release candidate's gate list)

All six have their fix in the code; none has been played since. They are switched on again one by one as each is seen working.

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
