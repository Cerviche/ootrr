# Enemies regen health

Setting `enemy_regen` (number 118), checkbox. Code: `crates/core/src/difficulty/regen.rs`. Built by a sub-agent on `feature/enemy-regen` (`d582934`).

## Request
"enemies regen health"

## Interpretation
While alive, enemies slowly recover health, never above their maximum. I left the rate to the agent, asking for something small every few seconds.

## Investigation
- No parameter for regeneration exists, so it had to be added to enemy AI.
- Nearly every enemy's AI calls one tiny shared graph, `AI/EnemyCommon.common.reactiondamagecolor.module.ainb`, in the global AI pack. Stalfos use an identical private copy in four packs. That graph is the hook: one edit reaches about 123 actors.
- The heal itself is the node `OneShotLifeSetOwnLife`. The player's "heal" event uses it with mode 0 (add and clamp to max), and a Floormaster graph proves it works on enemies.
- A looping frame counter node is used the same way in Twinrova's own graph.

## Implementation
- The graph gets a new root that runs the old node alongside a 150-frame looping timer. Each time it fires: has a life component, is not dead, add 1 health.
- Every actor that runs the graph gets a little more AI memory (`InstanceHeapSize` + 0x4000), as the Bunny Hood feature does.
- Size table entries grow for the graphs and packs.

## How it was tested
- 22 difficulty tests, including checks that each added node has the same shape as a game-authored node of the same kind.
- A real build with health at 2x and regen on; the built graph has 11 nodes in both places.
- Found and worked around a bug: the repo misreads the compression dictionary of the global AI pack.

## In the game
Never played. To test:
- Turn this on with enemy health at 3x or more, so there is health to regain.
- Hit a Stalfos or Lizalfos a few times, back off for 30 seconds, then count the hits to kill it against a fresh one.
- Pass: it takes noticeably more hits after the wait.
- Watch bosses with health phases (Volvagia, King Dodongo, Twinrova): healing back over a phase line may replay a phase.
- A crash when any enemy loads means the graph edit is rejected; report the map.

## Notes
- Rate is one point per 150 frames, about five seconds if the game counts 30 per second. That assumption is unverified.
- Enemies with 1 or 2 health show no effect.
- Not covered: Beamos, Anubis, flying pots, Shabom, Peahat larvae, Gohma eggs, and boss parts such as Morpha's tentacle.
- This shared graph is the best hook found for "make every enemy do X". Fairies-help-enemies and call-for-backup reuse it. Order matters: regen must be applied first.
