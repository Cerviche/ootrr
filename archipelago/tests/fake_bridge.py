import json
import os
import sys

STATE = os.environ["OOTRR_FAKE_STATE"]
SEED = os.environ["OOTRR_FAKE_SEED_KEY"]
KEYS = [k for k in os.environ.get("OOTRR_FAKE_KEYS", "").split(",") if k]
FOUND = [k for k in os.environ.get("OOTRR_FAKE_FOUND", "").split("|") if k]
GOAL_AFTER = int(os.environ.get("OOTRR_FAKE_GOAL_AFTER", "0"))

bools = {f"OOTRR.Seed.{SEED}": True, f"OOTRR.Goal.{SEED}": False, "OOTRR.AP.Pending": False}
for k in KEYS:
    bools[f"OOTRR.AP.Recv.{k}"] = False
    bools[f"OOTRR.AP.Got.{k}"] = False
for f in FOUND:
    bools[f] = True
reads = 0
log = []


def dispatch():
    if not bools.get("OOTRR.AP.Pending"):
        return
    bools["OOTRR.AP.Pending"] = False
    for k in KEYS:
        if bools[f"OOTRR.AP.Recv.{k}"]:
            bools[f"OOTRR.AP.Recv.{k}"] = False
            if not bools[f"OOTRR.AP.Got.{k}"]:
                bools[f"OOTRR.AP.Got.{k}"] = True
                log.append(k)
            return


def save():
    with open(STATE, "w", encoding="utf-8") as f:
        json.dump({"bools": bools, "given": log, "reads": reads}, f)


for line in sys.stdin:
    r = json.loads(line)
    if r["op"] == "read":
        reads += 1
        if GOAL_AFTER and reads >= GOAL_AFTER:
            bools[f"OOTRR.Goal.{SEED}"] = True
        out = {"ok": True, "bools": {n: bools.get(n) for n in r.get("bools", [])}, "keys": {n: False for n in r.get("keys", [])}}
    elif r["op"] == "write":
        bools[r["name"]] = r["value"]
        dispatch()
        out = {"ok": True, "was": False}
    else:
        out = {"ok": True, "protocol": 1}
    out["id"] = r.get("id")
    save()
    sys.stdout.write(json.dumps(out) + "\n")
    sys.stdout.flush()
