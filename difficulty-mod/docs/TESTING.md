# Testing the difficulty settings in the game

Nothing here is needed to use the randomizer. It is how we test. The mod itself is data only and works wherever OOTRR works; Longshot is just the most convenient place to script a playtest.

## By hand

1. Build a seed with one or two difficulty settings on (see the feature file for what to turn on).
2. Put the mod in the game's mods folder and play to the place the feature file names.
3. Report the release candidate number, the settings that were on and exactly what happened.

Change as little as possible per test. With everything on, a crash tells you nothing.

## Scripted, through Longshot's control bridge

[Longshot](https://git.magicbean.dev/MagicBeanDev/Longshot) is a PC runtime for the game, built on your own PC from your own copy of the game. It has a local control bridge that lets a script press buttons, take screenshots, teleport and read the game's state. It is not part of this repo and none of it is copied here.

### Set up Longshot

- Install it with its launcher, or build it from a clone with its own script. Its README section "Build from source" has the commands; in short: `python -I nx2/make_game.py --nup <your game file> --out <game folder> --language en-US`. It needs a recent Python, its compiler tool set and about 60 GB free while building.
- A recent version is needed for the bridge to report health, position, music and save flags. Older versions report only the map, age and whether an event is playing; with those, read the hearts from a screenshot.

### The bridge

- It is off unless the game is started with the environment variable `HOS_CONTROL` set. It is local only and for the current user only.
- The client is `scripts/longshot_ctl.py` in the Longshot repo (standard library only). Longshot's `docs/control.md` documents every command.
- Useful commands: `launch <game folder> --user <save folder>`, `status`, `press A --frames 8`, `stick L 0 1 --frames 90`, `shot <file.png>`, `teleport <map or id>`, `age adult`, `watch --events oot --flags <names>`, `quit`.

### Keep the real install safe

Never test in the folder you play in. Make a shadow game folder:

1. A new empty folder.
2. Directory junctions inside it for the large read-only folders of the real game folder (the game's `romfs` and the like).
3. Copies of the small top-level files and the executable.
4. Its own empty `mods` folder, and its own save folder passed with `--user`.

One randomizer mod at a time in the mods folder; mods load only at game start. Use a fresh save folder for each new game so starting settings apply.

### A typical scripted check

1. `ootrr generate <preset.json> "<seed>" <out.ootrr>` then `ootrr build <out.ootrr> <game romfs> <shadow mods folder> <name> --host longshot`. A preset is `{"id":"t","name":"t","tip":"t","order":99,"settings":{...}}`.
2. Launch the shadow game, press A through the title until `status` says `play`.
3. Read the starting state, do the thing, read the state again, take a screenshot.
4. Quit the game before starting the next run.

### Rules

- One copy of the game at a time.
- "Loads" is not "works": say which you saw.
- Teleports less than about eight seconds apart are refused.
- A new mod needs a restart of the game.

## What has been tested so far

See `CHANGELOG.md` for the results per release candidate, and each feature file for its own status.
