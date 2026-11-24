import json
import os
import pkgutil
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

DAY = 1
DAMPE = 2
AGES = ("adult", "child")


def load_json(name: str) -> Any:
    raw: Optional[bytes] = pkgutil.get_data(__name__, f"data/{name}")
    if raw is None:
        raise FileNotFoundError(name)
    return json.loads(raw.decode("utf-8"))


def num(v: Any) -> Optional[float]:
    if isinstance(v, bool):
        return 1.0 if v else 0.0
    if isinstance(v, (int, float)):
        return float(v)
    return None


def same(a: Any, b: Any) -> bool:
    x, y = num(a), num(b)
    if x is not None and y is not None:
        return x == y
    return type(a) is type(b) and a == b


def member(a: Any, b: Any) -> bool:
    if isinstance(b, list):
        return any(same(a, x) for x in b)
    if isinstance(b, str):
        return isinstance(a, str) and a in b
    if isinstance(b, dict):
        return isinstance(a, str) and a in b
    return False


def compare(op: str, a: Any, b: Any) -> bool:
    if op == "==":
        return same(a, b)
    if op == "!=":
        return not same(a, b)
    if op == "in":
        return member(a, b)
    if op == "not in":
        return not member(a, b)
    x, y = num(a), num(b)
    if x is None or y is None:
        if isinstance(a, str) and isinstance(b, str):
            x, y = a, b
        else:
            return False
    if op == "<":
        return x < y
    if op == "<=":
        return x <= y
    if op == ">":
        return x > y
    return x >= y


def arith(op: str, a: Any, b: Any) -> Any:
    if type(a) is int and type(b) is int:
        if op == "+":
            return a + b
        if op == "-":
            return a - b
        if op == "*":
            return a * b
        if b == 0:
            return None
        if op == "//":
            return a // b
        if op == "%":
            return a % b
        return a / b
    if isinstance(a, str) and isinstance(b, str) and op == "+":
        return a + b
    x, y = num(a), num(b)
    if x is None or y is None:
        return None
    if op == "+":
        return x + y
    if op == "-":
        return x - y
    if op == "*":
        return x * y
    if y == 0.0:
        return None
    if op == "/":
        return x / y
    if op == "//":
        return float(x // y)
    return x - y * (x // y)


class Ctx:
    __slots__ = ("cnt", "age", "tod", "region", "q")

    def __init__(self, cnt: List[int], age: Optional[int], tod: int, region: int, q: Optional["Search"]):
        self.cnt = cnt
        self.age = age
        self.tod = tod
        self.region = region
        self.q = q


def ev(n: Any, c: Ctx) -> bool:
    if n is True or n is False:
        return n
    t = n[0]
    if t == "h":
        return c.cnt[n[1]] >= n[2]
    if t == "and":
        for x in n[1]:
            if not ev(x, c):
                return False
        return True
    if t == "or":
        for x in n[1]:
            if ev(x, c):
                return True
        return False
    if t == "c":
        return bool(n[1])
    if t == "not":
        return not ev(n[1], c)
    if t == "any":
        return any(c.cnt[i] > 0 for i in n[1])
    if t == "all":
        return all(c.cnt[i] > 0 for i in n[1])
    if t == "ageis":
        return c.age is not None and AGES[c.age] == n[1]
    if t == "tod":
        f = n[1]
        if c.tod != 0:
            return c.tod & f != 0
        if ev(n[2], c):
            return True
        if c.q is None:
            return False
        if c.age is not None:
            return c.q.reach_tod(c.region, c.age, f)
        return any(c.q.reach_tod(c.region, a, f) for a in (0, 1))
    return bool(value(n, c))


def value(n: Any, c: Ctx) -> Any:
    if n is True or n is False:
        return n
    t = n[0]
    if t == "c":
        return n[1]
    if t == "i":
        return c.cnt[n[1]]
    if t == "d":
        return sum(1 for i in n[1] if c.cnt[i] > 0)
    if t == "age":
        return AGES[c.age] if c.age is not None else False
    if t == "neg":
        v = value(n[1], c)
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            return False
        return -v
    if t == "bin":
        r = arith(n[1], value(n[2], c), value(n[3], c))
        return False if r is None else r
    if t == "cmp":
        return compare(n[1], value(n[2], c), value(n[3], c))
    if t == "if":
        return value(n[2], c) if ev(n[1], c) else value(n[3], c)
    if t == "list":
        return [value(x, c) for x in n[1]]
    return ev(n, c)


def deps(n: Any, out: set, tod: List[bool]) -> None:
    if n is True or n is False:
        return
    t = n[0]
    if t in ("h", "i"):
        out.add(n[1])
    elif t in ("any", "all", "d"):
        out.update(n[1])
    elif t in ("and", "or", "list"):
        for x in n[1]:
            deps(x, out, tod)
    elif t == "tod":
        tod[0] = True
        deps(n[2], out, tod)
    elif t in ("not", "neg"):
        deps(n[1], out, tod)
    elif t in ("cmp", "bin"):
        deps(n[2], out, tod)
        deps(n[3], out, tod)
    elif t == "if":
        deps(n[1], out, tod)
        deps(n[2], out, tod)
        deps(n[3], out, tod)


class World:
    def __init__(self, w: Dict[str, Any]):
        self.items: List[str] = [i["name"] for i in w["items"]]
        self.adv: List[bool] = [bool(i["adv"]) for i in w["items"]]
        self.event: List[bool] = [bool(i["event"]) for i in w["items"]]
        self.item_ix: Dict[str, int] = {}
        for k, n in enumerate(self.items):
            self.item_ix.setdefault(n, k)
        self.region_time: List[int] = [r["time"] for r in w["regions"]]
        self.region_name: List[str] = [r["name"] for r in w["regions"]]
        self.region_dungeon: List[Optional[str]] = [r["dungeon"] for r in w["regions"]]
        self.region_exits: List[List[int]] = [r["exits"] for r in w["regions"]]
        self.region_locs: List[List[int]] = [r["locs"] for r in w["regions"]]
        self.exit_from: List[int] = [x["from"] for x in w["exits"]]
        self.exit_to: List[int] = [x["to"] for x in w["exits"]]
        self.exit_rule: List[Any] = [x["rule"] for x in w["exits"]]
        self.loc_name: List[str] = [l["name"] for l in w["locs"]]
        self.loc_region: List[int] = [l["region"] for l in w["locs"]]
        self.loc_rule: List[Any] = [l["rule"] for l in w["locs"]]
        self.loc_event: List[Optional[int]] = [l["event"] for l in w["locs"]]
        self.loc_ix: Dict[str, int] = {n: k for k, n in enumerate(self.loc_name)}
        self.goal: Any = w["goal"]
        self.start: List[Tuple[int, int]] = [(i, n) for i, n in w["start"]]
        self.root: int = w["root"]
        self.real: List[int] = list(w["real"])
        ne = len(self.exit_rule)
        self.users: List[List[int]] = [[] for _ in self.items]
        self.tod_users: List[int] = []
        rules = self.exit_rule + self.loc_rule
        for s, r in enumerate(rules):
            if r is True or r is False:
                continue
            found: set = set()
            tod = [False]
            deps(r, found, tod)
            for d in found:
                self.users[d].append(s)
            if tod[0]:
                self.tod_users.append(s)
        self.nexits = ne


class Search:
    def __init__(self, w: World, cnt: Optional[List[int]] = None):
        self.w = w
        self.cnt: List[int] = list(cnt) if cnt is not None else [0] * len(w.items)
        n = w.nexits + len(w.loc_name)
        r = len(w.region_name)
        self.visited: List[bool] = [False] * len(w.loc_name)
        self.reached: List[List[bool]] = [[False] * r, [False] * r]
        self.tods: List[List[int]] = [[0] * r, [0] * r]
        self.open: List[List[bool]] = [[False] * n, [False] * n]
        self.queued: List[List[bool]] = [[False] * n, [False] * n]
        self.queue: List[List[int]] = [[], []]
        self.dirty = False
        self.place: Dict[int, int] = {}
        for a in (0, 1):
            self._reach(a, w.root)

    def add(self, i: int, k: int = 1) -> None:
        self.cnt[i] += k
        self.dirty = True
        for s in self.w.users[i]:
            for a in (0, 1):
                self._push(a, s, False)

    def _push(self, a: int, s: int, op: bool) -> None:
        if op:
            self.open[a][s] = True
        if self.open[a][s] and not self.queued[a][s]:
            self.queued[a][s] = True
            self.queue[a].append(s)

    def _wanted(self, l: int) -> bool:
        if self.visited[l]:
            return False
        if self.w.loc_event[l] is not None:
            return True
        i = self.place.get(l)
        return i is not None and self.w.adv[i]

    def _reach(self, a: int, r: int) -> None:
        if self.reached[a][r]:
            return
        self.reached[a][r] = True
        self.dirty = True
        w = self.w
        self.tods[a][r] = w.region_time[r]
        for e in w.region_exits[r]:
            if not self.reached[a][w.exit_to[e]]:
                self._push(a, e, True)
        for l in w.region_locs[r]:
            if self._wanted(l):
                self._push(a, w.nexits + l, True)

    def _test(self, rule: Any, a: int, region: int) -> bool:
        if rule is True or rule is False:
            return rule
        return ev(rule, Ctx(self.cnt, a, 0, region, self))

    def _step(self, a: int, s: int) -> None:
        if not self.open[a][s]:
            return
        w = self.w
        ne = w.nexits
        if s < ne:
            to = w.exit_to[s]
            if self.reached[a][to]:
                self.open[a][s] = False
                return
            if self._test(w.exit_rule[s], a, w.exit_from[s]):
                self.open[a][s] = False
                self._reach(a, to)
            return
        l = s - ne
        if self.visited[l]:
            self.open[a][s] = False
            return
        if not self._test(w.loc_rule[l], a, w.loc_region[l]):
            return
        self.visited[l] = True
        self.open[0][s] = False
        self.open[1][s] = False
        self.dirty = True
        e = w.loc_event[l]
        if e is not None:
            self.add(e, 1)
        else:
            i = self.place.get(l)
            if i is not None:
                self.add(i, 1)

    def _requeue_tod(self) -> bool:
        any_ = False
        for s in self.w.tod_users:
            for a in (0, 1):
                if self.open[a][s] and not self.queued[a][s]:
                    self._push(a, s, False)
                    any_ = True
        return any_

    def settle(self, place: Dict[int, int]) -> None:
        self.place = place
        ne = self.w.nexits
        for l in self.w.real:
            if not self._wanted(l):
                continue
            r = self.w.loc_region[l]
            for a in (0, 1):
                if self.reached[a][r] and not self.open[a][ne + l]:
                    self._push(a, ne + l, True)
        while True:
            moved = True
            while moved:
                moved = False
                for a in (0, 1):
                    q = self.queue[a]
                    if q:
                        s = q.pop()
                        self.queued[a][s] = False
                        self._step(a, s)
                        moved = True
                        break
            if self.dirty and self.w.tod_users:
                self.dirty = False
                if self._requeue_tod():
                    continue
            break

    def won(self) -> bool:
        return ev(self.w.goal, Ctx(self.cnt, None, 0, self.w.root, None))

    def loc_access(self, l: int) -> bool:
        r = self.w.loc_region[l]
        rule = self.w.loc_rule[l]
        return any(self.reached[a][r] and self._test(rule, a, r) for a in (0, 1))

    def reach_tod(self, region: int, a: int, flag: int) -> bool:
        if not self.reached[a][region]:
            return False
        if self.tods[a][region] & flag:
            return True
        return self._expand_tod(region, a, flag)

    def _expand_tod(self, goal: int, a: int, flag: int) -> bool:
        w = self.w
        t = list(self.tods[a])
        q: List[int] = []
        for r, f in enumerate(t):
            if f & flag:
                q.extend(w.region_exits[r])
        k = 0
        hit = False
        while k < len(q):
            x = q[k]
            k += 1
            to = w.exit_to[x]
            if not self.reached[a][to] or flag & ~t[to] == 0:
                continue
            rule = w.exit_rule[x]
            ok = rule if (rule is True or rule is False) else ev(rule, Ctx(self.cnt, a, flag, w.exit_from[x], None))
            if ok:
                t[to] |= flag
                if to == goal:
                    hit = True
                    break
                q.extend(w.region_exits[to])
        self.tods[a] = t
        return hit


def explore(w: World, place: Dict[int, int], extra: Iterable[Tuple[int, int]] = ()) -> Search:
    s = Search(w)
    for i, n in w.start:
        s.cnt[i] += n
    for i, n in extra:
        s.add(i, n)
    s.settle(place)
    return s


class Bundle:
    def __init__(self, raw: Dict[str, Any]):
        if raw.get("format") != "ootrr-ap-bundle":
            raise ValueError("not an OOTRR Archipelago bundle")
        self.raw = raw
        self.preset: str = raw["preset"]
        self.settings: str = raw["settings"]
        self.seed: str = raw["seed"]
        self.world = World(raw["world"])
        w = self.world
        self.free: List[int] = [r["loc"] for r in raw["free"]]
        self.free_names: List[str] = [w.loc_name[l] for l in self.free]
        self.row: Dict[str, Dict[str, Any]] = {w.loc_name[r["loc"]]: r for r in raw["free"]}
        self.deny: Dict[str, frozenset] = {w.loc_name[r["loc"]]: frozenset(w.items[i] for i in r["deny"]) for r in raw["free"]}
        self.pool: List[str] = [w.items[i] for i in raw["pool"]]
        self.meta: Dict[str, Dict[str, Any]] = {w.items[x["id"]]: x for x in raw["pool_items"]}
        self.fixed: Dict[int, int] = {l: i for l, i in raw["fixed"]}
        self.forced: List[Tuple[str, str]] = [(w.loc_name[l], w.items[i]) for l, i in raw["forced"]]
        self.adv_names: frozenset = frozenset(n for n, a, e in zip(w.items, w.adv, w.event) if a and not e)

    def sweep(self, held: Dict[str, int]) -> "Sweep":
        w = self.world
        extra = [(w.item_ix[n], k) for n, k in held.items() if k > 0 and n in w.item_ix]
        s = explore(w, self.fixed, extra)
        return Sweep(frozenset(w.loc_name[l] for l in self.free if s.loc_access(l)), s.won())


class Sweep:
    __slots__ = ("reach", "won")

    def __init__(self, reach: frozenset, won: bool):
        self.reach = reach
        self.won = won


def bundle_names() -> List[str]:
    return [p["id"] for p in load_json("presets.json")]


def here() -> str:
    return os.path.dirname(__file__)
