# OOTRR nightly: changes since 0.9.0-nightly.20261125.9

A hotfix for the Archipelago page, from the first real rooms, and fixes for the Ice Traps. Covers the nightly branch from `b17a368` (the source of nightly 0.9.0-nightly.20261125.9).

## Ice Traps

Three reports turned out to be Ice Traps: a Slingshot with no Deku Seeds, wrong progressive items at Gold Skulltulas and freestanding spots, and item upgrades from traps. *None of these fixes is checked in game.*

- **A trap no longer leaves anything in your inventory.** The item a trap shows was a working copy of the real one. A trap that looked like a Slingshot gave a second Fairy Slingshot with no bullet bag, so it showed 0 seeds and could hold none. One that looked like a bigger bag, a scale or gauntlets gave that upgrade's effect, and one that looked like an Ocarina gave an Ocarina. The trap was only meant to take it away again afterwards. It is now a plain item that is never stored, like a dungeon map.
- **A trap in a chest no longer hands out a real upgrade or ammo.** A trap that looked like an upgrade could carry a real item of the seed in its chest, and the chest then gave that item's next upgrade, or its ammo, in place of the trap. It only ever carries a rupee now.
- **Traps on the ground, in pots and at Gold Skulltulas freeze you when you pick them up.** They checked which item was picked a moment too early, so they did nothing, left their copy in your inventory, and froze you at the next thing you picked up.
- A real Slingshot or Bow bought from a shop shelf or a Deku Scrub still gets its starting bag at your next item, not at once. Not changed here.

## Archipelago

- **Rooms that send Double Defense, Nocturne of Shadow or Deku Sticks now build.** The build stopped with "is not a usable actor name (at most 25 characters…)" as soon as another world held one of these four items, which is nearly every room. The limit was stricter than the game: its own Great Fairy of Courage hands out Double Defense under a 29 character name through the same item get, and normal seeds give all four this way. Names the mod makes up itself keep the 25 character limit. *The build is checked; receiving these items in a room is not checked in game.*
- **"no such job" in the browser version of the app.** There, building from a `.apootrr` file started the build and then lost track of it. The browser version now follows the Archipelago build the way it follows a normal one, and offers the mod as a zip download.
- **The Archipelago page in the browser version** only shows what a browser can do. Step 1 explains how to put the OOTRR world file in `custom_worlds` by hand, and step 4 explains that playing in a room needs the OOTRR app on your computer, because Archipelago's OOTRR Client runs it.
