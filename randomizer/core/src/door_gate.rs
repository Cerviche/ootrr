use std::path::PathBuf as PB;

use crate::ainb::Ainb;
use crate::bfevfl::flow::NONE;
use crate::bfevfl::{Bf, Case, Cont, Event, Flow, Kind, Val};
use crate::byml::{Byml, Node};
use crate::error::{Error, Res};
use crate::romfs::Romfs;
use crate::settings::{Setting, Settings, Value};
use crate::staging::{self, Out};
use crate::start_defaults::Edit;

pub const SETTING: &str = "door_of_time_condition";
pub const REWARDS: &str = "door_of_time_rewards";
pub const SONG: &str = "door_of_time_song";
pub const STONES: &str = "stones";
pub const DUNGEONS: &str = "dungeons";
pub const OPEN: &str = "open";
pub const SONG_ONLY: &str = "song";
pub const ESCAPE_EXP: &str = "DoorOfTime_IsAfter_Step2_1_Exp";
pub const REQ_EXP: &str = "DoorOfTime_IsAfter_Srep2_1_1_Exp";
pub const SENT: &str = "HavePlayedEvent.HyruleField_DoorOfTime_SendOff";
pub const ESCAPED: &str = "HavePlayedEvent.HyruleField_DoorOfTime_Escape";
pub const MODULE: &str = "Logic/HyruleField_Child_Prefab_0ce5.logic.module.ainb";
pub const AREA: &str = "Area_054e";
pub const AREA_FILE: &str = "Banc/HyruleField/HyruleField_Child_Static.bcett.byml.zs";
pub const AREA_HASH: u64 = 5374506228817442823;
pub const ESCAPE_STEP: &str = "Step03";
pub const OPEN_STEP: &str = "Step03";
pub const SONG_CHECK: &str = "Event60";
pub const SONG_CHECK_EXP: &str = "DoorOfTime_IsAfter_Step04_Exp";
pub const OPENED: &str = "Market_Child_TimeTempleDoor_Opened";
pub const REACTS: &str = "Market_Child_TimeTempleDoor_OcarinaReacts";
pub const PREFIX: &str = "OOTRRDoorGate";
pub const NOTE: &str = "The Door of Time opens when the Song of Time is played at the altar while the Door of Time requirement is met, and Zelda's escape from the castle waits for the same requirement in place of Dodongo's Cavern and Jabu-Jabu's Belly. The altar's song check reads the rewritten computed flag DoorOfTime_IsAfter_Srep2_1_1_Exp, the escape area in Hyrule Field appears while DoorOfTime_IsAfter_Step2_1_Exp holds (Impa's send off played, the escape not played yet, and the requirement met), clearing Dodongo's Cavern and Jabu-Jabu's Belly no longer moves the quest, and opening the door early leaves the escape, the Ocarina of Time and the Song of Time as checks. Experimental and untested in the game.";
pub const SONGLESS_NOTE: &str = "The Door of Time opens once Zelda's escape has played, without the Song of Time. The door's logic module opens it once the Door of Time quest is past Step03 in place of Step06, and the Master Sword can be drawn without the quest. Experimental and untested in the game.";
pub const VANILLA_NOTE: &str = "The Door of Time opens when the Song of Time is played at the altar once Dodongo's Cavern and Jabu-Jabu's Belly are cleared, without waiting for Zelda's escape. The altar's song check reads the two dungeons' quests (Get_StoneOfFire_IsCompleted_Exp and Get_StoneOfWater_IsCompleted_Exp), and opening the door early leaves the escape, the Ocarina of Time and the Song of Time as checks.";
pub const FIRE_EXP: &str = "Get_StoneOfFire_IsCompleted_Exp";
pub const WATER_EXP: &str = "Get_StoneOfWater_IsCompleted_Exp";
pub const SONG_ONLY_NOTE: &str = "The Door of Time opens when the Song of Time is played at the altar, with no other requirement. The opening scene no longer waits for Zelda's escape and only moves the Door of Time quest to Step06 when the quest is already past Step04, so Zelda's escape, the Ocarina of Time and the Song of Time stay checks. The door's logic module keeps it open from its opened flag, and the Master Sword can be drawn without the quest. Experimental and untested in the game.";
const COUNTS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const AREA_EL: &str = "ActorLogicAreaActor";
const CREATE: &str = "Logic_Create";
const STEP_IN: &str = "ChallengeStepName";
const CHALLENGE_IN: &str = "ChallengeName";
const WAIT_ACTOR: &str = "EventSystemActor";
const WAIT: &str = "EventWait";
pub const RIVER_MODULE: &str = "Logic/HyruleField_Child_Prefab_32f7.logic.module.ainb";
pub const RIVER_REQUEST: &str = "EventRequestActor_b3d3";
pub const RIVER_WALL: &str = "FldObj_AirWall_Player_GenerateNavMesh_066c";
const REQUEST: &str = "ActorLogicEventRequestActor";
const SIMPLE: &str = "ActorLogicSimpleActor";
const FLAG_GET: &str = "HoldGameDataGetBoolSimple";
const FLAG_IN: &str = "GameDataNameBool";
const PROXY: &str = "ActorLogicEventProxyRequestActor";
const HOLDER: &str = "GameDataActor";
const IS_EXP: &str = "EventQueryIsGameDataBoolExp";
const HEAD_EXP: &str = "Get_StoneOfFire_IsCompleted_Exp";
const STR: usize = 3;
const BOOL: usize = 1;
const OUT_BOOL: usize = 4;
const IS_EQUAL: u16 = 2;
const AND_OP: i32 = 3;
const OR_OP: i32 = 4;
const NOT_OP: i32 = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Need {
    #[default]
    Vanilla,
    Stones,
    Rewards(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opts {
    pub need: Need,
    pub song: bool,
    pub only: bool,
    pub open: bool,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts { need: Need::Vanilla, song: true, only: false, open: false }
    }
}

impl Opts {
    pub fn escape(&self) -> bool {
        self.need != Need::Vanilla
    }

    pub fn flow(&self) -> bool {
        !self.open
    }

    pub fn exps(&self) -> Vec<Edit> {
        let mut out: Vec<Edit> = Vec::new();
        if !self.open {
            out.push(Edit::Exp(REACTS.to_string(), vec![(0, OPENED.to_string())]));
        }
        if self.escape() {
            out.push(Edit::Exp(ESCAPE_EXP.to_string(), terms(self.need)));
            out.push(Edit::Exp(REQ_EXP.to_string(), req_terms(self.need)));
        }
        out
    }

    pub fn logic(&self) -> Vec<(&'static str, &'static str)> {
        let mut out: Vec<(&'static str, &'static str)> = match self.need {
            Need::Vanilla => Vec::new(),
            Need::Stones => vec![(SETTING, STONES)],
            Need::Rewards(n) => vec![(SETTING, DUNGEONS), (REWARDS, COUNTS[usize::from(n).min(9)])],
        };
        if self.only {
            out.push((crate::open_world::DOOR, "sot"));
        } else if !self.song {
            out.push((crate::open_world::DOOR, "stones"));
        }
        out
    }

    pub fn notes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if self.escape() {
            out.push(NOTE.to_string());
        }
        if !self.open && !self.escape() && !self.only {
            out.push(VANILLA_NOTE.to_string());
        }
        if self.only {
            out.push(SONG_ONLY_NOTE.to_string());
        }
        if !self.song {
            out.push(SONGLESS_NOTE.to_string());
        }
        out
    }
}

pub fn settings() -> Vec<Setting> {
    vec![
        Setting::choice(109, SETTING, "world", "Door of Time", "vanilla")
            .tip("What it takes to open the Door of Time. With a Closed choice, play the Song of Time at the altar once its requirement is met. The requirement also starts Zelda's escape from the castle, which leads to the Ocarina of Time and the Song of Time checks.")
            .opt("vanilla", "Closed - Vanilla", "Clear Dodongo's Cavern and Jabu-Jabu's Belly, as in the remake.")
            .opt(STONES, "Closed - Classic", "Hold all three Spiritual Stones, as in the original game. Experimental, untested in the game.")
            .opt(DUNGEONS, "Closed - Dungeon Rewards", "Hold the set number of Stones and Medallions together. Experimental, untested in the game.")
            .opt(SONG_ONLY, "Closed - Song Only", "Play the Song of Time at the altar in the Temple of Time to open the door, with no other requirement. Experimental, untested in the game.")
            .opt(OPEN, "Open", "The Door of Time is open from the start. Zelda's escape and the Song of Time stay checks."),
        Setting::number(110, REWARDS, "world", "Rewards for the Door of Time", 3, 0, 9)
            .tip("How many Stones and Medallions together start Zelda's escape. Any mix of the three Spiritual Stones and six Medallions counts.")
            .when(SETTING, DUNGEONS),
        Setting::toggle(111, SONG, "world", "Additionally requires Song of Time", true)
            .tip("On, the Song of Time must be played at the altar once the requirement is met. Off, the door also opens by itself once Zelda's escape has played. Off is experimental and untested in the game.")
            .when_not(SETTING, OPEN)
            .when_not(SETTING, SONG_ONLY),
    ]
}

pub fn read(s: &Settings, open: bool, bad: &mut Vec<String>) -> Opts {
    let need: Need = match s.text(SETTING) {
        Ok("vanilla") | Ok(OPEN) | Ok(SONG_ONLY) => Need::Vanilla,
        Ok(STONES) => Need::Stones,
        Ok(DUNGEONS) => match s.int(REWARDS) {
            Ok(n) if (0..=9).contains(&n) => Need::Rewards(n as u8),
            Ok(n) => {
                bad.push(format!("{REWARDS} is {n}, which is not between 0 and 9"));
                Need::Vanilla
            }
            Err(e) => {
                bad.push(e.to_string());
                Need::Vanilla
            }
        },
        Ok(v) => {
            bad.push(format!("{SETTING} is {v}, which is not a known choice"));
            Need::Vanilla
        }
        Err(e) => {
            bad.push(e.to_string());
            Need::Vanilla
        }
    };
    let only: bool = !open && s.text(SETTING).is_ok_and(|v| v == SONG_ONLY);
    let song: bool = open || only || crate::slice::support::toggle(s, SONG, bad);
    Opts { need, song, only, open }
}

pub fn legacy(v: &mut crate::settings::state::Vals) {
    if v.get(crate::open_world::DOOR) == Some(&Value::Bool(true)) {
        v.insert(crate::open_world::DOOR.to_string(), Value::Bool(false));
        v.insert(SETTING.to_string(), Value::s(OPEN));
    }
}

pub fn req_terms(n: Need) -> Vec<(i32, String)> {
    let held: Option<Vec<(i32, String)>> = match n {
        Need::Vanilla => None,
        Need::Stones => crate::medal_gate::at_least(&crate::medal_gate::REWARD_ITEMS[..3], 3),
        Need::Rewards(k) => crate::medal_gate::at_least(&crate::medal_gate::REWARD_ITEMS, usize::from(k).min(9)),
    };
    held.unwrap_or_else(|| vec![(0, SENT.to_string()), (0, SENT.to_string()), (NOT_OP, String::new()), (OR_OP, String::new())])
}

pub fn terms(n: Need) -> Vec<(i32, String)> {
    let mut t: Vec<(i32, String)> = vec![(0, SENT.to_string()), (0, ESCAPED.to_string()), (NOT_OP, String::new()), (AND_OP, String::new())];
    let held: Option<Vec<(i32, String)>> = match n {
        Need::Vanilla => None,
        Need::Stones => crate::medal_gate::at_least(&crate::medal_gate::REWARD_ITEMS[..3], 3),
        Need::Rewards(k) => crate::medal_gate::at_least(&crate::medal_gate::REWARD_ITEMS, usize::from(k).min(9)),
    };
    if let Some(h) = held {
        t.extend(h);
        t.push((AND_OP, String::new()));
    }
    t
}

fn one(hits: Vec<usize>, what: &str) -> Res<usize> {
    match hits.as_slice() {
        [i] => Ok(*i),
        _ => Err(Error::Msg(format!("door gate: expected one {what}, found {}, the module is not the unpatched one", hits.len()))),
    }
}

fn step_check(a: &Ainb, step: &str) -> Res<usize> {
    one(
        a.find_els(crate::door_open::STEP_CHECK)
            .into_iter()
            .filter(|&i| a.in_str(i, CHALLENGE_IN) == Some(crate::door_open::CHALLENGE) && a.in_str(i, STEP_IN) == Some(step))
            .collect(),
        &format!("DoorOfTime {step} check"),
    )
}

pub fn patch_module(a: &mut Ainb) -> Res<()> {
    let area: usize = one(a.find_els(AREA_EL).into_iter().filter(|&i| a.prop_str(i, "InstanceName") == Some(AREA)).collect(), "escape area")?;
    let step: usize = step_check(a, ESCAPE_STEP)?;
    let g: usize = a.find_in(area, BOOL, CREATE).ok_or(Error::Format("door gate: the escape area has no create input"))?;
    let x = a.input(BOOL, g).ok_or(Error::Format("door gate: input out of range"))?;
    if x.src < 0 || x.src as usize != step {
        return Err(Error::Format("door gate: the escape area is not created from the DoorOfTime Step03 check"));
    }
    a.force_bool(area, CREATE, true)?;
    if crate::medal_gate::reads_of(a, step) != 0 {
        return Err(Error::Format("door gate: something still reads the DoorOfTime Step03 check"));
    }
    Ok(())
}

pub fn patch_door(a: &mut Ainb) -> Res<()> {
    let i: usize = step_check(a, crate::door_open::STEP)?;
    let g: usize = a.find_in(i, STR, STEP_IN).ok_or(Error::Format("door gate: the door's step check has no step input"))?;
    let o: u32 = a.pool.add(OPEN_STEP);
    a.set_in_val(STR, g, &o.to_le_bytes())
}

pub fn patch_banc(y: &mut Byml) -> Res<()> {
    let actors: &mut Vec<Node> = y.root.get_mut("Actors").and_then(|n| n.as_arr_mut()).ok_or(Error::Format("door gate: the child Hyrule Field map has no actors"))?;
    let a: &mut Node = actors
        .iter_mut()
        .find(|a| matches!(a.get("Hash"), Some(Node::U64(h)) if *h == AREA_HASH))
        .ok_or(Error::Format("door gate: the child Hyrule Field map has no escape area"))?;
    if a.get("Gyaml").and_then(|g| g.as_str()) != Some("Area") || a.get("Presence").is_some() {
        return Err(Error::Format("door gate: the escape area is not the unpatched one"));
    }
    let p: Node = Node::Map(vec![
        ("FlagName".to_string(), Node::Str(ESCAPE_EXP.to_string())),
        ("IsCheckLazy".to_string(), Node::Bool(false)),
        ("IsNegation".to_string(), Node::Bool(false)),
    ]);
    a.set("Presence", p);
    Ok(())
}

pub fn banc_bytes(raw: &[u8]) -> Res<Vec<u8>> {
    let mut y: Byml = Byml::parse(raw)?;
    patch_banc(&mut y)?;
    let out: Vec<u8> = y.write()?;
    if Byml::parse(&out)?.root != y.root {
        return Err(Error::Format("door gate: the patched map does not read back"));
    }
    Ok(out)
}

pub fn patch_check(b: &mut Bf) -> Res<usize> {
    let f: &mut Flow = b.flows.iter_mut().find(|x| x.name == crate::door_fix::FLOW).ok_or(Error::Format("door gate: no DoorOfTime flow"))?;
    if f.events.iter().any(|e| e.name.starts_with(PREFIX)) {
        return Err(Error::Format("door gate: the DoorOfTime flow already carries the gate"));
    }
    let head: usize = f.events.iter().position(|e| e.name == crate::door_fix::FIRE_HEAD).ok_or(Error::Format("door gate: the DoorOfTime flow has no stone count head"))?;
    let ok: bool = matches!(&f.events[head].kind, Kind::Switch { .. })
        && f.events[head].kind.params().and_then(|p| p.get("GameDataName")) == Some(&Val::Str(HEAD_EXP.to_string()));
    if !ok {
        return Err(Error::Format("door gate: the stone count head is not the Fire Stone switch"));
    }
    let (a, x): (u16, u16) = crate::start_state::slot(f, WAIT_ACTOR, WAIT)?;
    let n: usize = f.events.len();
    if n >= NONE as usize {
        return Err(Error::Format("door gate: too many events"));
    }
    f.events.push(Event {
        name: format!("{PREFIX}0"),
        kind: Kind::Action {
            next: NONE,
            actor: a,
            act: x,
            params: Some(Cont { items: vec![("IsWaitEnd".to_string(), Val::Bool(1)), ("Frames".to_string(), Val::Int(1))] }),
        },
    });
    let red: Vec<String> = crate::door_fix::retarget(f, head as u16, n as u16);
    if red.is_empty() {
        return Err(Error::Format("door gate: nothing leads to the stone count head"));
    }
    Ok(red.len())
}

pub fn patch_song(b: &mut Bf, need: &[&str]) -> Res<()> {
    let f: &mut Flow = b.flows.iter_mut().find(|x| x.name == crate::door_fix::FLOW).ok_or(Error::Format("door gate: no DoorOfTime flow"))?;
    if f.events.iter().any(|e| e.name == format!("{PREFIX}1")) {
        return Err(Error::Format("door gate: the DoorOfTime flow already carries the song gate"));
    }
    let find = |f: &Flow, n: &str| -> Res<usize> { f.events.iter().position(|e| e.name == n).ok_or_else(|| Error::Msg(format!("door gate: the DoorOfTime flow has no {n}"))) };
    let check: usize = find(f, SONG_CHECK)?;
    let pass: u16 = match &f.events[check].kind {
        Kind::Switch { cases, params: Some(p), .. } if p.get("GameDataName") == Some(&Val::Str(SONG_CHECK_EXP.to_string())) => {
            cases.iter().find(|c| c.val == 0).map(|c| c.ev).ok_or(Error::Format("door gate: the song check has no pass case"))?
        }
        _ => return Err(Error::Format("door gate: Event60 is not the Step04 check of the door song")),
    };
    let ask = |name: String, slot: (u16, u16), flag: &str, yes: u16| Event {
        name,
        kind: Kind::Switch {
            actor: slot.0,
            query: slot.1,
            cases: vec![Case { val: 0, ev: yes }, Case { val: 1, ev: NONE }],
            params: Some(Cont { items: vec![("GameDataName".to_string(), Val::Str(flag.to_string()))] }),
        },
    };
    match need {
        [] => {
            if crate::door_fix::retarget(f, check as u16, pass).is_empty() {
                return Err(Error::Format("door gate: nothing leads to the door song check"));
            }
        }
        [first, rest @ ..] => {
            let slot: (u16, u16) = crate::start_state::query_slot(f, HOLDER, IS_EXP)?;
            let mut to: u16 = pass;
            for (i, x) in rest.iter().enumerate().rev() {
                let n: usize = f.events.len();
                if n >= NONE as usize {
                    return Err(Error::Format("door gate: too many events"));
                }
                f.events.push(ask(format!("{PREFIX}{}", 2 + i), slot, x, to));
                to = n as u16;
            }
            if let Kind::Switch { cases, params: Some(p), .. } = &mut f.events[check].kind {
                for (_, v) in p.items.iter_mut().filter(|(k, _)| k == "GameDataName") {
                    *v = Val::Str(first.to_string());
                }
                for c in cases.iter_mut().filter(|c| c.val == 0) {
                    c.ev = to;
                }
            }
        }
    }
    let catch: usize = find(f, &format!("{}0", crate::door_fix::PREFIX))?;
    let s6: usize = find(f, crate::door_fix::STEP06)?;
    match &mut f.events[catch].kind {
        Kind::Switch { cases, .. } if cases.len() == 2 && cases[0].ev as usize == s6 => cases[0].ev = cases[1].ev,
        _ => return Err(Error::Format("door gate: the door fix's opened check does not lead to Step06")),
    }
    let n: usize = f.events.len();
    if n >= NONE as usize {
        return Err(Error::Format("door gate: too many events"));
    }
    if crate::door_fix::retarget(f, s6 as u16, n as u16).is_empty() {
        return Err(Error::Format("door gate: nothing leads to the Step06 request"));
    }
    let (a, q): (u16, u16) = crate::start_state::query_slot(f, HOLDER, IS_EXP)?;
    f.events.push(Event {
        name: format!("{PREFIX}1"),
        kind: Kind::Switch {
            actor: a,
            query: q,
            cases: vec![Case { val: 0, ev: s6 as u16 }, Case { val: 1, ev: NONE }],
            params: Some(Cont { items: vec![("GameDataName".to_string(), Val::Str(SONG_CHECK_EXP.to_string()))] }),
        },
    });
    Ok(())
}

pub fn patch_opened(a: &mut Ainb) -> Res<()> {
    let door: usize = crate::door_open::door_element(a).ok_or(Error::Format("door gate: the module has no Temple of Time door"))?;
    let g: usize = a.find_in(door, BOOL, "isOpen").ok_or(Error::Format("door gate: the door has no isOpen input"))?;
    let x = a.input(BOOL, g).ok_or(Error::Format("door gate: input out of range"))?;
    if x.src < 0 {
        return Err(Error::Format("door gate: the door's isOpen is not linked to a gate"));
    }
    let gate: usize = x.src as usize;
    let g2: usize = a.find_in(gate, BOOL, "BoolMulti").ok_or(Error::Format("door gate: the door's gate has no multi input"))?;
    let m = a.input(BOOL, g2).ok_or(Error::Format("door gate: input out of range"))?;
    if m.src > -100 || m.plug < 1 {
        return Err(Error::Format("door gate: the door's gate input is not a multi link"));
    }
    let start: usize = (-100 - i32::from(m.src)) as usize;
    let is_step = |a: &Ainb, e: usize| e < a.els.len() && a.el_name(e) == crate::door_open::STEP_CHECK && a.in_str(e, CHALLENGE_IN) == Some(crate::door_open::CHALLENGE);
    let equal: Vec<usize> = (start..start + m.plug as usize)
        .filter(|&j| crate::door_open::multi_entry(a, j).is_some_and(|(e, o)| o == IS_EQUAL && is_step(a, e as usize)))
        .collect();
    let j: usize = one(equal, "exact step read in the door's gate")?;
    let proxied: bool = (start..start + m.plug as usize).any(|j| crate::door_open::multi_entry(a, j).is_some_and(|(e, _)| (e as usize) < a.els.len() && a.el_name(e as usize) == PROXY));
    if !proxied {
        return Err(Error::Format("door gate: the door's gate does not read the event proxy"));
    }
    let flag: usize = one(a.find_els(FLAG_GET).into_iter().filter(|&i| a.in_str(i, FLAG_IN) == Some(OPENED)).collect(), "opened flag read")?;
    let e: u16 = u16::try_from(flag).map_err(|_| Error::Format("door gate: element index out of range"))?;
    a.multi[8 * j..8 * j + 2].copy_from_slice(&e.to_le_bytes());
    a.multi[8 * j + 2..8 * j + 4].copy_from_slice(&0u16.to_le_bytes());
    let o: usize = a.params[flag].ioe[BOOL][2] as usize * OUT_BOOL;
    let w: &mut [u8] = a.outs[BOOL].get_mut(o..o + 4).ok_or(Error::Format("door gate: the opened flag read has no output"))?;
    let v: u32 = u32::from_le_bytes([w[0], w[1], w[2], w[3]]) | crate::ainb::build::LINKED;
    w.copy_from_slice(&v.to_le_bytes());
    Ok(())
}

pub fn patch_river(a: &mut Ainb) -> Res<()> {
    let req: usize = one(a.find_els(REQUEST).into_iter().filter(|&i| a.prop_str(i, "InstanceName") == Some(RIVER_REQUEST)).collect(), "river turn back request")?;
    let wall: usize = one(a.find_els(SIMPLE).into_iter().filter(|&i| a.prop_str(i, "InstanceName") == Some(RIVER_WALL)).collect(), "river air wall")?;
    if !a.force_bool(req, "EventRequestTrigger", false)? || !a.force_bool(wall, CREATE, false)? {
        return Err(Error::Format("door gate: the river block was already a constant, the module is not the unpatched one"));
    }
    Ok(())
}

pub fn stage_river(rf: &mut Romfs) -> Res<Out> {
    crate::medal_gate::module(rf, RIVER_MODULE, patch_river)
}

pub fn stage_door(rf: &mut Romfs, o: &Opts) -> Res<Vec<Out>> {
    let early: bool = !o.open;
    let late: bool = !o.song;
    let door: Out = crate::medal_gate::module(rf, crate::door_open::MODULE, |a| {
        if early {
            patch_opened(a)?;
        }
        if late {
            patch_door(a)?;
        }
        Ok(())
    })?;
    Ok(vec![door, crate::medal_gate::module(rf, crate::door_open::SWORD_MODULE, crate::door_open::patch_sword)?])
}

pub fn stage_flow(rf: &mut Romfs, o: &Opts) -> Res<Out> {
    let o: Opts = *o;
    staging::rewrite(rf, crate::door_fix::FLOW_FILE, |raw| {
        let mut b: Bf = Bf::parse(raw)?;
        crate::door_fix::patch_flow(&mut b)?;
        if o.escape() {
            patch_check(&mut b)?;
            patch_song(&mut b, &[REQ_EXP])?;
        } else if o.only {
            patch_song(&mut b, &[])?;
        } else {
            patch_song(&mut b, &[FIRE_EXP, WATER_EXP])?;
        }
        let out: Vec<u8> = b.write()?;
        if Bf::parse(&out)? != b {
            return Err(Error::Format("door gate: the patched flow does not parse back to the edited model"));
        }
        Ok(out)
    })
}

pub fn stage_escape(rf: &mut Romfs, more: &mut Vec<Out>, from: Option<&PB>) -> Res<()> {
    more.push(crate::medal_gate::module(rf, MODULE, patch_module)?);
    match more.iter().position(|o| o.path == AREA_FILE) {
        Some(i) => {
            let raw: Vec<u8> = rf.zs.decompress(&more[i].z)?;
            let out: Vec<u8> = banc_bytes(&raw)?;
            let old: usize = more[i].old;
            more[i] = staging::banc_out(rf, AREA_FILE, old, &out)?;
        }
        None => {
            let raw: Vec<u8> = staging::banc_raw(rf, AREA_FILE, from)?;
            let out: Vec<u8> = banc_bytes(&raw)?;
            more.push(staging::banc_out(rf, AREA_FILE, raw.len(), &out)?);
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet as BS;

    fn holds(t: &[(i32, String)], on: &[&str]) -> bool {
        let mut s: Vec<bool> = Vec::new();
        for (op, f) in t {
            match *op {
                0 => s.push(on.contains(&f.as_str())),
                AND_OP => {
                    let b: bool = s.pop().unwrap();
                    let a: bool = s.pop().unwrap();
                    s.push(a && b);
                }
                4 => {
                    let b: bool = s.pop().unwrap();
                    let a: bool = s.pop().unwrap();
                    s.push(a || b);
                }
                NOT_OP => {
                    let a: bool = s.pop().unwrap();
                    s.push(!a);
                }
                _ => panic!("op {op}"),
            }
        }
        assert_eq!(s.len(), 1);
        s[0]
    }

    const FOREST: &str = "IsGet.Item_Stone_Forest";
    const FIRE: &str = "IsGet.Item_Stone_Fire";
    const WATER: &str = "IsGet.Item_Stone_Water";
    const LIGHT: &str = "IsGet.Item_Medal_Light";

    #[test]
    fn classic_needs_the_three_stones_after_the_send_off_and_before_the_escape() {
        let t: Vec<(i32, String)> = terms(Need::Stones);
        assert!(holds(&t, &[SENT, FOREST, FIRE, WATER]));
        assert!(!holds(&t, &[SENT, FOREST, FIRE]));
        assert!(!holds(&t, &[SENT, FOREST, FIRE, LIGHT]));
        assert!(!holds(&t, &[FOREST, FIRE, WATER]));
        assert!(!holds(&t, &[SENT, ESCAPED, FOREST, FIRE, WATER]));
    }

    #[test]
    fn the_door_song_waits_for_the_requirement_only() {
        let t: Vec<(i32, String)> = req_terms(Need::Stones);
        assert!(holds(&t, &[FOREST, FIRE, WATER]));
        assert!(holds(&t, &[ESCAPED, FOREST, FIRE, WATER]));
        assert!(!holds(&t, &[SENT, FOREST, FIRE]));
        let none: Vec<(i32, String)> = req_terms(Need::Rewards(0));
        assert!(holds(&none, &[]) && holds(&none, &[SENT]));
    }

    #[test]
    fn dungeon_rewards_count_any_mix() {
        let t: Vec<(i32, String)> = terms(Need::Rewards(2));
        assert!(holds(&t, &[SENT, FOREST, LIGHT]));
        assert!(!holds(&t, &[SENT, LIGHT]));
        let none: Vec<(i32, String)> = terms(Need::Rewards(0));
        assert!(holds(&none, &[SENT]));
        assert!(!holds(&none, &[SENT, ESCAPED]));
    }

    #[test]
    fn the_settings_read_back_and_default_to_the_game() {
        let mut bad: Vec<String> = Vec::new();
        let mut s: Settings = Settings::default();
        let d: Opts = read(&s, false, &mut bad);
        assert_eq!(d, Opts::default());
        assert!(d.exps() == vec![Edit::Exp(REACTS.to_string(), vec![(0, OPENED.to_string())])] && d.logic().is_empty() && d.notes() == vec![VANILLA_NOTE.to_string()]);
        s.set(SETTING, Value::s(STONES)).unwrap();
        assert_eq!(read(&s, false, &mut bad).need, Need::Stones);
        s.set(SETTING, Value::s(DUNGEONS)).unwrap();
        s.set(REWARDS, Value::Int(5)).unwrap();
        let o: Opts = read(&s, false, &mut bad);
        assert_eq!(o.need, Need::Rewards(5));
        assert_eq!(o.logic(), vec![(SETTING, DUNGEONS), (REWARDS, "5")]);
        assert_eq!(o.exps(), vec![Edit::Exp(REACTS.to_string(), vec![(0, OPENED.to_string())]), Edit::Exp(ESCAPE_EXP.to_string(), terms(Need::Rewards(5))), Edit::Exp(REQ_EXP.to_string(), req_terms(Need::Rewards(5)))]);
        s.set(SONG, Value::Bool(false)).unwrap();
        assert!(!read(&s, false, &mut bad).song);
        assert!(read(&s, false, &mut bad).logic().contains(&(crate::open_world::DOOR, "stones")));
        assert!(read(&s, true, &mut bad).song);
        assert!(bad.is_empty(), "{bad:?}");
    }

    #[test]
    fn an_old_open_door_becomes_the_open_choice() {
        let mut s: Settings = Settings::default();
        s.set(crate::open_world::DOOR, Value::Bool(true)).unwrap();
        s.normalize();
        assert_eq!(s.text(SETTING).unwrap(), OPEN);
        assert!(!s.flag(crate::open_world::DOOR).unwrap());
        let mut bad: Vec<String> = Vec::new();
        assert!(crate::open_world::Gates::read(&s, &mut bad).door);
        assert_eq!(read(&s, true, &mut bad), Opts { open: true, ..Opts::default() });
        assert!(bad.is_empty(), "{bad:?}");
    }

    fn real() -> Option<Romfs> {
        let p: PB = ootrr_game::test_romfs::path();
        if p.join("Pack").join("ZsDic.pack.zs").is_file() {
            Romfs::open(&p).ok()
        } else {
            eprintln!("skipped, no RomFS dump at {} (set OOTRR_ROMFS)", p.display());
            None
        }
    }

    #[test]
    fn the_stone_check_does_nothing_and_the_rest_of_the_quest_stays() {
        let Some(mut rf) = real() else { return };
        let mut b: Bf = Bf::parse(&rf.read_zs(crate::door_fix::FLOW_FILE).unwrap()).unwrap();
        crate::door_fix::patch_flow(&mut b).unwrap();
        let before: Flow = b.flows[0].clone();
        assert!(patch_check(&mut b).unwrap() >= 2);
        let f: &Flow = &b.flows[0];
        let gate: usize = f.events.iter().position(|e| e.name == format!("{PREFIX}0")).unwrap();
        let e = f.entries.iter().find(|e| e.name == "DoorOfTime_CheckStoneNum").unwrap();
        assert_eq!(e.main as usize, gate);
        assert!(matches!(f.events[gate].kind, Kind::Action { next: NONE, .. }));
        let send: &Event = f.events.iter().find(|e| e.name == crate::door_fix::SEND_OFF).unwrap();
        assert!(matches!(send.kind, Kind::Action { next, .. } if next as usize == gate));
        let mains: BS<(String, u16)> = before.entries.iter().filter(|e| e.name != "DoorOfTime_CheckStoneNum").map(|e| (e.name.clone(), e.main)).collect();
        let after: BS<(String, u16)> = f.entries.iter().filter(|e| e.name != "DoorOfTime_CheckStoneNum").map(|e| (e.name.clone(), e.main)).collect();
        assert_eq!(mains, after);
        assert!(patch_check(&mut b).is_err());
        let out: Vec<u8> = b.write().unwrap();
        assert_eq!(Bf::parse(&out).unwrap(), b);
    }

    #[test]
    fn the_escape_area_is_always_created_and_the_map_gates_it() {
        let Some(mut rf) = real() else { return };
        let mut a: Ainb = Ainb::parse(&rf.read(MODULE).unwrap()).unwrap();
        patch_module(&mut a).unwrap();
        assert!(patch_module(&mut a).is_err());
        assert!(crate::ainb::audit(&Ainb::parse(&a.write().unwrap()).unwrap()).is_empty());
        let raw: Vec<u8> = rf.read_zs(AREA_FILE).unwrap();
        let y: Byml = Byml::parse(&banc_bytes(&raw).unwrap()).unwrap();
        let acts: &[Node] = y.root.get("Actors").and_then(|n| n.as_arr()).unwrap();
        let area: &Node = acts.iter().find(|a| a.get("Hash") == Some(&Node::U64(AREA_HASH))).unwrap();
        assert_eq!(area.get("Presence").and_then(|p| p.get("FlagName")).and_then(|n| n.as_str()), Some(ESCAPE_EXP));
        assert!(banc_bytes(&y.write().unwrap()).is_err());
    }

    #[test]
    fn song_only_reads_back_and_asks_for_the_song() {
        let mut s: Settings = Settings::default();
        s.set(SETTING, Value::s(SONG_ONLY)).unwrap();
        s.set(SONG, Value::Bool(false)).unwrap();
        let mut bad: Vec<String> = Vec::new();
        let o: Opts = read(&s, false, &mut bad);
        assert!(o.only && o.song && !o.escape() && o.flow());
        assert_eq!(o.logic(), vec![(crate::open_world::DOOR, "sot")]);
        assert_eq!(o.exps().len(), 1);
        assert!(bad.is_empty(), "{bad:?}");
    }

    #[test]
    fn the_song_opens_the_door_without_skipping_the_quest() {
        let Some(mut rf) = real() else { return };
        let mut b: Bf = Bf::parse(&rf.read_zs(crate::door_fix::FLOW_FILE).unwrap()).unwrap();
        crate::door_fix::patch_flow(&mut b).unwrap();
        patch_song(&mut b, &[]).unwrap();
        let f: &Flow = &b.flows[0];
        let at = |n: &str| f.events.iter().position(|e| e.name == n).unwrap();
        let (check, s6, guard): (usize, usize, usize) = (at(SONG_CHECK), at(crate::door_fix::STEP06), at(&format!("{PREFIX}1")));
        let leads = |to: usize| -> Vec<String> {
            f.events.iter().filter(|e| match &e.kind {
                Kind::Action { next, .. } | Kind::Sub { next, .. } | Kind::Join { next } => *next as usize == to,
                Kind::Switch { cases, .. } | Kind::SubSwitch { cases, .. } => cases.iter().any(|c| c.ev as usize == to),
                _ => false,
            }).map(|e| e.name.clone()).collect()
        };
        assert!(leads(check).is_empty());
        assert_eq!(leads(s6), vec![format!("{PREFIX}1")]);
        assert!(!leads(guard).is_empty());
        assert!(patch_song(&mut b, &[]).is_err());
        let mut c: Bf = Bf::parse(&rf.read_zs(crate::door_fix::FLOW_FILE).unwrap()).unwrap();
        crate::door_fix::patch_flow(&mut c).unwrap();
        patch_check(&mut c).unwrap();
        patch_song(&mut c, &[REQ_EXP]).unwrap();
        let g: &Flow = &c.flows[0];
        let e: &Event = g.events.iter().find(|e| e.name == SONG_CHECK).unwrap();
        assert_eq!(e.kind.params().and_then(|p| p.get("GameDataName")), Some(&Val::Str(REQ_EXP.to_string())));
        let out: Vec<u8> = b.write().unwrap();
        assert_eq!(Bf::parse(&out).unwrap(), b);
        let mut a: Ainb = Ainb::parse(&rf.read(crate::door_open::MODULE).unwrap()).unwrap();
        patch_opened(&mut a).unwrap();
        assert!(patch_opened(&mut a).is_err());
        assert!(crate::ainb::audit(&Ainb::parse(&a.write().unwrap()).unwrap()).is_empty());
        assert_eq!(stage_door(&mut rf, &Opts { only: true, ..Opts::default() }).unwrap().len(), 2);
    }

    #[test]
    fn the_river_never_turns_link_back() {
        let Some(mut rf) = real() else { return };
        let mut a: Ainb = Ainb::parse(&rf.read(RIVER_MODULE).unwrap()).unwrap();
        patch_river(&mut a).unwrap();
        assert!(patch_river(&mut a).is_err());
        assert!(crate::ainb::audit(&Ainb::parse(&a.write().unwrap()).unwrap()).is_empty());
    }

    #[test]
    fn the_songless_door_opens_from_step04() {
        let Some(mut rf) = real() else { return };
        let mut a: Ainb = Ainb::parse(&rf.read(crate::door_open::MODULE).unwrap()).unwrap();
        patch_door(&mut a).unwrap();
        let back: Ainb = Ainb::parse(&a.write().unwrap()).unwrap();
        assert!(step_check(&back, OPEN_STEP).is_ok());
        assert!(step_check(&back, crate::door_open::STEP).is_err());
        assert_eq!(stage_door(&mut rf, &Opts { song: false, ..Opts::default() }).unwrap().len(), 2);
        assert_eq!(stage_door(&mut rf, &Opts { need: Need::Stones, song: false, only: false, open: false }).unwrap().len(), 2);
    }

    #[test]
    fn the_escape_formula_names_only_known_flags() {
        let Some(mut rf) = real() else { return };
        let mut more: Vec<Out> = Vec::new();
        let edits: Vec<Edit> = [Need::Stones, Need::Rewards(9), Need::Rewards(0)].iter().map(|n| Edit::Exp(ESCAPE_EXP.to_string(), terms(*n))).collect();
        for e in edits {
            crate::start_defaults::stage(&mut rf, &mut more, &[e]).unwrap();
        }
        assert_eq!(more.len(), 3);
    }
}
