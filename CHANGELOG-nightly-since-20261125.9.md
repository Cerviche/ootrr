# OOTRR nightly: changes since 0.9.0-nightly.20261125.9

A hotfix for the Archipelago page, from the first real rooms. Covers the nightly branch from `b17a368` (the source of nightly 0.9.0-nightly.20261125.9).

## Archipelago

- **Rooms that send Double Defense, Nocturne of Shadow or Deku Sticks now build.** The build stopped with "is not a usable actor name (at most 25 characters…)" as soon as another world held one of these four items, which is nearly every room. The limit was stricter than the game: its own Great Fairy of Courage hands out Double Defense under a 29 character name through the same item get, and normal seeds give all four this way. Names the mod makes up itself keep the 25 character limit. *The build is checked; receiving these items in a room is not checked in game.*
- **"no such job" in the browser version of the app.** There, building from a `.apootrr` file started the build and then lost track of it. The browser version now follows the Archipelago build the way it follows a normal one, and offers the mod as a zip download.
- **The Archipelago page in the browser version** only shows what a browser can do. Step 1 explains how to put the OOTRR world file in `custom_worlds` by hand, and step 4 explains that playing in a room needs the OOTRR app on your computer, because Archipelago's OOTRR Client runs it.
