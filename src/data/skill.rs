//! `Skills.json` — usable skills.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::{Damage, Effect};

/// A single skill entry from `Skills.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Skill {
    /// Database id (1-based).
    pub id: i32,
    /// Animation id played on use.
    pub animation_id: i32,
    /// Damage calculation.
    pub damage: Damage,
    /// Help/description text.
    pub description: String,
    /// Effects applied on use.
    pub effects: Vec<Effect>,
    /// Hit type: 0 certain, 1 physical, 2 magical.
    pub hit_type: i32,
    /// Icon index.
    pub icon_index: i32,
    /// First usage message line.
    pub message1: String,
    /// Second usage message line.
    pub message2: String,
    /// How the usage message is shown: 0 hide, 1 use `message1`, 2 use both.
    pub message_type: i32,
    /// MP cost.
    pub mp_cost: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// When the skill may be used: 0 always, 1 battle, 2 menu, 3 never.
    pub occasion: i32,
    /// Number of repeats.
    pub repeats: i32,
    /// First required weapon type id.
    pub required_wtype_id1: i32,
    /// Second required weapon type id.
    pub required_wtype_id2: i32,
    /// Target scope code.
    pub scope: i32,
    /// Speed correction applied to action order.
    pub speed: i32,
    /// Skill type id.
    pub stype_id: i32,
    /// Base success rate (percent).
    pub success_rate: i32,
    /// TP cost.
    pub tp_cost: i32,
    /// TP gained on use.
    pub tp_gain: i32,
}
