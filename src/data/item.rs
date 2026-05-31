//! `Items.json` — consumable and key items.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::{Damage, Effect};

/// A single item entry from `Items.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Item {
    /// Database id (1-based).
    pub id: i32,
    /// Animation id played on use.
    pub animation_id: i32,
    /// Whether the item is consumed on use.
    pub consumable: bool,
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
    /// Item type id: 1 regular, 2 key item, 3 hidden A, 4 hidden B.
    pub itype_id: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// When the item may be used: 0 always, 1 battle, 2 menu, 3 never.
    pub occasion: i32,
    /// Buy price.
    pub price: i32,
    /// Number of repeats.
    pub repeats: i32,
    /// Target scope code.
    pub scope: i32,
    /// Speed correction applied to action order.
    pub speed: i32,
    /// Base success rate (percent).
    pub success_rate: i32,
    /// TP gained on use.
    pub tp_gain: i32,
}
