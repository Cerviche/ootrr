# Difficulty settings: changelog

One entry per release candidate (rc), newest first. Each rc is one commit on the `difficulty` branch. "Played" means seen in the running game, by hand or by a script through Longshot's control bridge (see [TESTING.md](TESTING.md)). Feature details are in [features/](features/).

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
