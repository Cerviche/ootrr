# OOTRR in Archipelago

OOTRR can play as one game in an Archipelago multiworld. The items of your shuffled chests, NPC rewards and pickups can belong to other players, and their games can hold yours. This support is experimental. The first test card passed in Longshot on 2026-11-24.

## What you need

- Archipelago 0.6.7 or newer.
- `ootrr.apworld`. The app's Archipelago page installs it or saves it for you; `tools/build_apworld.py` builds the same file from this folder.
- The OOTRR app (`ootrr.exe`) of the same version as the apworld.
- Your game dump and Longshot, as for a normal OOTRR seed.

## The quick way: the app's Archipelago page

The OOTRR app has an **Archipelago** page (in the menu, under Configuration) that does the steps below for you:

1. **Set up Archipelago:** pick your Archipelago folder. The app puts the OOTRR world in `custom_worlds` and writes its own place into `host.yaml`. Do it again after you update the app.
2. **Make your player file:** type your player name and save the file. It names the preset your settings match, or carries your settings string. The host's copy of the world can be saved from here too.
3. **Build your mod:** open the `.apootrr` file the host sent you. The mod goes to the mod folder of the Configuration page.
4. **Play:** type the room's address and open the OOTRR Client.

## Steps by hand

1. Put `ootrr.apworld` in Archipelago's `custom_worlds` folder.
2. Write your player file with `game: OOTRR` and pick a preset:

   ```yaml
   name: YourName
   game: OOTRR
   OOTRR:
     preset: default
   ```

   The presets are OOTRR's own: `default`, `easy`, `keysanity`, `full_shuffle` and `open_world`.

   To use your own settings instead, copy the settings string from the OOTRR app:

   ```yaml
   OOTRR:
     settings_string: "<your OOTRR settings string>"
   ```

   The settings string replaces the preset. The computer that generates the room then runs the OOTRR app to work out your world, so the host needs `ootrr.exe` and its path under `ootrr_options` in `host.yaml` (Archipelago asks for it the first time). Entrance shuffle is refused.
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

- Entrance shuffle is not supported.
- A settings string needs the OOTRR app on the computer that generates the room.
- With pots, crates, grass and other new kinds of checks, many locations cannot be seen by the client yet. They hold only your own items.
- Items sent by the server itself (`!getitem`) or by item links are not handed over.
- Dungeon items stay in their own world unless their setting is Anywhere. Hideout keys and key rings always stay.
- Songs, NPC gifts and boss rewards hold only your own items for now, because the client cannot see those spots on their own yet.
- No DeathLink.
