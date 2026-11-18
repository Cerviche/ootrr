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
