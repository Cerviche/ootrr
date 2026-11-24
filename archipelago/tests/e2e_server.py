import asyncio
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
PORT = 38291


def wait_port(port: int, t: float) -> None:
    end = time.time() + t
    while time.time() < end:
        try:
            socket.create_connection(("127.0.0.1", port), 1).close()
            return
        except OSError:
            time.sleep(0.5)
    raise SystemExit(f"the server did not open port {port}")


def main(zip_path: str) -> int:
    sys.argv = [sys.argv[0], "--nogui"]
    import ModuleUpdate
    ModuleUpdate.update(yes=True)
    from CommonClient import CommonContext, server_loop
    import worlds
    from worlds.AutoWorld import AutoWorldRegister
    from worlds.ootrr.tables import bundle

    z = zipfile.ZipFile(zip_path)
    name = next(n for n in z.namelist() if n.endswith(".apootrr"))
    patch = json.loads(z.read(name))
    b = bundle(patch["preset"])
    keys = [r["key"] for r in patch["remote"]] + [s["key"] for s in patch["start"]]
    found_locs = [l for l in patch["foreign"] if b.detect.get(l, ([], []))[0]][:2]
    flags = [b.detect[l][0][0] for l in found_locs]
    state = os.path.join(tempfile.mkdtemp(), "fake.json")
    env = dict(os.environ, OOTRR_FAKE_STATE=state, OOTRR_FAKE_SEED_KEY=patch["seed_key"], OOTRR_FAKE_KEYS=",".join(keys), OOTRR_FAKE_FOUND="|".join(flags), OOTRR_FAKE_GOAL_AFTER="12")
    py = sys.executable
    server = subprocess.Popen([py, "MultiServer.py", zip_path, "--port", str(PORT), "--host", "127.0.0.1"], stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    client = None
    try:
        wait_port(PORT, 60)
        code = f"import sys; sys.argv=['ootrr', '--nogui']; from worlds.ootrr.client import launch; launch('--nogui', '--name', {patch['slot']!r}, '--ootrr', {os.path.join(HERE, 'fake_bridge.py')!r}, '--connect', 'localhost:{PORT}')"
        clog = open(os.path.join(os.path.dirname(state), "client.log"), "w", encoding="utf-8")
        print(f"client log: {clog.name}")
        client = subprocess.Popen([py, "-c", code], env=env, stdin=subprocess.DEVNULL, stdout=clog, stderr=subprocess.STDOUT)
        quest_ids = AutoWorldRegister.world_types["APQuest"].location_name_to_id
        mine = [quest_ids[r["location"]] for r in patch["remote"]]
        other = next(p for p in (1, 2) if p != patch["player"])
        result = {}

        class Quester(CommonContext):
            game = "APQuest"
            items_handling = 0b111

            async def server_auth(self, password_requested: bool = False) -> None:
                await self.get_username()
                await self.send_connect(game=self.game)

            def on_package(self, cmd, args):
                if cmd == "Connected":
                    asyncio.create_task(self.send_msgs([{"cmd": "LocationChecks", "locations": mine}]))
                if cmd == "Retrieved":
                    result.update(args.get("keys", {}))

        async def run() -> None:
            ctx = Quester(f"localhost:{PORT}", None)
            ctx.auth = "Quester"
            ctx.server_task = asyncio.create_task(server_loop(ctx))
            end = time.time() + 90
            while time.time() < end:
                await asyncio.sleep(2)
                got = {}
                if os.path.exists(state):
                    with open(state, encoding="utf-8") as f:
                        got = json.load(f)
                from_link = [n for n in ctx.items_received if n.player == patch["player"]]
                status_key = f"_read_client_status_0_{patch['player']}"
                if ctx.slot is not None:
                    await ctx.send_msgs([{"cmd": "Get", "keys": [status_key]}])
                done = set(got.get("given", [])) >= set(keys)
                if done and len(from_link) >= len(found_locs) and result.get(status_key) == 30:
                    break
            result["given"] = got.get("given", [])
            result["from_link"] = [(n.item, n.location) for n in from_link]
            ctx.exit_event.set()
            await ctx.shutdown()

        asyncio.run(run())
        ok_items = set(result["given"]) >= set(keys)
        ok_checks = len(result["from_link"]) >= len(found_locs)
        ok_goal = result.get(f"_read_client_status_0_{patch['player']}") == 30
        print(f"items handed over in the fake game: {len(result['given'])} of {len(keys)}")
        print(f"checks sent from the OOTRR slot and received by Quester: {len(result['from_link'])} of {len(found_locs)} {found_locs}")
        print(f"OOTRR goal reported to the server: {ok_goal}")
        return 0 if ok_items and ok_checks and ok_goal else 1
    finally:
        for p in (client, server):
            if p is not None:
                p.kill()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))
