# OOTRR nightly: changes since 0.8.3-nightly.20261123.5

Covers the nightly branch from `c179e4b` (the source of nightly 0.8.3-nightly.20261123.5) to `dda8087` (the source of nightly 0.8.3-nightly.20261124.7): PRs #116, #117 and #118, plus one commit that makes the tests pass after the merge.

The adult trade quest can now be shuffled item by item. Holding the D-pad switches masks and boots without pausing. Excluded locations are always shuffled and only hold junk. Fixes for the Gerudo Membership Card, Link the Goron, the Goron City pot, Ocarinas picked up off the ground, dungeon items that gave nothing, chest sizes, oversized pickups and sunken shop goods.

**Only part of this has been checked in the game** (noted per entry). Back up your saves.

## Adult trade shuffle

- **Shuffle adult trade items** (Locations > Rewards, needs Shuffle trade checks): the eleven adult trade items, Pocket Egg to Claim Check, are each their own item and can be anywhere. The ten trade hand ins give only their shuffled item.
- You keep every trade item you find and can hand them over in any order.
- **One trade item shows at a time**: the one you picked. It is the one in the menu's trade slot, in the L/R list and on the item button. The others you hold wait out of sight. *Checked in game.*
- **Switching:** press A on the menu's trade slot again, and the next trade item you hold comes up (the menu blinks while it redraws the slot). Showing a trade item where nobody takes it does the same. Each trade item's description now ends with "Press A again to switch trade items." *The menu switch is checked in game.*
- **After a trade** the item button falls back to the trade item before the one you traded, instead of going empty. *Not checked in game.*
- The quest follows the earliest trade item you hold, so Grog and Fado appear for it, and Anju's first adult gift is still offered. Biggoron's sword no longer ends the quest for the other hand ins.
- The Pocket Egg only hatches while you hold no Pocket Cucco.
- Logic: each trade spot needs its own item. The Lost Woods trades also need adult Kakariko by day, and only the Pocket Cucco (not the egg) counts for Anju's cucco trade.

## Quality of life (#118)

- **Hold D-pad up** to switch to the next mask or the Lens of Truth you own (every mask once the mask quest is complete). **Hold D-pad down** to switch between the Iron and Hover Boots. Keep holding to keep switching. A tap still puts the gear on or takes it off. Always on. *The switching is checked in game.*
- **No hop when changing boots.** *Not confirmed in game that the boots still switch in every case.*
- **No "next objective" card** when a quest step moves on. Threads of Time no longer lists the game's own quests, so the card pointed at nothing. *Not checked in game.*

## Excluded locations (#117, Ontico)

- An excluded location is **always shuffled**, even when its kind of check is not, so its vanilla item moves elsewhere.
- It only holds **junk**: after the fill, each excluded spot is set to a Green or Blue Rupee (70%) or a small ammo bundle (30%). It no longer takes hearts, masks, maps or other items the logic skips.
- Songs that must stay on song spots are dropped from the exclusions, with a note.

## Fixes (#118)

- **Gerudo Membership Card with "Rescue one":** the guard's check is shuffled like in the other fortress modes. It holds the card unless Shuffle Gerudo Membership Card is on, then a random item. Before, "Rescue one" always kept the card vanilla.
- **Link the Goron (adult):** he rolls until you stop him with a bomb and then gives his check, whatever step the Fire Temple quest is on. Before, once the quest had moved on he could not be stopped. The tracker marks the check from the gift itself. *Not checked in game. A save that already took his late gift under the old build will see him roll again.*
- **Goron City spinning pot:** a winning spin now stops the pot and gives the placed item once. Before, the pot kept spinning, ignored the next bomb and the HUD never came back.
- **Ocarina from the ground or a Gold Skulltula** can be played at once. Before, it only worked after a reload. *Not checked in game.*
- **Ocarina of Time pickup** shows its proper model.
- **Ocarinas in chests** are handed out as the pickup, then added and equipped, so a progressive Ocarina chest gives the right one.
- **Dungeon items on the ground or from skulltulas that gave nothing** (no box, no key count): the item packs now carry the same pouch settings as their item rows, so the game no longer treats them as the current room's own keys. *Reported as "sometimes"; not confirmed fixed in game.*
- **Chest size matches the content everywhere:** a chest now always changes size for what it holds, even next to walls, pots or other chests, where it may clip. Story chests, special models and the Dampe pair keep their size.
- **Freestanding items are no longer enlarged.** They stand at the size of the game's own pickups.
- **Shop special deals** sit on the shelf board instead of sinking into it. *Not checked in game.*

## Menu (#116, #117, #118)

- Enemy randomizer is hidden, and the Difficulty page carries the same warning as the experimental page (#116, Ontico).
- "Night Gold Skulltulas expect Sun's Song" moved to Logic / Goal > Logic, below Skip Epona Race (#117, Ontico).
- Shuffle Gerudo Membership Card moved from its own page to Locations > Items.

## Logic

- **Dodongo's Cavern entrance:** a child needs a strength upgrade; an adult gets in without one. The lobby wall still needs the Hammer, explosives or the upgrade.
- **Market Potion Shop shelves:** a child with a bottle can buy from them. Meeting Zelda is no longer asked for.

