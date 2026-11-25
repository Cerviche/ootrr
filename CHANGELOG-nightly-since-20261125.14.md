# OOTRR nightly: changes since 0.9.0-nightly.20261125.14

More Archipelago checks that could be sent for the wrong spot, found by looking for the same cause as the Song of Time report. Covers the nightly branch from `2746d25` (the source of nightly 0.9.0-nightly.20261125.14).

**Make your player file again and regenerate the room** to get these. The client part also works in rooms made before.

## Archipelago

- **Breaking one pot no longer sends another pot's check.** Some pots share their map hash with another pot: KF House of Twins Pot 1 with the Know it All House pots, and Barinade Pots 1 to 5 in Jabu-Jabu's Belly with each other. Breaking one counted as all of them. A flag or key that more than one location would use no longer counts for any of them, so these 7 pots (Full shuffle only) hold only your own items. Every other preset is unchanged.
- **The four Gerudo jail keys** (when hideout keys are shuffled) were seen through a flag that only the key itself sets, so a jail holding another item never sent its check. Those jails now hold only your own items, like the song spots.
