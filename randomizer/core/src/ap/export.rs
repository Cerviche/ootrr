use std::collections::BTreeSet as BS;

use crate::error::{Error as E, Res};
use crate::json::Json;
use crate::logic_items::ItemId;
use crate::logic_val::Val;
use crate::rule_ast::{Bop, Cmp};
use crate::rules::{Node, Rule};
use crate::settings::Settings;
use crate::slice::data::Data;
use crate::slice::generate::{allowed, prepare, Prep};
use crate::slice::pick::scoped;
use crate::slice::support::{Plan, Songs};
use crate::rng::Rng;
use crate::search::{explore, Search};
use crate::world::{LocId, Placement, World};

pub const FORMAT: &str = "ootrr-ap-bundle";
pub const VERSION: i64 = 1;
pub const GAME: &str = "OOTRR";
pub const SEED: &str = "archipelago";

fn int(n: usize) -> Json {
    Json::Int(n as i64)
}

fn ids(v: &[ItemId]) -> Json {
    Json::Arr(v.iter().map(|i| int(*i)).collect())
}

fn val(v: &Val) -> Json {
    match v {
        Val::Bool(b) => Json::Bool(*b),
        Val::Int(n) => Json::Int(*n),
        Val::Float(f) => Json::Num(*f),
        Val::Str(s) => Json::Str(s.clone()),
        Val::List(l) => Json::Arr(l.iter().map(val).collect()),
        Val::Map(m) => Json::Obj(m.iter().map(|(k, x)| (k.clone(), val(x))).collect()),
    }
}

fn tag(t: &str, rest: Vec<Json>) -> Json {
    let mut v: Vec<Json> = vec![Json::Str(t.to_string())];
    v.extend(rest);
    Json::Arr(v)
}

fn cmp(c: Cmp) -> &'static str {
    match c {
        Cmp::Eq => "==",
        Cmp::Ne => "!=",
        Cmp::Lt => "<",
        Cmp::Le => "<=",
        Cmp::Gt => ">",
        Cmp::Ge => ">=",
        Cmp::In => "in",
        Cmp::NotIn => "not in",
    }
}

fn bop(b: Bop) -> &'static str {
    match b {
        Bop::Add => "+",
        Bop::Sub => "-",
        Bop::Mul => "*",
        Bop::Div => "/",
        Bop::Floor => "//",
        Bop::Rem => "%",
    }
}

pub fn node(n: &Node) -> Json {
    match n {
        Node::Const(v) => tag("c", vec![val(v)]),
        Node::Has(i, k) => tag("h", vec![int(*i), Json::Int(*k as i64)]),
        Node::Any(v) => tag("any", vec![ids(v)]),
        Node::All(v) => tag("all", vec![ids(v)]),
        Node::Item(i) => tag("i", vec![int(*i)]),
        Node::Distinct(v) => tag("d", vec![ids(v)]),
        Node::Age => tag("age", vec![]),
        Node::AgeIs(a) => tag("ageis", vec![Json::Str(a.name().to_string())]),
        Node::Tod(f, skip) => tag("tod", vec![Json::Int(*f as i64), node(skip)]),
        Node::Not(x) => tag("not", vec![node(x)]),
        Node::Neg(x) => tag("neg", vec![node(x)]),
        Node::And(v) => tag("and", vec![Json::Arr(v.iter().map(node).collect())]),
        Node::Or(v) => tag("or", vec![Json::Arr(v.iter().map(node).collect())]),
        Node::Cmp(c, a, b) => tag("cmp", vec![Json::Str(cmp(*c).to_string()), node(a), node(b)]),
        Node::Bin(o, a, b) => tag("bin", vec![Json::Str(bop(*o).to_string()), node(a), node(b)]),
        Node::If(k, t, e) => tag("if", vec![node(k), node(t), node(e)]),
        Node::List(v) => tag("list", vec![Json::Arr(v.iter().map(node).collect())]),
    }
}

fn rule(r: &Rule) -> Json {
    if r.always {
        return Json::Bool(true);
    }
    if r.never {
        return Json::Bool(false);
    }
    node(&r.node)
}

fn opt(s: &Option<String>) -> Json {
    match s {
        Some(x) => Json::Str(x.clone()),
        None => Json::Null,
    }
}

pub fn world_json(w: &World) -> Json {
    let items: Vec<Json> = (0..w.items.len())
        .map(|i| Json::obj().with("name", Json::Str(w.items.names[i].clone())).with("adv", Json::Bool(w.items.adv[i])).with("event", Json::Bool(w.items.event[i])))
        .collect();
    let regions: Vec<Json> = w
        .regions
        .iter()
        .map(|r| {
            Json::obj()
                .with("name", Json::Str(r.name.clone()))
                .with("time", Json::Int(r.provides_time as i64))
                .with("dungeon", opt(&r.dungeon))
                .with("scene", opt(&r.scene))
                .with("exits", Json::Arr(r.exits.iter().map(|x| int(*x)).collect()))
                .with("locs", Json::Arr(r.locations.iter().map(|x| int(*x)).collect()))
        })
        .collect();
    let exits: Vec<Json> = w.exits.iter().map(|x| Json::obj().with("name", Json::Str(x.name.clone())).with("from", int(x.from)).with("to", int(x.to)).with("rule", rule(&x.rule))).collect();
    let locs: Vec<Json> = w
        .locs
        .iter()
        .map(|l| {
            Json::obj()
                .with("name", Json::Str(l.name.clone()))
                .with("region", int(l.region))
                .with("rule", rule(&l.rule))
                .with("event", l.event.map(int).unwrap_or(Json::Null))
        })
        .collect();
    let tod: BS<u32> = w.tod_users.iter().copied().collect();
    Json::obj()
        .with("items", Json::Arr(items))
        .with("regions", Json::Arr(regions))
        .with("exits", Json::Arr(exits))
        .with("locs", Json::Arr(locs))
        .with("goal", rule(&w.goal))
        .with("start", Json::Arr(w.start.iter().map(|(i, n)| Json::Arr(vec![int(*i), Json::Int(*n as i64)])).collect()))
        .with("root", int(w.root))
        .with("real", Json::Arr(w.real.iter().map(|l| int(*l)).collect()))
        .with("tod_users", Json::Arr(tod.iter().map(|s| Json::Int(*s as i64)).collect()))
}

fn local_only(d: &Data, p: &Plan, name: &str) -> bool {
    let Some(i) = d.item(name) else { return false };
    if scoped(p, &i.kind) || crate::slice::dungeon_items::handles(p, &i.kind) {
        return true;
    }
    if p.keys_roam() && i.kind == "SmallKey" {
        return true;
    }
    p.songs == Songs::Own && i.actor.as_deref().is_some_and(crate::slice::generate::is_song)
}

pub fn foreign_ok(d: &Data, name: &str) -> bool {
    d.cat.by_name(name).is_some_and(|c| c.kind == "chest") || d.flow.by_name(name).is_some()
}

fn kind_of(d: &Data, name: &str) -> String {
    if let Some(c) = d.cat.by_name(name) {
        return c.kind.clone();
    }
    if d.flow.by_name(name).is_some() {
        return "npc".to_string();
    }
    if crate::shops::is_loc(name) {
        return "shop".to_string();
    }
    "other".to_string()
}

pub fn export(d0: &Data, s: &Settings, preset: &str) -> Res<Json> {
    Ok(export_with(d0, s, preset, 0)?.0)
}

fn probes(w: &World, free: &[LocId], fixed: &Placement, pool: &[ItemId], n: usize) -> Json {
    let mut rng: Rng = Rng::from_text("ootrr archipelago probes");
    let mut out: Vec<Json> = Vec::with_capacity(n);
    for k in 0..n {
        let keep: u64 = if k + 1 == n { 100 } else { (k as u64 * 100) / n.max(1) as u64 };
        let held: Vec<ItemId> = pool.iter().copied().filter(|_| rng.below(100) < keep).collect();
        let s: Search = explore(w, fixed, &held);
        let reach: Vec<Json> = free.iter().filter(|l| s.loc_access(**l)).map(|l| int(*l)).collect();
        out.push(Json::obj().with("held", ids(&held)).with("reach", Json::Arr(reach)).with("won", Json::Bool(s.won())));
    }
    Json::Arr(out)
}

pub fn export_with(d0: &Data, s: &Settings, preset: &str, n: usize) -> Res<(Json, Json)> {
    let Prep { p, lay: _, owned, w, vanilla, pk, gates, base } = prepare(d0, s, SEED, &[])?;
    let d: &Data = owned.as_ref().unwrap_or(d0);
    if gates.is_some() {
        return Err(E::Format("archipelago: entrance shuffle is not supported yet"));
    }
    let warnings: Vec<String> = Vec::new();
    let lacs: Vec<(LocId, ItemId)> = crate::lacs_key::forced(&w, &p, &pk.free, &base, &[])?;
    let free: Vec<LocId> = pk.free.clone();
    let mut pool: Vec<ItemId> = Vec::with_capacity(free.len());
    for l in &free {
        pool.push(base.get(*l).ok_or_else(|| E::Msg(format!("archipelago: the shuffled location {} holds no pool item", w.locs[*l].name)))?);
    }
    let kinds: BS<ItemId> = pool.iter().copied().collect();
    let kinds: Vec<ItemId> = kinds.into_iter().collect();
    let mut rows: Vec<Json> = Vec::with_capacity(free.len());
    let mut loose: usize = 0;
    for l in &free {
        let name: &str = &w.locs[*l].name;
        let tier: bool = crate::slice::tier_order::fixed_spot(d, name);
        let deny: Vec<ItemId> = kinds.iter().copied().filter(|i| !allowed(d, &w, &p, *i, *l) || (tier && d.family(&w.items.names[*i]).is_some())).collect();
        if deny.is_empty() {
            loose += 1;
        }
        let here: Option<&String> = w.regions[w.locs[*l].region].dungeon.as_ref();
        rows.push(
            Json::obj()
                .with("loc", int(*l))
                .with("kind", Json::Str(kind_of(d, name)))
                .with("dungeon", opt(&here.cloned()))
                .with("vanilla", vanilla.get(*l).map(|i| Json::Str(w.items.names[i].clone())).unwrap_or(Json::Null))
                .with("foreign", Json::Bool(foreign_ok(d, name)))
                .with("deny", ids(&deny)),
        );
    }
    let mut fixed_place: Placement = Placement::new(w.locs.len());
    for l in w.real.iter().filter(|l| !free.contains(l)) {
        if let Some(i) = vanilla.get(*l) {
            fixed_place.put(*l, i);
        }
    }
    let checks: Json = probes(&w, &free, &fixed_place, &pool, n);
    let fixed: Vec<Json> = w
        .real
        .iter()
        .filter(|l| !free.contains(l))
        .filter_map(|l| vanilla.get(*l).map(|i| Json::Arr(vec![int(*l), int(i)])))
        .collect();
    let items: Vec<Json> = kinds
        .iter()
        .map(|i| {
            let name: &str = &w.items.names[*i];
            let it = d.item(name);
            Json::obj()
                .with("id", int(*i))
                .with("kind", Json::Str(it.map(|x| x.kind.clone()).unwrap_or_default()))
                .with("dungeon", opt(&it.and_then(|x| x.dungeon.clone())))
                .with("junk", Json::Bool(crate::slice::place_rules::junk(d, name)))
                .with("local", Json::Bool(local_only(d, &p, name)))
        })
        .collect();
    Ok((Json::obj()
        .with("format", Json::Str(FORMAT.to_string()))
        .with("version", Json::Int(VERSION))
        .with("game", Json::Str(GAME.to_string()))
        .with("preset", Json::Str(preset.to_string()))
        .with("settings", Json::Str(s.encode()?))
        .with("seed", Json::Str(SEED.to_string()))
        .with("world", world_json(&w))
        .with("free", Json::Arr(rows))
        .with("pool", ids(&pool))
        .with("pool_items", Json::Arr(items))
        .with("fixed", Json::Arr(fixed))
        .with("forced", Json::Arr(lacs.iter().map(|(l, i)| Json::Arr(vec![int(*l), int(*i)])).collect()))
        .with("loose", int(loose))
        .with("warnings", Json::strs(warnings.iter().map(|x| x.as_str())))
        .with("notes", Json::strs(p.notes.iter().map(|x| x.as_str()))), checks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slice::generate::{generate_data, GenOpts};

    fn arr<'a>(j: &'a Json, k: &str) -> &'a [Json] {
        j.get(k).and_then(|v| v.as_arr()).unwrap()
    }

    #[test]
    fn the_default_bundle_matches_what_the_generator_shuffles() {
        let d: Data = Data::load().unwrap();
        let s: Settings = Settings::default();
        let b: Json = export(&d, &s, "default").unwrap();
        let w: &Json = b.get("world").unwrap();
        let locs: &[Json] = arr(w, "locs");
        let names: Vec<String> = arr(&b, "free").iter().map(|r| locs[r.get("loc").unwrap().as_i64().unwrap() as usize].get("name").unwrap().as_str().unwrap().to_string()).collect();
        let g = generate_data(&d, &s, SEED, &GenOpts::default()).unwrap();
        let mut want: Vec<String> = g.seed_file.placements.keys().cloned().collect();
        let mut got: Vec<String> = names.clone();
        want.sort();
        got.sort();
        assert_eq!(got, want);
        let items: &[Json] = arr(w, "items");
        let mut pool: Vec<String> = arr(&b, "pool").iter().map(|i| items[i.as_i64().unwrap() as usize].get("name").unwrap().as_str().unwrap().to_string()).collect();
        let mut placed: Vec<String> = g.seed_file.placements.values().cloned().collect();
        pool.sort();
        placed.sort();
        assert_eq!(pool, placed);
    }
}
