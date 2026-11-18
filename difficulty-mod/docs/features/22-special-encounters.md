# Special encounters: Dead Hand in two holes, Iron Knuckle in Hyrule Field

Setting `special_encounters` (number 130), checkbox, label "More enemies: special encounters". Code: `crates/core/src/difficulty/encounters.rs`. Built on `feature/dead-hand-holes` (worktree `wt\dead-hand-holes`). Separate from the `more_enemies` choice (119), which is unchanged.

## Request
- "add a dead-hand into the hole which is really dark and spooky", then after the picture catalogue of all holes: "9 and 23 should get a dead hand".
- "can we put a darknut in hyrule field? just in the otherwise pretty open area between kakariko, lonlon, and deku".
- Then: one setting for all of these hand placed encounters.

## Interpretation
- Hole 9 is the "Dark ReDead Hole" in east Hyrule Field (the ReDead stays). Hole 23 is the "Dodongo Fog Hole" at the foot of the Death Mountain trail (map `DeathMountainTrail`, Baby Dodongos and bomb flowers inside). One Dead Hand in each, on the floor a few units from where Link lands.
- This game has no Darknut. The equivalent is the Iron Knuckle. One Iron Knuckle in the open grass of Hyrule Field, roughly in the middle of the triangle Kakariko entrance, Lon Lon Ranch gate, Kokiri Forest entrance.
- Each placement is an entry in `PLACES` (a const list) so one can be dropped if its map will not load.

## Investigation
### What a vanilla Dead Hand needs
Two vanilla instances exist: Bottom of the Well (`BottomOfTheWell_Static`) and Shadow Temple (`ShadowTemple_Static`).
- Actor `Enemy_DeadHand` (the body) plus `Enemy_DeadHand_Hand` (the arms that grab). The body has a `Links` list with one `Reference` link to each hand (Well: 4 hands, Shadow Temple: 6).
- Body `Dynamic` in the Well: `DeadHand_HoldActorCancelSleepFrame` 1800, `DeadHand_IsManageHandNumByLife` true, `Drop__DropTable` "None". Shadow Temple has `DeadHand_RangeOfAppearArea` 4.0 instead of the manage flag. The actor's own defaults (read from its pack's blackboard table): appear range 6.0, cancel frames 900. The body also carries `Name: "DeadHand"`.
- Hands: the Well ring puts the hands at 5 units along each axis around the middle, each rotated to face the middle. Hand `Dynamic`: one has `DeadHandHand_IsEnableAppearDelay` false, one has `DeadHandHand_AppearOrderNum` 2, two have 3. All of them also carry `DefaultDungeonRoomIndex`, which is a dungeon field and is left out here.
- Both vanilla rooms put the body and hands in a `FarDeleteGroups` and a `SimultaneousGroups` entry (hash lists the engine loads and unloads together), and an `AiGroups` entry with the room logic: the room module, an area, an enemy counter linked to the body, a chest that appears when the counter reaches zero, a push wall and an `EventRequestActor`.
- The body's AI (`Pack/Actor/Enemy_DeadHand.pack.zs`) has its own states (hide, appear, land battle, escape). The room logic looks like it only provides the reward (the chest and the wall), not what makes him rise.

### What I reproduced and what is missing
- Reproduced: body with its Dynamic keys and Name, four hands (Well layout and Dynamic keys), the four `Reference` links, and one group entry in `FarDeleteGroups` and `SimultaneousGroups` holding the five hashes in ascending order, like vanilla.
- Not reproduced: the room logic (`AiGroups`), enemy counter, reward chest and push wall. A hole has no such logic, so the fight gives no reward and nothing closes behind you. That is intentional.
- Changed from vanilla: the hand ring radius is 3.0, not 5.0. Both holes are narrow: the hole 9 cave is a corridor about 7 wide (its inner area box is 7.3 by 15.4), so a radius 5 ring would put hands in the walls. The ring is turned to follow the cave's long axis. This is a guess: if hands stand in rock, change `RING`.
- Not verified: whether the body wakes without the room's enemy counter and area. The Shadow Temple body uses its own appear range, which is why it is expected to work, but nobody has seen it outside a room.

### Positions
All from the real map files; there is no collision geometry reader in the repo.
- Hole 9 (`HyruleField_Child` and `_Adult`, identical): hole base `C_01` at (163.27, 150.74, 27.38); landing anchor (163.27, 136.12, 27.38); restart anchor (the survey's inside point) (161.81, 135.84, 25.93). Floor height 135.84 (the ReDeads stand on it). The cave is a corridor along a 45 degree line from the landing end to the chest (152.2, 136.0, 16.7); the ReDeads are at (155.0, 15.7) and (148.7, 17.7). Dead Hand at (156.9, 135.84, 21.2): 6.8 from the restart anchor, 5.9 from the nearest ReDead, 6.5 from the chest.
- Hole 23 (`DeathMountainTrail_Child` and `_Adult`, identical): base `I_03` at (-1.65, 71.17, -26.12); restart anchor (-2.19, 58.69, -28.15). Floor 58.17 (Baby Dodongos and rupees stand on it). The cave inner box is centred at (-3.5, -33.0), 11 by 16, turned 15 degrees. Dead Hand at (-5.2, 58.17, -34.2): 6.7 from the restart anchor, 5.4 from the nearest Baby Dodongo, a few units from the bomb flowers and rupees.
- Iron Knuckle: the corners are `HyruleField_FrontOfKakarikoVillage` (210.8, 176.9, 10.0), the Lon Lon gate `Land_LonLon_Gate_B_01` (-61.8, 150.6, 184.2) and `HyruleField_FrontOfLostWoodBridge` (200.1, 175.7, 309.1); the middle is about (117, 170). Ground standing actors near there are three rocks 4 to 6 units from the chosen spot (`Land_HrlField_RockGround_A_01` at y 157.63 and 157.01, `Land_HrlField_EponaObstacles_Rock_A_01` at y 158.04). The spot is (134.0, 157.7, 168.5), 17 east of the exact middle so rocks are beside it. It is 46 from the nearest hole, 36 from the river bank, and far from every entrance trigger and the ranch walls. The ground height comes from the rocks and may be a few tenths off. Both ages have the same rocks at the same heights.

### Which Iron Knuckle
Vanilla: `Enemy_IronKnuckle_Silver` (two in the Spirit Temple), `Enemy_IronKnuckle_White` and `_Black` (Ganon's Castle, linked to each other), `Enemy_MiddleBoss_IronKnuckle_Nabooru` (room logic and a cutscene).
- Chosen: `Enemy_IronKnuckle_Silver`. Its Spirit Temple instances carry `BreakTarget` links to the chair and pillars that crumble when it stands, plus `Drop__DropTable` "None". Its actor has one blackboard boolean (`HasArmor`, true) and its AI reads perception, so it does not depend on room logic to fight.
- Reproduced: the actor with `Drop__DropTable` "None", no links, no group. The chair and pillars are left out: they would be a second new kind in the map and are only scenery.
- Sit or stand: its idle action is `Sit`, so it is expected to sit (with no chair under it) until it is hit or notices Link, then stand and fight. Inferred from its AI graph, not seen. If it looks wrong without a chair, the chair (`Obj_IronKnuckle_Chair_A_02`, linked as `BreakTarget`) can be added.

## Implementation
Every addition goes in the `_Static` file of its map (4 files: `HyruleField_Child`, `HyruleField_Adult`, `DeathMountainTrail_Child`, `DeathMountainTrail_Adult`). Hyrule Field's two files carry the hole 9 Dead Hand and the Iron Knuckle in one rewrite. No `_Dynamic` file is touched.

Why `_Static`, not `_Dynamic`, which was the preferred option: no actor in a Hyrule Field `_Dynamic` file has a `Links` list, and that file has no `AiGroups` or `FarDeleteGroups` keys, so a Dead Hand cannot hold its hands there. Also the ReDead in hole 9 (`Enemy_ReDead_AlwaysRespawn`), the Baby Dodongos in hole 23 and the field's Octoroks, Tektites and Peahat Larvae are all in `_Static`, so putting the new enemies there mirrors them. The Iron Knuckle mirrors the Octorok and Tektite entries: a plain actor, no links, in no group.

Known risk: Dead Hand, Dead Hand Hand and Iron Knuckle Silver are all kinds that these four maps do not contain, appended to a `_Static` file. That is the same shape as the Dodongo's Cavern edit that went to a black screen, and it cannot be avoided for the Dead Hand. That black screen was not isolated, so this is a test, not a safe bet.

The new actors use the repo's usual shape (`Gyaml`, `Hash`, `Phive.Placement.ID`, `SRTHash`, `Rotate`, `Translate`) with fresh hashes that are in neither file of the map (`ambush::fresh`). The step reads the files through the same chain as the other features, so it combines with More enemies, Enemies block the way and the rest.

## How it was tested
- Unit tests in `encounters.rs`: a Dead Hand gets four hands, matching links and one group per list, unique hashes, byte round trip; hand positions and rotations match the vanilla ring; an Iron Knuckle adds one plain actor and touches no group; two places in one map share no hash; a map without the group lists is refused.
- Game file test (`OOTRR_ROMFS`): each placement alone and all three together on the real maps. Each file gains exactly the expected actors, the old actors are untouched, `AiGroups` is unchanged, all hashes are unique, the Dead Hand sits more than 6.5 from the restart anchor and level with it, and nothing new is within 4 of an existing enemy or chest.
- `difficulty` tests (113) and `settings` tests (107) pass. The everything-on game test passes with the new files, the build notes count is 21, and the settings string with the setting off is unchanged.
- A real build in `playtest\encounters`: a mod generated and built with this setting on. The four map files hold the new actors (6 more in each Hyrule Field map: Dead Hand, 4 hands, knight; 5 more in each Death Mountain Trail map).
- The game was not launched.

## In the game
Never played. Turn this on alone (preset and steps in `<playtest folder>\encounters\TESTS.md`).
- Hole 9: `teleport HyruleField_Child`, then `warp 161.81 136.84 25.93`.
- Hole 23: `teleport DeathMountainTrail_Child`, then `warp -2.19 59.69 -28.15`.
- Iron Knuckle: `teleport HyruleField_Child`, then `warp 128 159.5 163`.
- Look for: the map loads; Dead Hand's hands appear on the floor and he rises when Link is grabbed or comes close; he can be hurt and killed; hands are in the floor and not in a wall; the ReDeads and Baby Dodongos are still there; the room's exit still works. The knight sits beside the grey rocks, wakes when hit or approached, can be killed and is not in the ground.
- Repeat as adult (`age adult`, `teleport HyruleField_Adult` and so on).

## Knight wake on approach (v2)
Seen in the game (18 Nov): the knight spawned and sat still. You asked for it to wake when Link comes near.

### What holds the knight seated in vanilla
All Iron Knuckle variants carry the same graphs. `AI/Enemy_IronKnuckle.action.Sit.module.ainb` plays the `Sit` animation and runs a `SelectorStackNode` of four wake conditions, each ending the sit with Success: (1) a sequence that needs a perceived target, a 3 frame count and the nearest `BreakTarget` memory (the chair) to be gone; (2) a `Reference` link life check; (3) damage of type 9; (4) `QueryPhysicsContactPlayer` (bumping into it). Every vanilla instance (Spirit Temple, Ganon's Castle, the stage maps) is linked to a chair by `BreakTarget`; ours has no chair. No placed-instance key, no variant and no blackboard value sets "awake": the Silver, White and Black packs have identical graphs, and the placed instances carry only `Drop__DropTable` and `DefaultDungeonRoomIndex`. So route 1 (a parameter) does not exist. Read from the graph, not seen.

### What changed (route 2)
Only `Pack/Actor/Enemy_IronKnuckle_Silver.pack.zs` changes, two files in it (`crates/core/src/difficulty/wake.rs`):
- The knight's blackboard table gets a bool `OotrrKnightWake`, default false, `IsInstanceParam` true. This is how a placed actor's `Dynamic` key reaches a blackboard bool (the Dead Hand's `DeadHand_IsManageHandNumByLife` works the same way).
- The Sit graph gets a fifth child, placed first in the stack: a switch on `QueryBlackboardGetBoolNoError(OotrrKnightWake)`; when true, a switch on `QueryUserPerceptionIsInsideSphereByInput` with radius 9 centred on the `Spine_2` bone (the bone query, `QueryMathMtx34fToVec3fMat33f` and sphere query are copies of the sphere check already in the same graph, which uses radius 0.8); inside gives `ExecuteGenericSetResultSuccess`, otherwise `ExecuteGenericSetResultNone`. Same shape as the contact child (switch to Success or None). Built with the same builder and wiring helpers as feature 17 (`backup.rs`).
- Our placed knight carries `Dynamic` `OotrrKnightWake` true.
Scope: the Spirit Temple Silver knights share the pack and so run the new branch, but their key is false, so it returns None and they behave as before. The White, Black and Nabooru packs are not touched. Nothing else in the game changes.

### Evidence and what is unverified
- Test with the game files: the Sit graph has exactly one new child, first in the stack, the old four unchanged after it; the flag query reads the new key, the sphere radius is 9 (the old sphere stays 0.8); only the Sit graph and the knight blackboard differ in the pack; layout audit clean; the written graph reads back byte for byte.
- Not seen in the game. Open: the stack node may not run a child past a busy earlier child (ours is first to avoid this); the sphere query's "user perception" may only report targets that the knight's senses report; the empty territory check module above the stack might hold a field knight; the 9 unit radius assumes the same units as the map positions.

### How to test
`<playtest folder>\encounters2\TESTS.md`: knight still seated at 12 units, stands and attacks when Link walks to 5 units; Spirit Temple knights still sit until hit.

## Knight awake from the start (feature/knight-roam, 18 Nov, not played)
You asked: "ideally he'd already be awake and just roaming".

### Awake
`wake.rs` has a const `MODE` (`Wake::Awake` or `Wake::SeatedUntilNear`). In `Awake` the new first child of the Sit stack is the flag switch only: our knight (`OotrrKnightWake` true) ends the Sit with Success as soon as he is active, no sphere. Spirit Temple knights (flag false) are unchanged. `SeatedUntilNear` is the old v2 graph.

### What he does after release (read from the graphs, not seen)
Brain (`Enemy_IronKnuckle.brain.root`) handles exactly five verbs: initial wait (Sit), a discovered/stand up verb, land battle, a hidden/event verb and an inaccessible verb. Sit finishing with Success sends the brain to the stand up verb (StandUp plays `StandUp`, then `Wait`, then asks the brain to update); that verb searches for a target every frame and goes to land battle when one is found. With no target he stays in the stand up/wait state: standing still. Unknown: whether the stand up container replays the stand up animation in a loop while nothing is perceived. How far his senses reach is also unknown (the 9 unit gate hid it).

### Roaming: not built
- The brain's idle selector `EnemyCommon.SelectIdleBrainVerb.module` has a patrol flag (`IsEnablePatrol`, false in the knight's brain) and a return-to-home path using a map rail (`QueryActorGetNearestPointOnRail`). But the patrol module `EnemyCommon.Patrol.module` is an empty graph, and the selector sets brain verbs 18/19/62 that the knight's brain updater has no case for (it falls to an "unsupported brain verb" failure node). The knight also has no territory module in the brain and no patrol or wander action container in its scheduler (only Wait, Discovered, LandBattle, EventAIAndHidden).
- Other enemies (Stalfos, Lizalfos, Wolfos) have only return-to-home modules, no wander. Wander graphs exist for animals and NPCs (`WanderDefault`, Guay, Kokko, Horse), not for sword enemies.
- A faithful roam would need: a new brain verb handler in the knight's brain, a new scheduler entry and `ActionSeqContainer` data file with a new action verb in the pack, an action sequence graph that calls his Walk action with a target position (the Walk action takes its target from the action verb arguments, not a free position), and a rail actor linked to him in the map. That is new BSA data with no vanilla precedent for the knight, so it was not attempted.

How to test: `<playtest folder>\knight-roam\TESTS.md`.

## Notes
- If a map shows a black screen, drop the place from `PLACES` and rebuild. The three are independent. Both Hyrule Field maps carry two places, so test them separately by editing the list.
- Open questions: does the Dead Hand need the room's enemy counter to wake? Should the knight have a chair? Is a radius 3 hand ring right for the narrow caves?

## Seen in the game (first playtest, shadow game, windowed)
Preset: `special_encounters`, 20 hearts, enemy_health 50, Din's Fire with `["magic_meter","dins_fire"]` (the other order is refused by the generator). Din's Fire is on the X slot; the bridge button is `ZR`. Magic is empty after about 4 casts and `give` cannot refill it (status shows no magic). Shots: `playtest\encounters\shots\`.

| Place | Age | LOAD | PRESENT | REACTS | KILL | EXIT |
|---|---|---|---|---|---|---|
| A Iron Knuckle, Hyrule Field | child | PASS (map loads, bright picture) | PASS (armour seen at the spot, `A-child-wait5.png`) | Stays seated and still for 15 s with Link 2 units away. Din's Fire wakes it (music BGM_MiddleBoss); it hits for about 3 hearts a swing and killed Link (20 hearts) in 6 casts | Not killed in 6 casts | - |
| A | adult | PASS | picture taken | Same: still for 15 s, woke on the first cast, attacked; Link died | Not completed (2 casts) | - |
| B Dead Hand, hole 9 | child | PASS for the map; the warp into the room failed, Link ended in Kokiri Forest | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |
| B | adult | Warp into the room landed at (161.9, 135.7, 26.5) with an event for about 5 s, then Link was in Kokiri Forest (cause unknown: Dead Hand grab, or the room reload) | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |
| C Dead Hand, hole 23 | child | PASS | PASS (`C-child-wait5.png`) | Rose and attacked within 15 s (BGM_MiddleBoss, life 72 to 56) without any room logic | PASS: smoke, music stopped, after 3 to 4 Din's Fire casts (`C-child-cast4b.png`) | NOT TESTED |
| C | adult | PASS | PASS | Rose and attacked | Not killed: magic ran out after 5 casts and Link died | NOT TESTED |

Notes: no black screen in Hyrule Field or Death Mountain Trail, either age. The knight does nothing on proximity alone (a code change for that is being prepared). Hole 9 needs a retest with a better method (the earlier survey reached it by warp without the mod).

## Seen in the game (v2 build, wake on approach, 18 Nov)
Same shadow game, windowed, 20 hearts, `enemy_health` default. Shots `playtest\encounters\shots2-*.png`, log `playtest\encounters
esult.txt`, hole 9 samples `playtest\encounters2\hole9-log-a1.json`. Approach points are on the south-west side of the knight (the east side is off the map: Link fell and respawned in Kokiri Forest).

| Check | Age | Result |
|---|---|---|
| Knight stays seated at 14 units, 10 s | child, adult | PASS: no life change, field music only, no movement (`v2-c-14u-9s.png`) |
| Knight wakes at 5 units, no hit, Link still | child | PASS: battle music (BGM_MiddleBoss) already on within 1.6 s of the warp (the first poll), first hit at 6.1 s (life 80 to 68, Link knocked back 2.5), further hits at about 12 s and 18 s (`v2-c-5u-6s.png`: axe swing, Link knocked down) |
| same | adult | PASS: same signs, music at once, hit at 6.0 s (20 to 8 quarter hearts), Link died later and was recovered with Try Again |
| Spirit Temple silver knight, Link 5 units away for 15 s | adult | PASS (control): still seated, life and music unchanged (`v2-spirit-5u-8s.png`). Warp point from TESTS.md landed safely |
| Hole 9 Dead Hand | child | FAIL (case c): warp to (161.81, 136.84, 25.93) put Link in the room at once (y settled at 135.74, room music, dark picture, nothing seen). An event ran for 4.2 s, then the map changed to KokiriForest_Child (-126.5, 114.7, -38.4). Life unchanged (32), never dead, y never dropped. So a map change event fires at (161.81, 135.74, 25.93) about 4 s after entering. Second attempt not made. PRESENT and REACTS not seen |
| Hole 23 exit | child | PASS: warp to the exit trigger area (-1.71, 68.5, -25.99) never arrived (Link was at the entrance, -1.65, 120.17, -26.12, already); after warp to the `HidingHole_FieldExitGo` anchor (-1.65, 59.05, -26.12) Link stood at (-1.19, 120.17, -26.52): carried back up. The first warp into the room had already been followed by the carry-up, so which trigger did it is not isolated. Dead Hand music was on in the room |

Open: hole 9 event. Compare with the unmodded game (survey: stayed in the room, music BGM_Common_Dungeon) so the mod's change to hole 9 (Dead Hand placement or its room logic) triggers a map change; fix in code, then retest.



## Session fixes (18 Nov, feature/session-fixes, not played)
User, after entering hole 9 properly: the Dead Hand "could be moved back a bit tho, right now it's attacking the moment you land already".
- Hole 9 (Hyrule Field): Link lands at the exit anchor (163.27, 136.12, 27.38). The cave runs 15 units from there to the chest at (152.2, 16.7), with the ReDeads near its far end, so the very end is taken. Dead Hand now at (155.72, 135.84, 20.09): 10.5 from the landing spot, 4.5 from the nearest ReDead, 4.9 from the chest. Nearest hand 7.5 from the landing spot (was about 5.9).
- Hole 23 (Death Mountain Trail): landing at (-1.65, 59.05, -26.12). Dead Hand now at (-5.0, 58.17, -38.5), the far end of the room among the Baby Dodongos: 12.8 from the landing spot (was 8.8), 4.2 or more from every Dodongo. Nearest hand about 9.8 from the landing spot.
- The ring and floor heights are unchanged. Both child and adult maps use the same numbers. The game test now also requires every hand to be at least 6 units from the landing anchor.
- In the game: see `playtest\session-fixes\TESTS.md`.
