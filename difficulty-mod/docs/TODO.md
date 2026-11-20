# Difficulty settings: what does not work yet

Newest state first. "Seen" means seen in the running game. Items move to the changelog when they work.

## Broken or unsafe

Nothing known at the moment.

## To confirm by hand

| What | State | What to do |
|---|---|---|
| More enemies slider, rc 15 rules | Played at 10x in rc 14; the rules that came out of it (peaceful places, perched Keese, big enemies apart, Iron Knuckle cap, nothing in lava, no added Dodongos in Dodongo's Cavern) are built, not all seen | Play child Hyrule Field, Kokiri Forest, Kakariko and Dodongo's Cavern at 10x; then 2x and 5x; a room whose door waits for its enemies; die in a dungeon and check the retry is safe |
| Enemies call for backup, lesser kind | Built (rc 15), not played | Fight a Stalfos (a Stalchild should come), a Wolfos (two Keese), a Lizalfos (a Keese). Eight callers called nobody in an open-field script (Stalchild, Freezard, Skulltula, Wallmaster, Armos, Spike, Tailpasaran, Biri): check some in their homes |
| Does a kill free one of the 12 spawn slots? | Unknown | Spawn 12, kill some, spawn again |
| Boss adds in the other rooms | Seen with Gohma and Phantom Ganon (by hand) and Twinrova (by script) | Barinade, Volvagia, Morpha, Bongo Bongo: walk in, the adds come with the fight |
| Shadow Temple with "Bosses spawn minions" | Black screen before the rc 16 fix; the fix is not loaded in the game yet | Load the Shadow Temple with the setting on, then fight Bongo Bongo |
| Fireball storms, other stretches | Middle road and castle garden seen | Hyrule Field south-central and west road, child and adult |
| Gates with dungeon entrance shuffle | Gates 9 present and 2 absent (script) | Gates 1 and 6 absent |
| Jabu-Jabu water spawns | Positions fine (script) | Look at the scrub room pool at 10x: no Octorok under a ceiling |
| Zora's Domain ambush | Map loads, Skulltulas present | Walk the cave |
| The held-back gate 7 | Never tried | Walk the Lost Woods east passage, kill the guard, see the barrier drop |
| Dead Hand in the Hyrule Field hole | Rises near the entrance; left as is by decision | None |
| Fake pots | Never seen | Break pots until one flies |
| Temple of Time Beamos (rc 17) | Built, never seen | Adult, after Sheik's first talk: do ten Beamos stand between the pillars, do they hit Link in the aisle, do they leave the Light Arrows scene alone |
| Storm rocks from the volcano (rc 17) | Rocks seen at every new spot by script; direction not seen | Stand at a spot and watch which way the rocks arc in |
| Lake Hylia Octoroks and Mad Scrubs (rc 17) | Seen by script | Swim at the bridges (3 to 4 at once took 15 hearts); kill a Mad Scrub; look at the adult field scrubs |
| Volvagia in Hyrule Field (rc 17, Q17) | Passed by script, hammer kill and respawn | Fight her by hand; is about 3 hearts a hit every 6 s too harsh; win the real Fire Temple fight with the setting on and see her gone |
| Mega Like Likes (rc 17) | Passed by script | Get swallowed (shield gone), kill one (shield back), as a child and an adult; is 3x life right |
| Gates 14 to 23 (rc 17) | Passed by script, kills by damage command except gate 20 | Real sword kills at each, gate 20 at night, Nocturne arrival at the Graveyard, gate 22 with the lake drained, the vanilla Wasteland Leevers |
| Random fuses (rc 17) | Bombs and Bombchus by script | Sticks, bomb flowers and Bombchu Bowling by hand |
| Limited ammo round 2 (rc 17) | Not played | Walk the Deku Tree, Dodongo's Cavern, the Fire Temple eye switch and Ganon's Castle with the setting on |
| Enemy randomizer v3 (rc 17) | Not played | Load Bottom of the Well, Dodongo's Cavern, Ice Cavern, child Spirit Temple; judge movement and walls |
| Skip area intros (rc 17) | Fire Temple entrance seen by hand | Walk into the other areas and dungeon entrances |
| Temple of Time Beamos reach (rc 18) | Run down the aisle costs 0 to 1.5 hearts (script) | Adult, after Sheik's first talk: by hand, and the Light Arrows scene |
| Random enemy sizes (rc 18) | Wild by script in 5 maps | Mild, walls, the mix with other settings, 27 MB with health following size |
| Shorter timers (rc 18) | Switch and torch times by script | Win the Dodongo's Cavern torch room at 50%; the eye switches |
| Damage taken (rc 18) | Played by script incl. fairy revive | Pit and lava respawn, grabs, one hit kills by hand |
| Bottle refusal notice (rc 18) | Seen by script | Look at the notice wording in a fight |

## Not built yet (asked for)

- The Gerudo fighter as a placed enemy (she plays a fortress scene when beaten), and Volvagia or other bosses outside their rooms.
- Enemy speed: most enemies ignore the setting (see enemy-speed.md). Making the others follow it is open.
- Mido as a blocker; lose-heart surviving a quit.

## Gates

Eleven of the first twelve were played by hand in earlier sessions (the Death Mountain Trail one with its Dodongo guard: "perfect"). Gates 14 to 23 (rc 17) passed by script only. Gate 7 (Lost Woods, east passage) is held back until someone tries it.

## Works, with a known limit

- **Lose heart on death:** the loss lasts for the play session; quitting and continuing restores the hearts.
- **Limited ammo with shuffled shops:** a seed can end up with no source of arrows; boss fights keep their ammo (round 2, rc 17, not played).
- **Dead Hand in the Hyrule Field hole** rises right by the entrance, although he is placed at the back of the room.
- **Pots in the Market guard house** give no surprises.
- **Bosses spawn minibosses:** King Dodongo is left out; adding a miniboss there hung his dungeon.
- **Fireball storms:** child Hyrule Field ran at about 30 to 37 frames a second with 5x more enemies plus the storm.

## Never seen in the game

Enemy speed by hand (measured by script: most enemies ignore it, see enemy-speed.md), enemies respawn sooner, Zora's Domain ambush, the ten Like Likes at Lake Hylia, fake pots.

## Ideas saved for later

- Grass that normally gives bugs joins the surprise pool.
- The Gerudo fighter as a gate guard.
- Mido as a visible blocker.
- A bomb-proof, glowing wall was replaced by the purple barrier; nothing further planned.
