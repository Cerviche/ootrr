# Difficulty settings: changelog

One entry per release candidate (rc), newest first. Each rc is one commit on the `difficulty` branch. "Played" means seen in the running game, by hand or by a script through Longshot's control bridge (see [TESTING.md](TESTING.md)). Feature details are in [features/](features/).

## rc 12

Results here come from scripted runs in the game unless they say otherwise. Enemy health can now be read during tests, which is what made several of these checks possible.

- **Enemies call for backup: fixed and seen working.** It crashed the game seconds after an enemy appeared; the cause was how its two flags were stored. Now: no crash through 90 seconds beside two Lizalfos, and each called one Peahat larva, once, at 10 and 16 seconds. The wait is about 12 seconds, not the 25 intended. See [features/17](features/17-enemies-call-for-backup.md).
- **Spawn enemies with L + R + D-pad: fixed and seen working** with scripted button presses: 12 presses gave 12 enemies in front of Link, D-pad Up repeated the last kind, and further presses gave nothing (12 alive at once). Before, only the first press ever worked. Not yet tried on a real controller. See [features/25](features/25-spawn-enemies-combo.md).
- **Enemy regen: seen working.** A Stalfos hit from 18 to 11 waited four seconds, then healed 1 about every two seconds back to 18; with the setting off it stayed at 12 for a minute.
- **Enemy health: seen working by the numbers.** At 2x a Stalfos has 36 and a Lizalfos 18.
- **Like Likes at Lake Hylia: present.** The map loads and they are at their spots.
- **"Red gate guards" was not a bug.** The big Deku Baba on the Deku Tree path has a red head with every difficulty setting off.
- **Gates:** unchanged from rc 11 (six on). The six held back all load their maps and show their guards; killing the guards and seeing the barrier drop is still to be done by hand.

## rc 11

- **Merged upstream** (32 commits; the repository layout changed: `crates/` is now `randomizer/`). All upstream tests and ours pass.
- **Surprises from grass and pots** are now 1 in 50 for each kind, and grass releases the wall Skulltula (on the ground) or a little Floormaster hand; pots a Poe or a little hand. Played by hand at a raised test rate: "perfect". The clumps of grass that always give a bug (for example round the Dead Hand hole in Hyrule Field) now join the pool; not played yet.
- **Cuccos take revenge at once** now covers Kakariko's cuccos too. Played: one hit brings the swarm.
- **Dead Hand** in both holes: placed further back and his rise distance shortened. Not played since; before this he still rose near the entrance.
- **Gates:** seven were in rc 10; six are in this rc. The Lost Woods room was changed to one Stalfos for both barriers and is held back until that is seen working, along with the five held back before (which got their fixes: own guards for the meadow and the trail, two Stalfos in Gerudo Valley, a rule that a gate only triggers from the guard's side, the Hyrule Field barrier half a unit lower). None of the fixes is played yet.
- **Enemies call for backup crashes the game** (found by a scripted test, twice). Leave it off. Regen alone and fairies alone did not crash.
- Docs: the status board, the capture playbook and the code review are now in this folder too.

## rc 10

Played by hand unless it says otherwise. Things known not to work are listed in [TODO.md](TODO.md).

- **Enemies block the way, rebuilt.** A gate is the game's own purple barrier. Nothing is there until you come near its guard; then the barrier rises and the guard appears with it; it drops when the guards are dead. Seven gates are in: three on Zora's River (Lizalfos; the upper one has two), the Deku Tree path (the withered Deku Babas, closed forest only), the small room in the Lost Woods (two barriers), and the road to Gerudo Valley in adult Hyrule Field (an Iron Knuckle). All seven were seen working. Five more are built but held back until they work (see TODO). See [features/20](features/20-enemies-block-the-way.md).
- **New in special encounters: grass and pot surprises.** Cut grass can release a wall Skulltula or a little Floormaster hand; a broken pot a Poe or a little hand; and about one plain pot in ten is a flying pot. The chance is 1 in 100 in this rc (1 in 50 is planned). Grass surprises were seen working at a raised test rate. Pots in the Market guard house gave nothing; not understood yet. See [features/23](features/23-grass-and-pot-surprises.md).
- **New in special encounters: ten Like Likes around Lake Hylia.** Not played yet.
- **Special encounters:** the Iron Knuckle in Hyrule Field is awake from the start ("perfect"). Dead Hand in the Hyrule Field hole was moved back twice and still rises near the entrance.
- **New: Cuccos take revenge at once.** One hit brings the swarm. Seen working in Kakariko. The two minigame cucco kinds are left out. See [features/24](features/24-cucco-revenge.md).
- **Enemy regen, fairies help enemies, call for backup** now run beside the game's damage-flash routine instead of inside it. Not confirmed: with these on, killing a gate guard crashed the game once. Leave them off for now.
- **New setting that does not work yet: spawn enemies with L + R + D-pad.** The checkbox is there; pressing the buttons does nothing. See [features/25](features/25-spawn-enemies-combo.md).
- Review fixes: a broken gate or shop setting is reported instead of ignored; the battle-music change is pinned to its five enemies; tips corrected.

## rc 9

- **Lose heart on death now works.** The heart is taken after the Try Again reload, in an event the game runs once the map is back. Played by script and by hand: 6 → 5 → 4 → 3 hearts, a further death stays at 3, a fairy revival costs no heart. Limit: the loss lasts for the play session; quitting and continuing restores the hearts. See [features/07](features/07-lose-heart-on-death.md).
- **No bottles during combat** also refuses bottles while miniboss music plays (it only knew the ordinary battle music). Played by hand against the Iron Knuckle: refused while he is aggro, allowed again at a distance, and a bottled fairy still revives Link in the fight. See [features/08](features/08-no-bottles-in-combat.md).
- **New: More enemies: special encounters** (one checkbox). A Dead Hand waits in two holes (Hyrule Field east, Death Mountain Trail) and an Iron Knuckle stands in the open part of Hyrule Field, awake from the start. Played: both maps load as child and adult; both Dead Hands rise and attack; the knight wakes without being hit. Dead Hand was then moved further from the landing spot, and "awake from the start" replaced "wakes when near"; those two changes are not played yet. See [features/22](features/22-special-encounters.md).
- **Enemies block the way:** the wall became two spider webs in front of a solid wall. Played by hand: the web blocks but can be bombed, and it sat on top of its guards. It is being rebuilt on positions marked in the game; treat this setting as unfinished in this rc. See [features/20](features/20-enemies-block-the-way.md).
- Confirmed in play with no code change: no Z-targeting, no dupe items (a second fairy cannot be bottled), no gossip fairies (six songs tried), no heart drops and no ammo drops (about 20 grass: rupees and bugs only), the Bean Seller's pitch.
- Known problem: with enemy regen, call for backup or fairies help enemies on, enemies can look tinted red. Cause not found yet.
- Not played yet: enemy speed, enemy regen, more enemies, boss minibosses and minions, enemies respawn sooner, fairies help enemies, call for backup, Zora's Domain ambush.

## rc 8

- **Enemies call for backup**, reworked. See [features/17](features/17-enemies-call-for-backup.md).
  - Strictly once per enemy: no second call, even after the helper dies.
  - Bosses, minibosses, Iron Knuckles, Dead Hand and Phantom Ganon never call.
  - The helper fits the caller: Shabom for Jabu-Jabu's enemies, a flying pot for the Ice Cavern, Shadow and Spirit Temple enemies, a Peahat larva for the rest.
- Played: nothing yet.

## rc 7

- **No bottles during combat:** Leevers and Moblins, the only enemies that fought without battle music, now start it when this setting is on, so the music check covers them. See [features/08](features/08-no-bottles-in-combat.md).
- **No dupe items:** a refused catch or hand-over now ends properly (head shake, then the same closing steps the game uses) instead of ending the event at its start. See [features/13](features/13-no-dupe-items.md).
- Played: nothing yet.

## rc 6

- **Lose heart on death, third version:** the heart is taken through the player's own "set max health" action on Retry. See [features/07](features/07-lose-heart-on-death.md).
- **New: Enemies block the way.** First gate: the path to the Deku Tree stays walled while the two withered Deku Babas beside it are alive. Nothing is saved. Skipped with an open forest or entrance shuffle. See [features/20](features/20-enemies-block-the-way.md).
- Played by script (rc.4 Longshot):
  - Lose heart on death still fails: 6 hearts before and after one game over and Try Again. The save flag read 0 before, at game over and after, so the added steps did not run at all.
  - Enemies block the way: Kokiri Forest loads, and the wall stops Link at the expected line with both Deku Babas alive (three attempts). Killing the Babas and walking through was not tested.
  - King Dodongo's black screen (rc 4 builder) comes from the edited static map file of Dodongo's Cavern: the cave loads without it, and stays black without either of the other two edited files.
  - Not tested: red enemies, Zora's Domain ambush.

## rc 5

- **Bosses spawn minibosses** no longer touches King Dodongo: his dungeon loaded to a black screen with it on.
- Played by script (rc.4 Longshot):
  - Enemy damage 3x works: a Deku Baba bite took 2 quarter hearts at 1x and 6 at 3x.
  - Dodongo's Cavern loads with boss minibosses and minions on.
  - Combat music is named `BGM_EnemyBattle`; it starts when an enemy notices Link and stays on until the map changes or Link dies.
  - Lose heart on death failed: 6 hearts before and after two game overs.

## rc 4

- Four more settings, 19 in total: **Enemies call for backup**, **Zora's Domain ambush**, **Fairies help enemies too**, **Bean Seller's sales pitch**.
- Played by script (rc.3 Longshot), load test only: Kokiri Forest, Zora's Domain, Hyrule Field, the Deku Tree and Fire Temple load with nine map and enemy settings on. Dodongo's Cavern went to a black screen, traced to boss minibosses.

## rc 3

- First combined candidate, 15 settings. Added: **Enemies regen health**, **No Z-targeting**, **More enemies**, **Enemies respawn sooner**, **No dupe items**, **Bosses spawn minibosses**, **Bosses spawn minions**.
- Edits that touch the same file now build on each other.
- Played: nothing.

## rc 2

- Fixes from the rc 1 playtest:
  - **Bottles:** the combat check was inverted, and a refused bottle froze Link.
  - **No heart / ammo drops:** the game's global drop lottery added drop chance at low health or ammo; those boosts are zeroed and the lottery's heart and ammo actors point at the green rupee.
  - **Lose heart on death:** Retry now asks the game to save before it reloads.
- The enemy sliders read as multipliers.
- Played: nothing.

## rc 1

- New Difficulty page with eight settings: **enemy health**, **enemy damage**, **enemy speed**, **no heart drops**, **no ammo drops**, **no gossip fairies**, **lose heart on death**, **no bottles during combat**.
- Played by hand: enemy health scaling works. Bottles never worked and froze Link, hearts and ammo still dropped, lose heart on death did not stick.
