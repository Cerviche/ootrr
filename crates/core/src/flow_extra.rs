pub mod analyzed;
mod apply;
mod clone;
pub mod cows;
mod data;
pub mod frogs;
mod gate;
mod op;
pub mod potion;
pub mod sim;
pub mod trade;

pub use apply::{always_ops, edit_bytes, edit_flow, ops_for, ops_of};
pub use data::{Extra, Spec, JSON};
pub use gate::{bound, fits, open, pouch, POUCH_ONLY};
pub use op::{apply as apply_op, How, Once, Op, ITEM};

pub const SETTING: &str = "shuffle_rewards_extra";

pub const MERCHANTS: u16 = 97;
pub const MERCHANTS_SETTING: &str = "shuffle_merchants";

pub fn merchants(s: &crate::settings::Settings, bad: &mut Vec<String>) -> (bool, bool) {
    match s.text(MERCHANTS_SETTING) {
        Ok("off") => (false, false),
        Ok("beans") => (true, false),
        Ok("no_beans") => (false, true),
        Ok("all") => (true, true),
        Ok(v) => {
            bad.push(format!("{MERCHANTS_SETTING} is {v}, which is not a known choice"));
            (false, false)
        }
        Err(e) => {
            bad.push(e.to_string());
            (false, false)
        }
    }
}
