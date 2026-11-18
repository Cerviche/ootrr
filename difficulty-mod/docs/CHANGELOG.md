# Difficulty settings: changelog

One entry per release candidate (rc), newest first. Each rc is one commit on the `difficulty` branch. "Played" means seen in the running game, by hand or by a script through Longshot's control bridge (see [TESTING.md](TESTING.md)). Feature details are in [features/](features/).

## rc 7

- **No bottles during combat:** Leevers and Moblins, the only enemies that fought without battle music, now start it when this setting is on, so the music check covers them. See [features/08](features/08-no-bottles-in-combat.md).
- **No dupe items:** a refused catch or hand-over now ends properly (head shake, then the same closing steps the game uses) instead of ending the event at its start. See [features/13](features/13-no-dupe-items.md).
- Played: nothing yet.

## rc 6

- **Lose heart on death, third version:** the heart is taken through the player's own "set max health" action on Retry. See [features/07](features/07-lose-heart-on-death.md).
- **New: Enemies block the way.** First gate: the path to the Deku Tree stays walled while the two withered Deku Babas beside it are alive. Nothing is saved. Skipped with an open forest or entrance shuffle. See [features/20](features/20-enemies-block-the-way.md).
- Played: nothing yet.

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
