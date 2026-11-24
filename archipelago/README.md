# OOTRR in Archipelago

OOTRR can play as one game in an Archipelago multiworld. The items of your shuffled chests, NPC rewards and pickups can belong to other players, and their games can hold yours. This support is experimental and not tested in the game yet.

## What you need

- Archipelago 0.6.7 or newer.
- `ootrr.apworld`, built from this folder by `tools/build_apworld.py`.
- The OOTRR app (`ootrr.exe`) of the same version as the apworld.
- Your game dump and Longshot, as for a normal OOTRR seed.

## Steps

1. Put `ootrr.apworld` in Archipelago's `custom_worlds` folder.
2. Write your player file with `game: OOTRR` and pick a preset:

   ```yaml
   name: YourName
   game: OOTRR
   OOTRR:
     preset: default
   ```

   The presets are OOTRR's own: `default`, `easy`, `keysanity`, `full_shuffle` and `open_world`.
3. The host generates the multiworld as usual. Each OOTRR player gets a `.apootrr` file in the output.
4. Build your mod from it:

   ```
   ootrr ap-build <your file>.apootrr <romfs folder> <Longshot mods folder> --host longshot
   ```

   The build checks the file with OOTRR's own logic first and refuses a world it cannot beat.
5. Install the mod and start a new save. The mod marks the save with your slot's seed.
6. Open the Archipelago Launcher and start **OOTRR Client**. Connect to the room with your slot name. The first time, it asks where `ootrr.exe` is.

## How it works

- Your checks are read from the running game's memory. Chests use their own opened flags, and other locations use the flags and object keys OOTRR's tracker knows. A location OOTRR cannot see yet holds only your own items, so nothing another player needs can hide there.
- An item from another world is handed over by the mod itself. The client sets a flag in memory, the mod plays the item get, then sets a saved flag that says you have it. Loading an older save gives back only what that save is missing.
- Items for other players show as an Archipelago item with the item's name and its player.

## Limits for now

- Settings come from the presets only, and entrance shuffle is not supported.
- Items sent by the server itself (`!getitem`) or by item links are not handed over.
- Dungeon items stay in their own world.
- No DeathLink.
