# Capture playbook: things that need the user's hands

Scripted tests never wait for the user. When a step needs a person at the controls, the test marks it NOT TESTED, adds a row here and carries on. When the user is back, we run through this list in one session: the script warps and sets up, the caption says what to do, and the recorder captures the input for later replay.

Each row: what to do, where (map and position), the setup the script does first, and what we record or look for.

| # | For feature | What the user does | Where | Setup by script | We capture / look for | Status |
|---|---|---|---|---|---|---|
| 1 | Test tooling | Walk forward and swing the sword for about 10 seconds | Kokiri Forest, near Link's house | Sword equipped | First real recording, replay drift check | Open |
| 2 | Test tooling | Equip sword and shield from the pause menu | Anywhere | New game with sword and shield in starting items | A replayable "equip" routine | Open |
| 3 | Lose heart on death, others | Die to a Deku Baba as fast as possible | Kokiri Forest or inside the Deku Tree | Warp next to the Baba (measured: Deku Tree baba room, warp 5.5 100.2 -1.3, no stick input, re-warp after each hit; scripts can do it) | A replayable "die here" routine | Open |
| 4 | Enemies block the way | Kill the two withered Deku Babas by the Deku Tree path, then walk through | Kokiri Forest, Babas near x 38, wall at z about -19 | Closed forest, sword equipped, warp past Mido (NOT TESTED by heart-v4: killing the Babas needs sword play the script could not do) | Whether the wall opens only after both die; a replayable kill routine | Hands-on: user walked through at z -19.08 with Babas alive and battle music (see playtesthandsonRESULT.md): NO wall; feature being redesigned with a visible wall. gate.seq captured (truncated) |
| Song of Storms holes | Play Song of Storms at hole spots (HF: no cover holes such as Lost Woods hole 2 and others), record routine | map and x y z from playtest\holes\INDEX.md | ocarina + song via give, record | note buttons |
Hands-on: hole 9 is a Song of Storms hole (user saw it); ocarina and song given; entry not confirmed in the log |
| 5 | Lose heart on death | Get a bottled fairy (give lists only swords and shields), die once with it | Deku Tree baba room | Bottle with a fairy in the inventory | Fairy revival must cost no heart (NOT TESTED) | Open |
| 6 | Batch 1 (06, 08) | Play a song at a gossip stone; fill a bottle (fairy) and use it in and out of combat; die with the fairy | Kokiri Forest / Deku Tree baba room | Mod with those settings, ocarina and song, empty bottle via give | Whether the fairy spirit appears; head shake in combat; automatic fairy revival | Open |

| 19 Nov | Gate guards: kill and barrier | Stand on each guard mark (gates 1, 2, 11, 12), kill the guard with a sword or Din Fire, look toward the wall | Maps and marks in docs; test exe with HELD empty at playtest\gatetest | Shots before and after of the purple barrier | Open |
