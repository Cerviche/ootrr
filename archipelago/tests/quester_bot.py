import asyncio
import sys
import time


def main(address: str, name: str, delay: float) -> None:
    sys.argv = [sys.argv[0], "--nogui"]
    import ModuleUpdate
    ModuleUpdate.update(yes=True)
    from CommonClient import CommonContext, logger, server_loop

    class Bot(CommonContext):
        game = "APQuest"
        items_handling = 0b111

        async def server_auth(self, password_requested: bool = False) -> None:
            await self.get_username()
            await self.send_connect(game=self.game)

    async def run() -> None:
        ctx = Bot(address, None)
        ctx.auth = name
        ctx.server_task = asyncio.create_task(server_loop(ctx))
        sent = False
        seen = 0
        start = time.time()
        while not ctx.exit_event.is_set():
            await asyncio.sleep(1)
            if ctx.slot is None:
                continue
            if not sent and time.time() - start >= delay:
                locs = sorted(ctx.missing_locations)
                await ctx.send_msgs([{"cmd": "LocationChecks", "locations": locs}])
                print(f"{name} checked all {len(locs)} of its locations, so their items go out now", flush=True)
                sent = True
            while seen < len(ctx.items_received):
                n = ctx.items_received[seen]
                seen += 1
                print(f"{name} received {ctx.item_names.lookup_in_slot(n.item)} from {ctx.player_names.get(n.player, n.player)}", flush=True)

    asyncio.run(run())


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "localhost:38281", sys.argv[2] if len(sys.argv) > 2 else "Quester", float(sys.argv[3]) if len(sys.argv) > 3 else 20.0)
