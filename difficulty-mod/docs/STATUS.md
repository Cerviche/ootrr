# Difficulty features: status

What is going on and what needs the user. Decisions: `DECISIONS.md`. Feature detail and results: `features\`. History: `CHANGELOG.md`, `HISTORY.md`.

## Needs your attention

- **Question Q17:** Volvagia in the field does about 3 hearts a hit every ~6 s and only the hammer hurts her. Too harsh? *Default: leave her deadly; her damage follows `enemy_damage`.*
- **Judge by hand** (steps in [capture-playbook.md](capture-playbook.md)): which way the storm rocks come from; the Octorok fire at the Lake Hylia bridges (3-4 at once took 15 hearts while swimming; the frame rate dips to 22-29 there); killing a Mad Scrub.
- **Pull requests:** paste the #52 description from `pr-drafts\difficulty-rc16.md`; open the faster blocks PR (`pr-drafts\faster-blocks.md`).

## Right now

| Running | Branch | State |
|---|---|---|

## Built, waiting for a game test

- Gates 9, 14-23 (`feature/gate-crater`): passed by script (barrier holds, drops when the guards die, map loads) for 9, 14, 15, 17, 18, 19, 21, 22, 23; 15 is absent without the sword. Gate 16 (Trail bypass closed) and gate 20 (Lon Lon: cloned Leevers, giant hp 16, all six killed with real sword hits, barrier drops) fixed and played (`2275bb9`). Left: the Nocturne arrival at the Graveyard, gate 22 with the lake drained, gate 20 at night with the clones, a look at the vanilla Wasteland Leevers.
- Mega Like Likes that eat shields (`feature/shield-likelikes`, `9dbeb72`, setting 144): passed by script. One shield per swallow (Mirror, Hylian, Deku); the Hylian and Deku come back as pickups when one dies; the Mirror goes back into the pouch with no pickup or message (the game has no Mirror pickup outside the randomizer's item builder). Left: a hand session; a sparkle or get-item scene for the Mirror is not built.
- Volvagia in the field (`feature/volvagia-field2`, `e773a90`): fixed and passed by script. She appears, scales (30/30), dies to the hammer with no body left, returns after a reload, and is gone once the Fire Temple victory flag is set. Left: the real Fire Temple fight by hand, and `enemy_damage`.


- Limited ammo round 2 (`feature/limited-ammo`, `<playtest folder>\limited-ammo`).
- Enemy randomizer v3 (`feature/enemy-randomizer`): load Bottom of the Well, Dodongo's Cavern, Ice Cavern, child Spirit Temple.
- Temple of Time Beamos (`feature/lake-octoroks`, `<playtest folder>\temple-beamos`): appear after Sheik's first adult talk; check they hit Link in the aisle and whether they act during the Light Arrows scene.
- Storms from the volcano (`feature/fireball-storms-2`): runs; direction needs eyes.

## Queue

1. Gate editor: crash on any `gate` command (v7), hang from the enemy sweep, then v5 changes: [gate-editor.md](gate-editor.md)
2. Random fuses by hand, bomb flowers and Bombchu Bowling: [31](features/31-random-fuses.md)
3. Enemy randomizer: judge movement and walls: [26](features/26-enemy-randomizer.md)
4. Then, in your order: random enemy sizes (optionally scaled health), less time on timed switches and torches, damage options up to a one-hit-KO preset
5. Next playtest build: small keys removed, boss key at start, lose heart on death, the spawn combo, the latest features
6. Bottle gives stopped being taken in the session 6 build: look into
7. Boss adds in Barinade, Morpha, Bongo Bongo: [11](features/11-boss-minibosses.md)
8. Zora cave C failed by script: [18](features/18-zora-ambush.md)
9. Bottle refusal notice probe: [08](features/08-no-bottles-in-combat.md)
10. Frame rate at 10x alone: [10](features/10-more-enemies.md); enemy speed timing: [03](features/03-enemy-speed.md)
11. Ideas not asked for yet: Gerudo fighter as a placed enemy; Mido as a blocker; lose-heart surviving a quit
## Features

States: not built, built (never seen), scripted (seen by script), seen by hand, released. "Released" means in a numbered rc or an upstream pull request.

| # | Feature | State | Left |
|---|---|---|---|
| 01 | [Enemy health](features/01-enemy-health.md) | released | Peahat larva not read |
| 02 | [Enemy damage](features/02-enemy-damage.md) | released | Grabs not covered |
| 03 | [Enemy speed](features/03-enemy-speed.md) | built | Most enemies ignore it; no hand test |
| 04 | [No heart drops](features/04-no-heart-drops.md) | released | None |
| 05 | [Limited ammo](features/05-no-ammo-drops.md) | built | Round 2 built; game test |
| 06 | [No gossip fairies](features/06-no-gossip-fairies.md) | released | None |
| 07 | [Lose heart on death](features/07-lose-heart-on-death.md) | released | Not kept across quit (accepted) |
| 08 | [No bottles in combat](features/08-no-bottles-in-combat.md) | released | Refusal notice probe |
| 09 | [Enemy regen](features/09-enemy-regen.md) | released | None |
| 10 | [More enemies](features/10-more-enemies.md) | released | Frame rate at 10x alone |
| 11 | [Boss minibosses](features/11-boss-minibosses.md) | released (rc 16) | Barinade, Morpha, Bongo by hand |
| 12 | [Boss minions](features/12-boss-minions.md) | released (rc 16) | As 11 |
| 13 | [No dupe items](features/13-no-dupe-items.md) | released | Other duplicate cases |
| 14 | [Respawn sooner](features/14-enemies-respawn-sooner.md) | built | Never tested |
| 15 | [No Z-targeting](features/15-no-z-targeting.md) | released | None |
| 16 | [Fairies help enemies](features/16-fairies-help-enemies.md) | released | Green heal seen by hand |
| 17 | [Enemy backup](features/17-enemies-call-for-backup.md) | released | Some callers call nobody |
| 18 | [Zora ambush](features/18-zora-ambush.md) | released (in more enemies) | Cave C failed by script |
| 19 | [Bean pitch](features/19-bean-sellers-pitch.md) | retired | None |
| 20 | [Gates](features/20-enemies-block-the-way.md) | released (rc 16) | Gates 14-22 built; Dark Link; editor |
| 21 | [Volvagia in the field](features/21-volvagia-roams-hyrule-field.md) | scripted | Re-test of fixes and respawning |
| 22 | [Special encounters](features/22-special-encounters.md) | released (in more enemies) | Octoroks, Mad Scrubs, Beamos added; not released |
| 23 | [Grass and pot surprises](features/23-grass-and-pot-surprises.md) | built | Fake pots never seen |
| 24 | [Cucco revenge](features/24-cucco-revenge.md) | scripted | Control run with setting off |
| 25 | [Spawn combo](features/25-spawn-enemies-combo.md) | released | None |
| 26 | [Enemy randomizer](features/26-enemy-randomizer.md) | scripted (v2 loads) | Judge movement and walls |
| 27 | [Bowling grass](features/27-unbreakable-bowling-grass.md) | released (rc 16) | None |
| 28 | [Unequip gear](features/28-unequip-gear.md) | dropped | Replaced by the shield-eating Like Likes study |
| 29 | [Fireball storms](features/29-fireball-storms.md) | released (rc 16) | More places being built |
| 30 | [Faster blocks](features/30-faster-blocks.md) | built | Own upstream pull request |
| 31 | [Random fuses](features/31-random-fuses.md) | scripted | Bomb flowers, Bowling, by hand |
| - | Skip area intros (setting 137) | scripted | Dungeon entrances seen; play by hand |
| - | Skip first pickup text (#53) | seen by hand | Chest with it on |

