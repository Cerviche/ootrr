# Zora's Domain ambush

Setting `zora_ambush` (number 127), checkbox. Code: `crates/core/src/difficulty/ambush.rs`. Built by a sub-agent on `feature/zora-ambush` (`f098307`).

## Request
"in zora's domain, there's a hole that has absolutely nothing in it. let's make that more fun. put a wall skulltulla on each of the far walls, and when you go into the wall area, spawn a ceiling skultylla behind you, so it catches you on the way out. heheh. fun"

## Interpretation
Find the empty hidden hole in Zora's Domain, put a wall-crawling Skulltula on each far wall, and make a hanging Skulltula appear near the way out once you have walked in.

## Investigation
- Zora's Domain has exactly one hidden hole. Its cover is near (17, 100, -18) and the cave is about 40 m below. It holds only six blue rupees. Your later map link points right at it.
- The cave is the piece `Land_HidingHole_Base_D_01`: a central chamber with three side caves. The same piece is used in the Hyrule Castle hole, where the side caves are sealed by bombable walls; here they are open and empty. "Wall area" was read as those side caves.
- **Appear on entry already exists in the game:** the first room of the Forest Temple has a four-node logic graph where an area sensor latches and creates a Skulltula. It can be reused as it is.
- Wall and ceiling heights came from the cave's visible model, not its collision shape.

## Implementation
In all three Zora's Domain maps (child, adult, adult alternative):
- **3 wall Skulltulas**, one on the back wall of each side cave, about 3 m up. Two are exact transplants of the vanilla wall Skulltulas in the Hyrule Castle hole. The third cave has no vanilla example; its spot was picked from the model.
- **3 hanging Skulltulas**, one in each side cave's doorway, 3.8 m above the floor.
- **3 trigger areas** covering the inner part of each side cave. Each is tied to its Skulltula through a new group that points at the Forest Temple graph.

## How it was tested
- 22 difficulty tests; one checks the code reproduces the vanilla castle coordinates exactly, one checks all three real maps and that Gold Skulltulas are untouched.
- Builds with the toggle on and off differ only in the three map files and the size table.
- Positions are plotted in `<builds>\rc-4\entity-maps\ZoraDomain_Child_zoom.png`.

## In the game
Never played. To test:
- Turn this on alone and drop into the hole in Zora's Domain.
- Pass: a wall Skulltula on the back wall of each side cave; walking to the back of a side cave makes a Skulltula drop in its doorway.
- Report per cave (A is toward +x, B toward -x and -z, C toward +z): Skulltula floating off the wall or inside it, ceiling one too high or low, or no ambush at all.
- If you meant a different spot, say where.

## Notes
- The third wall Skulltula is the most likely to look wrong.
- Whether the Forest Temple graph runs in another map is untested.
- The object map site shows this cave's real collision; with it the positions could be corrected precisely.
- Reusing a vanilla logic graph by pointing a new group at it is a cheap way to add scripted behaviour to a map without editing any graph.
