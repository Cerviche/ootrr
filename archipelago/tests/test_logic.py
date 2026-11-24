import json
import os
import sys
import time
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ootrr import logic

PROBES = os.environ.get("OOTRR_AP_PROBES", "")


class LogicMatchesRust(unittest.TestCase):
    @unittest.skipUnless(PROBES, "set OOTRR_AP_PROBES to the folder ootrr ap-export --probes wrote")
    def test_every_probe_reaches_what_the_rust_search_reaches(self):
        for name in logic.bundle_names():
            b = logic.Bundle(logic.load_json(f"{name}.json"))
            w = b.world
            with open(os.path.join(PROBES, f"{name}.probes.json"), encoding="utf-8") as f:
                cases = json.load(f)
            t = time.perf_counter()
            for k, c in enumerate(cases):
                held = {}
                for i in c["held"]:
                    held[w.items[i]] = held.get(w.items[i], 0) + 1
                got = b.sweep(held)
                want = frozenset(w.loc_name[l] for l in c["reach"])
                self.assertEqual(sorted(got.reach - want), [], f"{name} case {k}: Python reaches more")
                self.assertEqual(sorted(want - got.reach), [], f"{name} case {k}: Python reaches less")
                self.assertEqual(got.won, c["won"], f"{name} case {k}")
            print(f"{name}: {len(cases)} cases, {(time.perf_counter() - t) / len(cases) * 1000:.1f} ms a sweep")


if __name__ == "__main__":
    unittest.main()
