//! `Armors.json` — equippable armor.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// A single armor entry from `Armors.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Armor {
    /// Database id (1-based).
    pub id: i32,
    /// Armor type id.
    pub atype_id: i32,
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
}
