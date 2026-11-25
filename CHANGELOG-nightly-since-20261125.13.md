# OOTRR nightly: changes since 0.9.0-nightly.20261125.13

Two Archipelago fixes from the first rooms. Covers the nightly branch from `e9b0a35` (the source of nightly 0.9.0-nightly.20261125.13).

**Make your player file again and regenerate the room** to get both fixes. The client part of the first fix also works in rooms made before it.

## Archipelago

- **Getting a song no longer sends the check of that song's own spot.** Songs, Sheik, Saria's and Malon's gifts, Zelda's Letter, the ocarinas, Rauru and the boss rewards were seen through a flag the game sets whenever that item's get plays, wherever it came from. Receiving Song of Time from another world sent "Song from Ocarina of Time", and a gift spot holding another item never sent its own check. These 85 spots are no longer seen that way. Until they have a flag of their own, they hold only your own items, which the game gives as usual, so nothing another player needs can be lost there. The client also ignores those flags in rooms generated before this fix.
- **Keys can be in other players' worlds.** With Small Keys, Boss Keys, Ganon's Boss Key or Maps and Compasses set to Anywhere, they now go into the shared pool. A key from another world shows its item get, is counted for its own dungeon the way a key from a chest is, and names the dungeon. Hideout keys and key rings still stay in your own world. *The build and a server test are checked, receiving a key in game is not.*
