# Enemies block the way

Setting `enemy_gates` (number 129), checkbox "Enemies block the way" on the Difficulty page, default off. Built on `feature/enemy-gates`. Experimental, never loaded in the game. Code: `crates/core/src/difficulty/gates.rs`.

## Request
"enemies block progression. not all enemies, but many, should block the way-forward. this could be by adding a new lock to the existing door, or by putting up some temporary wall. the idea here is that if enemies are too hard to defeat, you should retreat and come back when you're stronger. it should never make a game impossible to beat, or lock you without the option to return. obv in areas like hyrule field this doesn't make sense, and some rooms already have this (like the stalfos at the end of the lost woods) but there's some obvious places where this would make sense, like the deku babas between the kokiri forrest and the deku tree"

Then, simplifying it: "do we really need to perma-save the state of these enemies-block-passage? can't it just be that if a particular enemy is alive, the wall is in-place, and when the enemies are defeated, it goes away? seems pretty stateless to me."

## What it does
No saved state and no new flag. The wall exists exactly while its guarding enemies are alive. If the area loads again and the enemies are back, the wall is back. The earlier blocker (needing a persistent flag) is gone.

## Gates (an explicit list, `GATES` in gates.rs)
1. The two Deku Babas on the path from Kokiri Forest to the Deku Tree (child map only). Babas at (39.12, 113.19, -18.58) and (37.74, 113.26, -19.59). The path there runs north to south, so one invisible wall (`FldObj_AirWall_Player_GenerateNavMesh`, the kind Mido's blocker uses) stands across it at z = -19.08, 16 wide, with the two babas on it. The other three babas of that map are untouched.

I did not add more gates. None of the other candidates could be checked as safe for the same reasons (see below).

## Mechanism
- **Death signal:** a Logic graph. The nodes are `ActorLogicSimpleActor` ones, the same kind the game uses for an enemy whose `Dead` output is read (the wolfos in `Gimmick_DefeatEnemy_Open_03`) and for air walls (`FldObj_AirWall_Player_Unridable` in the Death Mountain Trail graph). For each guard the build adds two nodes: a guard node, and a wall node whose `Logic_Delete` input is wired to the guard node's `Dead` output. With two guards there are two walls stacked at the same place, so the passage opens only when both are dead, with no AND node needed.
- **Host graph:** no vanilla graph has the shape "N enemies dead, then delete one plain actor" without a save flag or a typed enemy node. The closest are `Gimmick_AllDay_Skulltula_115c` (typed to the Gold Skulltula) and the counter rooms, which all set a persistent flag. So the build adds the four nodes to `Gimmick_AllDay_Skulltula_115c`, the smallest graph, exactly the way `minibosses.rs` adds a node, and binds them through a new `AiGroups` entry in the Kokiri Forest child static map, the way `ambush.rs` does. Other maps that use the same graph get four extra nodes bound to nothing.
- **Dynamic to static:** the five babas live in `KokiriForest_Child_Dynamic`, which has no `AiGroups`, and across 40 maps no group refers to a dynamic actor. Graphs only ever bind static actors. So the two guard babas are moved, unchanged, with their `Presence` flag, from the dynamic map into the static one. None is added or lost (the game test counts them).
- **Same flag on the wall:** each wall copies the baba's `Presence` (`KokiriForest_DeleteDekubaba`, negated). Once the Stone of Forest is yours or the Kokiri 0150 event has played, the babas are not placed and neither is the wall, whatever the graph says about a guard that was never created.
- **Wall choice:** invisible, because the vanilla visible barriers (webs, brambles, Mido's gate) burn, break or need an event, and a barrier that can be bypassed is not a gate. An invisible wall is confusing, so the build report says where it is.

## Never trapping the player
- The babas stand in the wall (within 0.5 of its plane), so a sword swing, the boomerang, a slingshot shot or a bomb reaches them from both sides. Coming out of the Deku Tree with the babas alive again you meet the wall from behind and can kill the babas from there.
- Warp songs, Farore's Wind and "Return with Navi" do not land in this corridor. Entrance shuffle can put you out of the tree on the far side, so the gate is skipped when entrances are shuffled.
- If you have no weapon at all, Deku Sticks and the tree's own contents still give one.

## Never making a seed unbeatable
`KF Outside Deku Tree` needs `is_adult or open_forest == 'open' or 'Showed Mido Sword & Shield'`, and the Mido event needs the Kokiri Sword. With a closed or closed-Deku forest a child already needs the sword (and the logic already counts it as the way to kill these babas: "Deku Baba Sticks" needs it or the Boomerang), so the gate adds nothing. With `open_forest` open a child needs nothing, so the gate is skipped there. Adult Kokiri Forest is untouched. The build report says when the setting is on but nothing is gated.

## Earlier findings that still matter
- Only Logic graphs react to deaths; map actors have no link that does.
- `FldObj_AirWall_*` actors take a `Presence` block. Mido's wall is `FldObj_AirWall_Player_GenerateNavMesh` at (25.17, 110.84, 3.79), scale (1.11, 1.5, 0.12), next to `Land_Kokiri_FieldProp_StoneGate_A_01`.
- The Deku Tree entrance is at about (90, 115.5, -28). Coming out of the tree puts you east of the wall.
- Counter rooms (Spirit 04_03, Ice Cavern 01 and 04, Forest Temple 06 and 18, Jabu, Shadow) set persistent flags, which is why none can be reused.

## How it was tested
- `cargo test -p ootrr-core difficulty` (96 pass, with `OOTRR_ROMFS` set) and `cargo test -p ootrr-core settings` (107 pass), `cargo build -p ootrr`.
- Unit tests: guards found by position and moved with their flag, missing or flagless guard refused, a wall per guard with the guard's flag, sorted unique hashes, round trip of the map, skipping under open forest or shuffled entrances, setting read and settings string unchanged.
- Game-file test (gated on `OOTRR_ROMFS`): guard count preserved across the two maps, walls added, old actors untouched, groups sorted, graph audits clean, each wall's delete input wired to its guard's `Dead` output, same output on every build.
- A real build with the setting on completed; the mod holds both Kokiri Forest child maps and the edited graph.

## In the game: what to check first
1. Kokiri Forest (child, closed forest) loads at all, with the Deku Babas where they were and no crash. A crash on loading the forest points at the moved babas or the new graph nodes.
2. Walk up from Mido's gate: is the invisible wall at the second baba pair and does it stop you? Is anything bypassed around its ends?
3. Kill both babas: does the wall go? Kill only one: it should stay.
4. Go through the tree and come out: you are east of the wall, the babas are back, so is the wall; hit a baba from this side.
5. With the Stone of Forest, the wall must not be there.
If the engine refuses a simple actor node bound to a baba, the next step is a vanilla typed node or a counter room.


## Redesign (branch `feature/gates-v2`, never played)
User, after the hands-on session: "there was no wall. don't make it invisible. use the burnable spiderweb wall but give it the purple glow that invincible enemies have." Earlier: "the wall shouldn't be invisible. make it visible. make it mido, or maybe the spiderweb wall but with the purple glowing effect."

### What was built
- Each guard now has, besides the air wall, a visible web: `Obj_InsideTheDekuTree_SpiderWeb_A_02` (the solid, elastic wall web used in the Deku Tree, Hyrule Field and the Shadow Temple) plus its cosmetic partner `Obj_InsideTheDekuTree_InteractiveSpiderWebParts_A_02` (instance name `SpiderwebPart`), joined by a `Reference` link from the web to the part, as in the game's own maps. Two webs stand on the wall line at x 36.4 and 40.4, one per guard, at scale 1 (the game places these with no scale).
- The air wall stays as the real blocker and is thicker (z scale 0.12, like Mido's blocker, instead of 0.03). The old wall was thin and Link crossed it in the hands-on session.
- Each web, part and wall is deleted by the Dead output of its guard through the same graph nodes (`ActorLogicSimpleActor`, `Logic_Delete`), so the webs of a guard vanish when that guard dies and the air wall vanishes when both are dead. Webs and parts copy the guard's `Presence` flag.

### Fire-proof: not done
The web dies by fire through its chemical parameters (`RequestDeathIfBurned` and `RequestDeathIfExploded` are true in `Obj_SpiderWeb`) and through its AI graph `Spiderweb_Wall` (burning leads to a death request; a lit Deku Stick sends an ignite message). Changing that needs a private copy of the actor (a new actor class with its own pack, rows in the actor info tables, the tag table and the resource size table, and an edited AI graph), as the cow clone feature does. That is data only but large and cannot be checked here, so it was not built. Instead the air wall holds when the web burns: fire removes the picture, never the blocker. The closest alternative if the user wants an unburnable visible barrier: a cosmetic web prop (`Land_Deku_SpiderWebProp_*`, no chemical parameters, no collision) in front of the air wall; its size is unknown.

### Purple glow: not done
The tint is not a model or actor parameter. It is applied by the native node `ExecuteSetEnemyDamageColorAndLight` in `AI/EnemyCommon.common.reactiondamagecolor.module.ainb` (the node's only data is its name; the colour lives in game code) and runs from the enemy graphs. No map actor key, model info or material parameter that a non-enemy object can carry was found. A glow would need game code or a rebuilt model, so the webs have none.

### Why the user saw no wall
Not the open-forest setting: `open_forest` defaults to "closed", so the gate is on by default, and the preset in the hands-on session set `enemy_gates` with entrances off. The map file of that session's mod could not be inspected afterwards (the mod folder was replaced), so the cause is not proven. Candidates: the wall was only 0.3 thick (now 1.2), a guard counted as dead (Dead true) before it was spawned, or the mod lacked the map. Nothing else in the difficulty code edits the Kokiri Forest map files, so a later merge overwriting it is ruled out. The test must print the mod's map and graph first (see `playtest/gates-v2/TESTS.md`).

### Unverified
Everything in the game: that the map loads, the web size and facing, that the air wall blocks, the deletion by the guards, the return of the webs.


## Session fixes (18 Nov, feature/session-fixes, not played)
User, with the gate v2 build: the web is visible and blocks, a bomb destroys it, and after that, with both guards alive, he could walk through.
- Why the wall failed: the invisible wall was `FldObj_AirWall_Player_GenerateNavMesh`, whose box is 4 by 4 by 4 before scaling (half extents 2). At scale (1.6, 1.5, 0.12) it was 6.4 wide, 6 high and 0.5 thick, not the 16 wide the earlier text said, so it was narrower than the two webs together (34.4 to 42.4). The game places that class in `_Dynamic` maps (Mido's blocker); our wall is in the static map, where the game uses `FldObj_AirWall_Common_Cube` (six in this map). The earlier script test that "stopped" Link at z -19.9 was within a metre of a guard Baba and may have hit the Baba.
- Change: the wall is now `FldObj_AirWall_Common_Cube` (a 1 unit cube standing on its origin), at (38.43, 112.2, -19.08), scale (16, 7, 1.2): x 30.4 to 46.4, y 112.2 to 119.2, z -19.68 to -18.48. Both guards and both webs are inside it. Same deletion wiring (both guards dead).
- Webs: still the Deku Tree wall web and its part. A bomb or fire can still destroy the web, but the wall stays, so the way stays shut until both guards are dead. A bombed web leaves an invisible wall.
- Not built: a private bomb-proof web. It needs a new actor class (pack, actor info rows, tag table, size table, edited AI and life data) and a wrong entry would crash Kokiri Forest on load with no way to check here. The air wall does the blocking. The purple glow is still not done.
- Test: `the_wall_covers_the_webs_and_the_guards_and_uses_a_class_the_static_map_already_holds`.
- In the game: see `playtest\session-fixes\TESTS.md`.
