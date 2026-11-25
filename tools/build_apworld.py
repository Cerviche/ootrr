import argparse
import json
import os
import subprocess
import sys
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "archipelago", "ootrr")
SKIP = {"__pycache__", ".gitignore"}
CONTAINER = 7


def main() -> int:
    ap = argparse.ArgumentParser(description="Export the OOTRR logic bundles and pack ootrr.apworld.")
    ap.add_argument("--ootrr", required=True, help="path to the ootrr executable built from this tree")
    ap.add_argument("--out", default=os.path.join(ROOT, "ootrr.apworld"), help="where to write the apworld")
    a = ap.parse_args()
    data = os.path.join(SRC, "data")
    r = subprocess.run([a.ootrr, "ap-export", data])
    if r.returncode != 0:
        print("ap-export failed", file=sys.stderr)
        return r.returncode
    n = 0
    with zipfile.ZipFile(a.out, "w", zipfile.ZIP_DEFLATED) as z:
        for base, dirs, files in os.walk(SRC):
            dirs[:] = sorted(d for d in dirs if d not in SKIP)
            for f in sorted(files):
                if f in SKIP or f.endswith(".pyc"):
                    continue
                p = os.path.join(base, f)
                arc = os.path.join("ootrr", os.path.relpath(p, SRC)).replace(os.sep, "/")
                if arc == "ootrr/archipelago.json":
                    with open(p, encoding="utf-8") as m:
                        manifest = json.load(m)
                    manifest.update({"version": CONTAINER, "compatible_version": CONTAINER})
                    z.writestr(arc, json.dumps(manifest, indent=1))
                else:
                    z.write(p, arc)
                n += 1
        for extra in ("LICENSE", "NOTICE"):
            p = os.path.join(ROOT, extra)
            if os.path.isfile(p):
                z.write(p, f"ootrr/{extra}")
                n += 1
    print(f"wrote {a.out} with {n} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
