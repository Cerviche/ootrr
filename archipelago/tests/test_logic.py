import json
import os
import sys
import time
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ootrr import logic

PROBES = os.environ.get("OOTRR_AP_PROBES", "")


ROLLS = os.environ.get("OOTRR_AP_ROLLS", "")


def check(t, name, b, cases):
    w = b.world
    start = time.perf_counter()
    for k, c in enumerate(cases):
        held = {}
        for i in c["held"]:
            held[w.items[i]] = held.get(w.items[i], 0) + 1
        got = b.sweep(held)
        want = frozenset(w.loc_name[l] for l in c["reach"])
        t.assertEqual(sorted(got.reach - want), [], f"{name} case {k}: Python reaches more")
        t.assertEqual(sorted(want - got.reach), [], f"{name} case {k}: Python reaches less")
        t.assertEqual(got.won, c["won"], f"{name} case {k}")
    print(f"{name}: {len(b.free_names)} shuffled, {len(cases)} cases, {(time.perf_counter() - start) / len(cases) * 1000:.1f} ms a sweep")


def read(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


class LogicMatchesRust(unittest.TestCase):
    @unittest.skipUnless(PROBES, "set OOTRR_AP_PROBES to the folder ootrr ap-export --probes wrote")
    def test_every_probe_reaches_what_the_rust_search_reaches(self):
        for name in logic.bundle_names():
            b = logic.Bundle(logic.load_json(f"{name}.json"))
            check(self, name, b, read(os.path.join(PROBES, f"{name}.probes.json")))

    @unittest.skipUnless(ROLLS, "set OOTRR_AP_ROLLS to the folder ootrr ap-export --roll wrote")
    def test_rolled_settings_reach_what_the_rust_search_reaches(self):
        rolls = read(os.path.join(ROLLS, "rolls.json"))
        self.assertTrue(rolls)
        for r in rolls:
            b = logic.Bundle(read(os.path.join(ROLLS, f"{r['id']}.json")))
            self.assertEqual(b.settings, r["settings"])
            check(self, r["id"], b, read(os.path.join(ROLLS, f"{r['id']}.probes.json")))


if __name__ == "__main__":
    unittest.main()
