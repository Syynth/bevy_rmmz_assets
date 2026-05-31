//! `Enemies.json` — battle enemies.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// One entry in an enemy's action pattern.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct EnemyAction {
    /// First condition parameter; meaning depends on `condition_type`.
    pub condition_param1: f64,
    /// Second condition parameter; meaning depends on `condition_type`.
    pub condition_param2: f64,
    /// Condition kind gating the action (turn, HP, MP, state, party level, switch).
    pub condition_type: i32,
    /// Relative weight used when selecting among valid actions.
    pub rating: i32,
    /// Id of the skill performed.
    pub skill_id: i32,
}

/// A possible item drop from an enemy.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct DropItem {
    /// Id of the dropped data entry; interpreted per `kind`.
    pub data_id: i32,
    /// Drop probability denominator (1-in-`denominator`).
    pub denominator: i32,
    /// Drop kind: 0 none, 1 item, 2 weapon, 3 armor.
    pub kind: i32,
}

/// A single enemy entry from `Enemies.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Enemy {
    /// Database id (1-based).
    pub id: i32,
    /// Action pattern.
    pub actions: Vec<EnemyAction>,
    /// Battler graphic hue rotation.
    pub battler_hue: i32,
    /// Battler graphic file name.
    pub battler_name: String,
    /// Item drops.
    pub drop_items: Vec<DropItem>,
    /// EXP awarded on defeat.
    pub exp: i32,
    /// Innate traits.
    pub traits: Vec<Trait>,
    /// Gold awarded on defeat.
    pub gold: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Base parameter values in order: HP, MP, ATK, DEF, MAT, MDF, AGI, LUK.
    pub params: [i32; 8],
}
