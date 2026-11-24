import hashlib
from typing import Dict, List

from .logic import Bundle, bundle_names, load_json

GAME = "OOTRR"
FILLER = "Rupee (1)"


def stable_id(kind: str, name: str) -> int:
    d = hashlib.sha256(f"{GAME}|{kind}|{name}".encode("utf-8")).digest()
    return (int.from_bytes(d[:7], "big") & ((1 << 52) - 1)) or 1


_bundles: Dict[str, Bundle] = {}


def bundle(preset: str) -> Bundle:
    b = _bundles.get(preset)
    if b is None:
        b = Bundle(load_json(f"{preset}.json"))
        _bundles[preset] = b
    return b


def presets() -> List[Dict[str, str]]:
    return load_json("presets.json")


def _tables():
    names: Dict[str, List[str]] = load_json("names.json")
    items: Dict[str, int] = {n: stable_id("item", n) for n in names["items"] + [FILLER]}
    locs: Dict[str, int] = {n: stable_id("location", n) for n in names["locations"]}
    for p in bundle_names():
        b = bundle(p)
        for n in b.pool:
            items.setdefault(n, stable_id("item", n))
        for n in b.free_names:
            locs.setdefault(n, stable_id("location", n))
    for table, what in ((items, "item"), (locs, "location")):
        seen: Dict[int, str] = {}
        for n, i in table.items():
            if i in seen:
                raise ValueError(f"OOTRR: the {what}s {seen[i]!r} and {n!r} share the id {i}")
            seen[i] = n
    return dict(sorted(items.items())), dict(sorted(locs.items()))


ITEM_NAME_TO_ID, LOCATION_NAME_TO_ID = _tables()
