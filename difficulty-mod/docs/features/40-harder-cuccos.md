# 40. Harder cucco game (Talon, Lon Lon Ranch)

## Requested (user, 21 Nov)

"When you toss a cucco, it doesn't stay in-place for a while. Also there's 50% more cuccos and 50% as much time." First read as Anju's cucco game; the user corrected it: it is Talon's game at Lon Lon Ranch (child), where cuccos are scattered in the house and you find his 3 super cuccos before time runs out. More cuccos means more ordinary decoys, not more super cuccos.

## Setting

`harder_talon_cuccos`, number 155, "Harder cucco game at Lon Lon Ranch", off by default. Difficulty page, section Minigames (next to the Bombchu Bowling grass). Number 154 is taken by lightning and 152 and 153 may be in use elsewhere, so 155 is the next free one. Default settings string and seeds unchanged.

Tip: "In Talon's game there are more cuccos to dig through, they don't stay put after they land, and you get half the time. Anju's cuccos are not changed."

It does not touch Anju's cuccos, so it has no conflict with `anju_one_cucco` (that setting changes Anju's game in Kakariko, this one Talon's in Lon Lon Ranch). Both can be on.

## Where it lives (found in the game files)

Talon's game is `Minigame_FindOutCucco`, not `Minigame_CatchCucco` (Anju's).

- **Time**: `Event/EventFlow/Minigame_FindOutCucco.bfevfl.zs`, event `Event27`: `ChallengeSystemActor.EventTriggerMiniGameTimerSetting {IsCountDown=true, TimerLimit=1800}` (frames at 60 per second, so 30 s). The setting halves the number to 900.
- **Cuccos**: `Banc/HyruleField/HyruleField_Child_Static.bcett.byml.zs` holds Talon (`Npc_Hylia_Talon_Child`, named in the flow), 10 `Animal_Cucco_FindOutCucco` (`Animal_Cucco_01` to `_10`) and 3 `Animal_SuperCucco`. The flow resets each named cucco by its own name; the win is lifting the three super cuccos (`LiftUp_SuperCucco1` to `3`).
- **Not staying put**: the pack `Pack/Actor/Animal_Cucco_FindOutCucco.pack.zs` has its own wander file, `Game/BSA/ActionSeqContainer/Animal_Cucco_FindOutCucco_Wander`, with `MoveIntervalMin` 100, `MoveIntervalMax` 600 and `NoMoveTime` 60 frames: a cucco that has landed waits 2 to 10 seconds before it moves again.
- Talon is only at the ranch once the castle quest is past `Step03_02` (his placement has a presence flag), so test with `open_kakariko` open.

## What the setting does

`difficulty/talon.rs`, three files:

1. The flow: `TimerLimit` 1800 to 900 (15 seconds). Refuses a flow without exactly one such event.
2. The wander file: 100/600/60 become 5/40/5 frames, so a cucco is on the move again within about a second at most. Refuses a file that does not have the vanilla numbers, and the size must not change.
3. The map: 5 more `Animal_Cucco_FindOutCucco` placements (so 15 ordinary decoys, plus the same 3 super cuccos) copied from the first placement with new names (`Animal_Cucco_11` to `_15`), fresh hashes, positions inside the room (y 150.7) and no link to Talon. They are not named in the flow, so they are never reset or hidden by it: they are plain decoys that wander about the room, and they stay in the room after a game.

All edits are data (a flow number, three floats, five placements). Nothing needs a code patch; it works wherever OOTRR works.

## Tested

- `cargo test -p ootrr-core difficulty`, `settings` and `qol` pass with the game files. New tests: the limit reads 1800 and becomes 900; the wander file changes only its three numbers and is refused with other numbers; the map gains exactly five cuccos with unique hashes and nothing else changes; a second run is refused; the stage returns the three files and only the wander file changes in the pack.
- In the game (Longshot test build, windowed, fresh save, setting on, `open_kakariko` open so Talon is at the ranch, 100 rupees given): the mod loads and Hyrule Field (child) loads; the ranch house is full of cuccos (the debug overlay listed 18 `Animal_Cucco` objects nearby, which includes some outside the house, so 15 in the house was not counted exactly); Talon talks, takes 10 rupees and starts the game; the timer ran down 9.39 then 5.03 on two screenshots 2.5 s apart (a 30 s limit could not read 9.39 so soon after the start); the cuccos moved between screenshots; at 00:00 Talon said time's up, the dialogue closed and Link walked again (no freeze).

## Not tested

- **Winning**: finding and lifting all 3 super cuccos in 15 seconds, and the reward being given. Too hard for a script (the user said not to spend time on it); a step is in the capture playbook.
- The exact start value of the timer (15.00) was not caught on screen, nor the old rest time compared side by side with a vanilla build.
- Whether the five extra cuccos get in the way of the super cuccos' jump anchors.
- Talon's lines may mention the time limit; the text is not changed.

## How to test it in the game

Preset: `harder_talon_cuccos` on, `open_kakariko` open. Go to Lon Lon Ranch as a child with 10 rupees, talk to Talon in the house and start. Expect more cuccos than usual, a timer of 15 seconds, and cuccos that you pick up and throw start moving again almost at once. Find the three super cuccos (they jump about) and lift them for the reward.
