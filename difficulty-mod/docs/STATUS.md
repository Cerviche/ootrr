# Difficulty features: status board

Last updated: 18 Nov, late night. rc 10 is cut and waiting for the user's review; unattended work on the backlog (docs/TODO.md) continues, Sonnet agents only.

Goal set by the user: every requested feature is either **Done** (seen working in the game, polished) or **Blocked** (investigated, with a written reason and the question for the user). Nothing stays at "built, never played".

## Right now

| What | State |
|---|---|
| **Has the game** | Nobody right now. Scripted tests run one at a time as fixes land |
| **Next for the game** | Scripted: which of regen / backup / fairy crashes on a kill (spawn an enemy with the tool, kill it with Din's Fire, one setting at a time); the spawn combination probes; Like Likes load test |
| **Then** | Merge what passes, cut rc 11. Anything needing the user's hands goes to capture-playbook.md |
| **Running without the game** | Gate fix round 2 (gates 1, 2, 11, 12, Lost Woods, same-side rule, red guards); surprises round 2 (1 in 50, bug grass, guard-house pots, Dead Hand's position) |
| **Waiting on the user** | Nothing blocks the work. Open questions are in the last section |

## Features

States: **Done** = seen working in the game. **Works, finishing** = seen working, polish or follow-up checks under way. **Built, untested** = never seen in the game. **Half tested** = part seen. **Broken** = seen failing, being fixed. **Not possible** = judged not doable with data edits.

| # | Feature | State | What was seen in the game | Next step |
|---|---|---|---|---|
| 01 | Enemy health | Done | Scaling works (by hand, rc 1) | Recheck one boss |
| 02 | Enemy damage | Done | 3x: a bite took 6 quarter hearts instead of 2 (script, rc 5) | Grabs (ReDead, Like Like) not covered; parked |
| 03 | Enemy speed | Built, untested | Nothing (batch 1 aborted before this) | Batch 1 |
| 04 | No heart drops | Done (small sample) | User (18 Nov): about 20 grass cut in the Lost Woods, only rupees and bugs dropped | Hearts the map places on the ground are left alone (user: fine unless they respawn; to check from the files) |
| 05 | No ammo drops | Done (small sample) | Same run as 04: no ammo dropped | Pots and enemy kills not sampled yet. With Shopsanity on, the note now warns that a special deal can replace an ammo shelf (review finding 3) |
| 06 | No gossip fairies | Done | User (18 Nov): none of the six songs produced a fairy at a gossip stone | None |
| 07 | Lose heart on death | Done, with one limit | 6 → 5 → 4 → 3 hearts over three deaths; a 4th death stays at 3; no side effects. The final build was replayed: 6 → 5 → 4 (script, 18 Nov). Confirmed by hand too (user died without a fairy: back with 5 hearts). Merged into `playtest/difficulty-all` | Limit: the loss does not survive quitting and continuing (accepted as "simplest for now"). Fairy revival costs no heart: confirmed by the user (revived at 6 hearts max) |
| 08 | No bottles in combat | Done | Confirmed by the user at the controls (18 Nov): a bottled fairy could NOT be used while the Iron Knuckle was aggro and COULD once far enough away; the refusal animation is right; a bottled fairy still revived Link automatically in the fight (6 hearts kept). Found and fixed on the way: the block ignored miniboss music (commit 630870b) | Not seen against a full boss; low risk |
| 09 | Enemies regen health | Built, untested | No crash in the scripted spawn-and-kill test (18 Nov). Healing never seen: enemy health is not readable yet | Enemy health in the debug panel (in progress), then check healing |
eview-fixes, not yet seen in the game) |
| 10 | More enemies (slider) | Half tested | Maps load (rc 4) | Batch 3: positions, the "red enemies" look |
| 11 | Bosses spawn minibosses | Broken (King Dodongo only) | Dodongo's Cavern black screen, caused by our edited static map file; King Dodongo is switched out, the other four are in | Batch 3: four Dodongo variants to find the cause; walk into the other four boss rooms |
| 12 | Bosses spawn minions | Half tested | Maps load (rc 4) | Batch 3 |
| 13 | No dupe items | Done (bottled fairy) | Confirmed by the user (18 Nov): a second fairy could not be bottled while holding one, and the refusal animation is right | Other duplicate cases (gifts, other bottle contents) not tried |
| 14 | Enemies respawn sooner | Built, untested | Nothing | Batch 2 |
| 15 | No Z-targeting | Done | Confirmed by the user at the controls (18 Nov): enemies cannot be locked on to | None |
| 16 | Fairies help enemies | Built, untested | No crash in the scripted spawn-and-kill test (18 Nov). Never seen doing its job | Needs a fairy used near a hurt enemy with enemy health visible. The "respawn the last defeated enemy" half is not possible |
eview-fixes, not yet seen in the game) |
| 17 | Enemies call for backup | Broken: crashes | Scripted test, twice (18 Nov): crashes within seconds of an enemy appearing near Link, alone or with regen and fairies. The helper has never been seen | Fix the way its flags are stored on enemies (a started fix is saved), then rerun the crash test. Leave off |
eview-fixes, not yet seen in the game) |
| 18 | Zora's Domain ambush | Half tested | Map loads (rc 4) | Batch 3 |
| 19 | Bean Seller's pitch | Done | User (18 Nov): the dialogue works | None |
| 20 | Enemies block the way | Built (round 2), untested | v5 in the game: gates 3, 4, 5, 13 worked, gate 6 perfect; gates 1 and 2 never opened, gate 11 up at the entrance | Round 2 on feature/gates-r2: gates 1 and 2 create their own Mad Scrub / Red Tektite, trigger box kept on the guard side of every wall, gate 12 two Stalfos, gates 9 + 10 one Stalfos, gate 13 barrier 0.5 lower. Red tint cause not found. Test with playtest/gates-r2/TESTS.md |
| 21 | Volvagia over Hyrule Field | Not possible | Not built | Write-up in `features\21-volvagia-roams-hyrule-field.md` |
| 22 | Special encounters (one checkbox): Dead Hand in hole 9 (Hyrule Field) and hole 23 (Death Mountain Trail), Iron Knuckle in the open part of Hyrule Field | Done, two tweaks wanted | All three confirmed in the game. Hole 23 Dead Hand: script. **Hole 9 Dead Hand: user entered with the Song of Storms and found him.** Iron Knuckle: wakes without being hit (script and user). Maps load, child and adult; hole 23's exit works | Tweaks from the user: move Dead Hand back from the landing spot (he attacks the moment Link lands); the knight should be awake from the start (built on `feature/knight-roam`, untested; roaming is not possible). Session fixes: Dead Hand moved back in both holes (10.5 and 12.8 from the landing spot), untested. Ten Like Likes at Lake Hylia (`feature/likelikes`, code only, 18 Nov; both ages, same Hyrule Field maps): not played, new enemy kind in the map, see `playtest\likelikes\TESTS.md`. Round 2 (`feature/enc-r2`): Dead Hand appear range cut from 6 to 3.5 in both holes; he notices Link from the landing spot (music at 18 units) whatever the placement, so check whether the rise now feels far enough back |
| 23 | Grass and pot surprises (part of special encounters): 1 in 100 that cut grass or a broken pot spawns a Skulltula (grass) or Poe (pot), or a small crawling hand (both; branch `feature/surprise-hands`, exe `playtest\hands\ootrr.exe`, test hook env `OOTRR_SURPRISE_CHANCE` 1 to 20), and 1 in 10 plain pots are flying pots | Built, untested (code only) | Nothing. Commit `163112b` on `feature/surprises`; 138 difficulty and 107 settings tests pass | Play it: cut about 300 blades and break about 200 pots; look for fake pots in Zora's Domain and Kakariko. Riskiest: whether a drop can create an enemy. Steps in `playtest\surprises\TESTS.md`. Round 2 (`feature/enc-r2`): rate now 2 percent per surprise (1 in 50), the bug grass round the hole 9 Dead Hand (`Land_kokiri_Plant_A_02_DropFixed_03`) now gets the grass surprises; Market guard house pots are fixed rupee pots and stay as they are |
| 24 | Cucco revenge on the first hit (setting 131, `cucco_revenge`) | Built, untested (code only) | Nothing. Commit `f9b7c87` on `feature/surprises`. The hit threshold `StartSpecialAttackCount` goes 4 to 1 in the Cucco hit graphs; Anju's and the find-out Cuccos are left alone | Play it: one sword hit on an adult Kakariko Cucco; carry and throw one. If one hit is not enough, set `FIRST_HIT` to 0. Steps in `playtest\surprises\TESTS.md` |
| 25 | Spawn enemies with L + R + D-pad (setting 132, `enemy_spawn_combo`): Down spawns a random ordinary enemy in front of Link, Up spawns the same again | Built, untested (code only) | Nothing. Commit `ab226b0`, v2 (43 kinds, 12 at once) `2c21522` on `feature/spawn-combo`, exe in `playtest\spawn-combo2`; 9 new tests, 132 difficulty and 107 settings tests pass | Play it: hold L and R, press D-pad Down, then Up. The controller read, hold mask into the press checker, and a holder on the player are guesses. Steps in `playtest\spawn-combo\TESTS.md` |
| 26 | Small hands and floor Skulltulas from grass and pots (new request, saved for later) | Built in code, untested (see 23) | Requested 18 Nov: the small crawling hands (the Floormaster's little ones) and the floor Skulltulas sometimes appear instead of bugs or rupees when grass is cut or a pot is broken. Extends the grass and pot surprises (23) | After the current build is polished and out |
| 27 | "No ammo drops" becomes "Limited ammo" (change request, saved for later) | Not started | Requested 18 Nov: ammo still drops in boss fights and anywhere else the player cannot flee and would otherwise have no option but to die (the user thinks that is only bosses). Also answers the review's finding that no ammo drops plus shuffled shops can leave no arrow source | After the current build is polished and out |

## Queue for the game (one copy at a time)

Batch 1 (18 Nov) was stopped early with nothing confirmed: scripts cannot sword-fight, aim, or work the pause menu reliably. So testing is split by what each side is good at.

**Scripted (stand still, warp, read numbers, take pictures):**

1. **S1, running:** forest gate v2 (20), Iron Knuckle awake build (22), enemy speed (03), call for backup (17).
2. **S2:** boss minibosses and minions (11, 12) with the Dodongo variants, Zora ambush (18), more enemies pictures (10).
3. **S3:** respawn sooner (14), fairies help enemies (16) if a stand-still check exists; else to hands-on.

**Hands-on with the user (script sets up and logs, the user plays; advance by button press):** no heart drops (04), no ammo drops (05), no gossip fairies (06), no bottles in combat and the fairy revival (08), enemy regen (09), no dupe items (13), bean pitch (19), gate: kill the guards (20), Dead Hand in hole 9 (22). The list with setup and what to look for is `capture-playbook.md`.

Then: fix what failed, replay only those, merge the passing branches and cut rc 9.

## Done today besides features

- **rc 10 cut (18 Nov late):** commit `ecbdb04` on `difficulty`, build in `builds
_2026-11-18_rc10` with CHANGELOG.md and TODO.md. Local only: waiting for the user's review and push. Seven gates in, five held back; what does not work is in `docs\TODO.md`.
- **rc 11 (18 Nov evening):** `builds	_2026-11-18_rc9-spawn-combo`: rc 9 plus the Longshot tools with enemy spawning on L+R+D-pad (Down = random, Up = same again), every press counted. Confirmed by the user. Local only.
- **rc 10 (18 Nov evening):** `builds_2026-11-18_rc9-with-spawn-keys`: the rc 9 randomizer plus a launcher for the source-built Longshot with the spawn keys (F4 random enemy, F3 same again, F9 screenshot). The spawn keys are a Longshot tool, not part of the mod. The launcher itself is not tried yet.
- **rc 9 cut (18 Nov evening):** commit `e303654` on `difficulty`, build in `builds 9_2026-11-18_rc9`. Local only, not pushed. From now on a release candidate is cut each time a major feature finishes testing.
- Release branch `difficulty`: one commit per release candidate, rc 1 to rc 8, docs in `difficulty-mod/docs/`, nothing pushed. Builds 007 and 008 are in `builds\`.
- Longshot test tools (local patch, `<local path>`): warp to coordinates, on-screen captions, give item, input recorder. Recording real input is not yet tried.
- Hole catalogue: 28 of 30 holes pictured from inside, `playtest\holes\index.html`. Holes 27 and 29 (Gerudo) not reached.
- Capture playbook for things that need the user's hands: `capture-playbook.md`.

## Not yet merged

These branches hold work that is not in `playtest/difficulty-all` or on `difficulty` yet. They are merged when their test passes.

| Branch | What | Waiting for |
|---|---|---|
| `feature/heart-final` (from `heart-v5b`) | Lose heart on death, new design | The final test now running |
| `feature/dead-hand-holes` | Special encounters (feature 22) | Its load test |
| `feature/dodongo-fix` | Nothing yet (investigation only) | Batch 3 variants |

## Open questions for the user (none blocks the work)

- Holes 27 and 29 could not be photographed. Not needed unless you want them.
- Volvagia over Hyrule Field: judged not possible. Read the write-up and say if you want a different take on the idea.
