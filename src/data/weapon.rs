//! `Weapons.json` — equippable weapons.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// A single weapon entry from `Weapons.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Weapon {
    /// Database id (1-based).
    pub id: i32,
    /// Animation id used when attacking with this weapon.
    pub animation_id: i32,
    /// Help/description text.
    pub description: String,
    /// Equip-slot type id.
    pub etype_id: i32,
    /// Traits granted while equipped.
    pub traits: Vec<Trait>,
    /// Icon index.
    pub icon_index: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Parameter bonuses, one per base parameter (8 entries).
    pub params: Vec<i32>,
    /// Buy price.
    pub price: i32,
    /// Weapon type id.
    pub wtype_id: i32,
}
