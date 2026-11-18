//! Lifts a pickup's item model off the ground.
//!
//! A pickup rests on its physics body, and the actor's origin sits as far above the ground as the
//! body reaches below it: the base pickup's sphere starts at the origin, the token's capsule 0.2 m
//! below. Item models are centred on their origin (Link holds them there), so they sank by as much
//! as they reach below it, and the turn (actor::SPIN) also tips them over. The pickup gets its own
//! copy of the body's shapes moved down until the body reaches as far below the origin as the
//! model reaches in any direction ([`of_row`], capped by [`CAP`]), with copies of the controller
//! set and the physics settings that point at them.

use crate::byml::{Byml, Node};
use crate::error::{Error as E, Res};
use crate::pickup_pack::actor::Made;
use crate::pickup_pack::closure::{chain, local, norm, work, Pool, COMPONENTS, PARENT};
use crate::pickup_pack::{Layers, BASE};
use crate::romfs::Romfs;

pub const PHYSICS_REF: &str = "PhysicsRef";
/// The highest lift, in millimetres: the Megaton Hammer reaches 1.45 m along its handle.
pub const CAP: u32 = 600;
const SET_PATH: &str = "ControllerSetPath";
const SHAPES: &str = "ShapeNamePathAry";
const BODIES: &str = "RigidBodyEntityNamePathAry";
const SHAPE_NAME: &str = "ShapeName";
const FILE_PATH: &str = "FilePath";
const NAME: &str = "Name";
const POINTS: [&str; 5] = ["Center", "CenterA", "CenterB", "Min", "Max"];

/// The lift for a model, in millimetres, from its actor row's ModelAabbMin and ModelAabbMax: the
/// farthest the model reaches from its origin along any axis, up to [`CAP`].
pub fn of_row(row: &Node) -> Option<u32> {
    let mut far: f32 = 0.0;
    for k in ["ModelAabbMin", "ModelAabbMax"] {
        let v: &Node = row.get(k)?;
        for a in ["X", "Y", "Z"] {
            far = far.max(num(v.get(a)?)?.abs());
        }
    }
    Some(((far * 1000.0).round() as u32).min(CAP))
}

fn num(n: &Node) -> Option<f32> {
    match n {
        Node::F32(x) => Some(*x),
        Node::F64(x) => Some(*x as f32),
        Node::I32(x) => Some(*x as f32),
        _ => None,
    }
}

/// `l` with the lift filled in for a spin layer: from the actor row of the model the pickup shows
/// (the model layer's donor, else the item).
pub fn layers(rf: &mut Romfs, item: &str, l: &Layers) -> Res<Layers> {
    if !l.spin || l.lift != 0 {
        return Ok(l.clone());
    }
    let shown: &str = l.model.as_deref().unwrap_or(item);
    let t: Byml = rf.read_byml(crate::pickup_pack::rows::ACTOR_INFO)?;
    let mm: u32 = crate::pickup_pack::rows::find(&t, shown).and_then(of_row).unwrap_or(0);
    Ok(Layers { lift: mm, ..l.clone() })
}

pub fn physics_file(name: &str) -> String {
    format!("Component/Physics/{name}.engine__component__PhysicsParam.bgyml")
}

pub fn set_file(name: &str) -> String {
    format!("Phive/ControllerSetParam/{name}.phive__ControllerSetParam.bgyml")
}

pub fn shape_file(name: &str, k: usize) -> String {
    format!("Phive/ShapeParam/{name}_Lift_{k}.phive__ShapeParam.bgyml")
}

fn text<'a>(n: &'a Node, k: &str) -> Option<&'a str> {
    n.get(k).and_then(|v| v.as_str())
}

fn file(p: &Pool, work_path: &str, what: &str) -> Res<(String, Byml)> {
    let f: String = norm(work_path).ok_or_else(|| E::Msg(format!("lift: the {what} reference {work_path} is not a file")))?;
    if p.get(&f).is_none() {
        return Err(E::Msg(format!("lift: the {what} {f} is not in the pickup's packs")));
    }
    let y: Byml = p.byml(&f)?;
    Ok((f, y))
}

/// Moves every point of a shape down by `dy` metres.
fn shift(n: &mut Node, dy: f32) {
    match n {
        Node::Map(m) => {
            for (k, v) in m.iter_mut() {
                if POINTS.contains(&k.as_str()) {
                    if let Some(Node::F32(y)) = v.get_mut("Y") {
                        *y -= dy;
                        continue;
                    }
                }
                shift(v, dy);
            }
        }
        Node::Arr(a) => a.iter_mut().for_each(|x| shift(x, dy)),
        _ => {}
    }
}

fn bottom(shape: &Node) -> Option<f32> {
    shape.get("AutoCalc").and_then(|a| a.get("Min")).and_then(|m| m.get("Y")).and_then(num)
}

/// Gives the pickup `m` (actor `name`) a body that reaches `mm` millimetres below its origin, built
/// from the body its actor uses (its own PhysicsRef, else the pickup base's). Nothing changes when
/// the body already reaches that far.
pub fn apply(p: &Pool, mut m: Made, name: &str, mm: u32) -> Res<Made> {
    if mm == 0 {
        return Ok(m);
    }
    let mut actor: Byml = Byml::parse(&m.actor.1)?;
    let own: Option<String> = actor.root.get(COMPONENTS).and_then(|c| text(c, PHYSICS_REF)).map(|s| s.to_string());
    let base_ref: String = match own {
        Some(r) => r,
        None => chain(p, BASE)?.comps.get(PHYSICS_REF).cloned().ok_or(E::Format("lift: the pickup base has no physics"))?,
    };
    let (phys_path, phys): (String, Byml) = file(p, &base_ref, "physics")?;
    let set_ref: String = text(&phys.root, SET_PATH).map(|s| s.to_string()).ok_or_else(|| E::Msg(format!("lift: {phys_path} names no controller set")))?;
    let (_, mut set): (String, Byml) = file(p, &set_ref, "controller set")?;
    let body_ref: String = set.root.get(BODIES).and_then(|a| a.as_arr()).and_then(|a| a.first()).and_then(|b| text(b, FILE_PATH)).map(|s| s.to_string()).ok_or(E::Format("lift: the controller set has no rigid body"))?;
    let (_, body): (String, Byml) = file(p, &body_ref, "rigid body")?;
    let body_shape: String = text(&body.root, SHAPE_NAME).map(|s| s.to_string()).ok_or(E::Format("lift: the rigid body names no shape"))?;
    let shapes: Vec<(String, String)> = set
        .root
        .get(SHAPES)
        .and_then(|a| a.as_arr())
        .map(|a| a.iter().filter_map(|s| Some((text(s, NAME)?.to_string(), text(s, FILE_PATH)?.to_string()))).collect())
        .unwrap_or_default();
    let (_, main): (String, Byml) = file(p, &shapes.iter().find(|s| s.0 == body_shape).map(|s| s.1.clone()).ok_or_else(|| E::Msg(format!("lift: the controller set has no shape {body_shape}")))?, "body shape")?;
    let low: f32 = bottom(&main.root).ok_or(E::Format("lift: the body shape has no bounds"))?;
    let dy: f32 = mm as f32 / 1000.0 + low;
    if dy <= 0.0 {
        return Ok(m);
    }
    let mut paths: Vec<String> = Vec::new();
    for (k, (_, path)) in shapes.iter().enumerate() {
        let (_, mut y): (String, Byml) = file(p, path, "shape")?;
        if y.root.get("Polytope").and_then(|a| a.as_arr()).is_some_and(|a| !a.is_empty()) {
            return Err(E::Format("lift: a body made of polytopes cannot be moved"));
        }
        shift(&mut y.root, dy);
        let f: String = shape_file(name, k);
        m.extra.push((f.clone(), y.write()?));
        paths.push(work(&f));
    }
    if let Some(a) = set.root.get_mut(SHAPES).and_then(|a| a.as_arr_mut()) {
        for (s, w) in a.iter_mut().zip(&paths) {
            s.set(FILE_PATH, Node::Str(w.clone()));
        }
    }
    let sf: String = set_file(name);
    m.extra.push((sf.clone(), set.write()?));
    let mut ph: Node = Node::Map(Vec::new());
    ph.set(PARENT, Node::Str(work(&phys_path)));
    ph.set(SET_PATH, Node::Str(work(&sf)));
    let pf: String = physics_file(name);
    m.extra.push((pf.clone(), Byml { root: ph, version: phys.version, big: phys.big }.write()?));
    actor.root.get_mut(COMPONENTS).ok_or(E::Format("lift: the actor has no component list"))?.set(PHYSICS_REF, Node::Str(local(&pf)));
    m.actor.1 = actor.write()?;
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, z: f32) -> Node {
        Node::Map(vec![("X".into(), Node::F32(x)), ("Y".into(), Node::F32(y)), ("Z".into(), Node::F32(z))])
    }

    #[test]
    fn the_lift_is_the_farthest_reach_of_the_model_up_to_the_cap() {
        let row = |a: Node, b: Node| Node::Map(vec![("ModelAabbMin".into(), a), ("ModelAabbMax".into(), b)]);
        assert_eq!(of_row(&row(v(-0.10858154, -0.039733887, -0.120462894), v(0.10882568, 0.03945923, 0.5272422))), Some(527));
        assert_eq!(of_row(&row(v(-0.44, -0.27, -0.32), v(0.21, 0.27, 1.45))), Some(CAP));
        assert_eq!(of_row(&Node::Map(Vec::new())), None);
    }

    #[test]
    fn a_shape_moves_every_point_down() {
        let mut s: Node = Node::Map(vec![
            ("AutoCalc".into(), Node::Map(vec![("Center".into(), v(0.0, 0.2, 0.0)), ("Min".into(), v(-0.2, 0.0, -0.2)), ("Max".into(), v(0.2, 0.4, 0.2))])),
            ("Sphere".into(), Node::Arr(vec![Node::Map(vec![("Center".into(), v(0.0, 0.2, 0.0)), ("Radius".into(), Node::F32(0.2))])])),
        ]);
        shift(&mut s, 0.5);
        assert_eq!(bottom(&s), Some(-0.5));
        let c = s.get("Sphere").and_then(|a| a.as_arr()).and_then(|a| a.first()).and_then(|x| x.get("Center")).and_then(|c| c.get("Y")).cloned();
        assert_eq!(c, Some(Node::F32(-0.3)));
        assert_eq!(s.get("Sphere").and_then(|a| a.as_arr()).and_then(|a| a.first()).and_then(|x| x.get("Radius")).cloned(), Some(Node::F32(0.2)));
    }
}
