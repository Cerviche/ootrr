# Difficulty features: status board

Last updated: 19 Nov. rc 11 is pushed to the user's fork; work resumed after a usage pause.

Goal set by the user: every requested feature is either **Done** (seen working in the game, polished) or **Blocked** (investigated, with a written reason and the question for the user). Nothing stays at "built, never played".

## Right now

| What | State |
|---|---|
| **Has the game** | Scripted checks (Sonnet), restarted: regen with enemy health visible, health scaling, load tests of the six held-back gates, the Deku Baba's normal colour, Like Likes |
| **Next for the game** | The Opus agent's crash retest and spawn probes |
| **Then** | Switch on the gates that pass, merge the fixes that work, cut rc 12. Hands-on items go to capture-playbook.md |
| **Running without the game** | Opus agent, restarted: the backup crash and the spawn combination (takes the game after the scripted checks) |
| **Waiting on the user** | Nothing blocks the work. Open questions are in the last section |

## Features

States: **Done** = seen working in the game. **Works, finishing** = seen working, polish or follow-up checks under way. **Built, untested** = never seen in the game. **Half tested** = part seen. **Broken** = seen failing, being fixed. **Not possible** = judged not doable with data edits.

| # | Feature | State | What was seen in the game | Next step |
|---|---|---|---|---|
| 01 | Enemy health | Done | Scaling seen (19 Nov): at enemy_health 200 the overlay shows Stalfos 36/36 and Lizalfos 18/18, exactly double vanilla 18 and 9 | Peahat larva not read |
| 02 | Enemy damage | Done | 3x: a bite took 6 quarter hearts instead of 2 (script, rc 5) | Grabs (ReDead, Like Like) not covered; parked |
| 03 | Enemy speed | Built, untested | Nothing (batch 1 aborted before this) | Batch 1 |
| 04 | No heart drops | Done (small sample) | User (18 Nov): about 20 grass cut in the Lost Woods, only rupees and bugs dropped | Hearts the map places on the ground are left alone (user: fine unless they respawn; to check from the files) |
| 05 | No ammo drops | Done (small sample) | Same run as 04: no ammo dropped | Pots and enemy kills not sampled yet. With Shopsanity on, the note now warns that a special deal can replace an ammo shelf (review finding 3) |
| 06 | No gossip fairies | Done | User (18 Nov): none of the six songs produced a fairy at a gossip stone | None |
| 07 | Lose heart on death | Done, with one limit | 6 → 5 → 4 → 3 hearts over three deaths; a 4th death stays at 3; no side effects. The final build was replayed: 6 → 5 → 4 (script, 18 Nov). Confirmed by hand too (user died without a fairy: back with 5 hearts). Merged into `playtest/difficulty-all` | Limit: the loss does not survive quitting and continuing (accepted as "simplest for now"). Fairy revival costs no heart: confirmed by the user (revived at 6 hearts max) |
| 08 | No bottles in combat | Done | Confirmed by the user at the controls (18 Nov): a bottled fairy could NOT be used while the Iron Knuckle was aggro and COULD once far enough away; the refusal animation is right; a bottled fairy still revived Link automatically in the fight (6 hearts kept). Found and fixed on the way: the block ignored miniboss music (commit 630870b) | Not seen against a full boss; low risk |
| 09 | Enemies regen health | Done | Seen (19 Nov): Stalfos hit 18 -> 11, then +1 hp about every 2 s back to 18/18 in 15 s with Link doing nothing; control with no settings stayed at 12/18 for 60 s | None |
eview-fixes, not yet seen in the game) |
| 10 | More enemies (slider) | Half tested | Maps load (rc 4) | Batch 3: positions, the "red enemies" look |
| 11 | Bosses spawn minibosses | Broken (King Dodongo only) | Dodongo's Cavern black screen, caused by our edited static map file; King Dodongo is switched out, the other four are in | Batch 3: four Dodongo variants to find the cause; walk into the other four boss rooms |
| 12 | Bosses spawn minions | Half tested | Maps load (rc 4) | Batch 3 |
| 13 | No dupe items | Done (bottled fairy) | Confirmed by the user (18 Nov): a second fairy could not be bottled while holding one, and the refusal animation is right | Other duplicate cases (gifts, other bottle contents) not tried |
| 14 | Enemies respawn sooner | Built, untested | Nothing | Batch 2 |
| 15 | No Z-targeting | Done | Confirmed by the user at the controls (18 Nov): enemies cannot be locked on to | None |
| 16 | Fairies help enemies | Built, untested | No crash in the scripted spawn-and-kill test (18 Nov). Never seen doing its job | Needs a fairy used near a hurt enemy with enemy health visible. The "respawn the last defeated enemy" half is not possible |
eview-fixes, not yet seen in the game) |
| 17 | Enemies call for backup | Fixed, scripted test passed | Opus fix (19 Nov, `feature/opus-fixes` e4ff217, exe `playtest\opus-fixes`): the flags were placement keys, which no game graph writes; now plain keys. Backup alone: no crash with two Lizalfos beside Link (old build crashed in 4 to 8 s); one Peahat larva per Lizalfos 10 to 16 s after it appeared, none again after they died | User: fight one enemy for 15 s, check one helper; try Jabu-Jabu and Ice Cavern helpers and a miniboss (must call nothing) |
eview-fixes, not yet seen in the game) |
| 18 | Zora's Domain ambush | Half tested | Map loads (rc 4) | Batch 3 |
| 19 | Bean Seller's pitch | Done | User (18 Nov): the dialogue works | None |
| 20 | Enemies block the way | Loads checked, kills not | Seen (19 Nov), all six held gates opened in a test exe: all five maps load (meadow, Death Mountain Trail, Lost Woods, Market, Gerudo Valley), no crash. Guards listed: gate 1 Mad Scrubs, 2 Red Tektites, 11 two ReDeads (middle-boss music), 12 two Stalfos (battle music). Guard kill and barrier gone not confirmed (Din Fire casts unreliable) | Look by hand at kills and barriers; decide whether to release the held gates. Deku Baba red is its normal colour |
| 21 | Volvagia over Hyrule Field | Not possible | Not built | Write-up in `features\21-volvagia-roams-hyrule-field.md` |
| 22 | Special encounters (one checkbox): Dead Hand in hole 9 (Hyrule Field) and hole 23 (Death Mountain Trail), Iron Knuckle in the open part of Hyrule Field | Done, two tweaks wanted | Like Like present too (19 Nov): Enemy_LikeLike 12/12 listed on the Lake Hylia island in Hyrule Field, loads fine. Earlier: Dead Hand and Iron Knuckle confirmed in the game | Tweaks as before |
| 23 | Grass and pot surprises (part of special encounters): 1 in 100 that cut grass or a broken pot spawns a Skulltula (grass) or Poe (pot), or a small crawling hand (both; branch `feature/surprise-hands`, exe `playtest\hands\ootrr.exe`, test hook env `OOTRR_SURPRISE_CHANCE` 1 to 20), and 1 in 10 plain pots are flying pots | Built, untested (code only) | Nothing. Commit `163112b` on `feature/surprises`; 138 difficulty and 107 settings tests pass | Play it: cut about 300 blades and break about 200 pots; look for fake pots in Zora's Domain and Kakariko. Riskiest: whether a drop can create an enemy. Steps in `playtest\surprises\TESTS.md`. Round 2 (`feature/enc-r2`): rate now 2 percent per surprise (1 in 50), the bug grass round the hole 9 Dead Hand (`Land_kokiri_Plant_A_02_DropFixed_03`) now gets the grass surprises; Market guard house pots are fixed rupee pots and stay as they are |
| 24 | Cucco revenge on the first hit (setting 131, `cucco_revenge`) | Built, untested (code only) | Nothing. Commit `f9b7c87` on `feature/surprises`. The hit threshold `StartSpecialAttackCount` goes 4 to 1 in the Cucco hit graphs; Anju's and the find-out Cuccos are left alone | Play it: one sword hit on an adult Kakariko Cucco; carry and throw one. If one hit is not enough, set `FIRST_HIT` to 0. Steps in `playtest\surprises\TESTS.md` |
| 25 | Spawn enemies with L + R + D-pad (setting 132, `enemy_spawn_combo`): Down spawns a random ordinary enemy in front of Link, Up spawns the same again | Fixed, scripted test passed | Opus fix (19 Nov, `feature/opus-fixes` 4efaaaf): every create used holder slot 0, so only the first press worked; now a slot counter. Bridge L+R+Down: 12 presses, 12 different enemies 3 m ahead (Up repeated the kind), the 13th and 14th nothing (cap 12). The user probably saw nothing because Longshot's own L+R+D-pad tool shortcut takes the D-pad | User: real pad with the Longshot shortcut off; check that a kill frees a slot |
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

- **rc 11 cut (19 Nov):** `difficulty` now has upstream merged (commit `a3550be`, 32 upstream commits, new `randomizer/` layout) and rc 11 on top (`c8d4b34`); build in `builds_2026-11-19_rc11`. Local only: rc 10 is on the user's fork, rc 11 waits for review and push. All docs are in `difficulty-mod/docs` (34 files, scanned for personal details).
- **Longshot debug panel shows enemy health** (`hp CUR / MAX`), seen working: a bomb took a Stalfos from 18 to 16.
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
