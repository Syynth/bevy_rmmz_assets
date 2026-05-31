//! `Troops.json` — enemy groups and their battle event pages.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::EventCommand;

/// A member (enemy placement) of a troop.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct TroopMember {
    /// Id of the enemy placed.
    pub enemy_id: i32,
    /// Battle-screen x position.
    pub x: i32,
    /// Battle-screen y position.
    pub y: i32,
    /// Whether the member starts hidden (appears mid-battle).
    pub hidden: bool,
}

/// The condition gating a troop event page.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct TroopPageConditions {
    /// HP percentage threshold for the actor condition.
    pub actor_hp: i32,
    /// Actor id for the actor condition.
    pub actor_id: i32,
    /// Whether the actor condition is enabled.
    pub actor_valid: bool,
    /// HP percentage threshold for the enemy condition.
    pub enemy_hp: i32,
    /// Troop member index for the enemy condition.
    pub enemy_index: i32,
    /// Whether the enemy condition is enabled.
    pub enemy_valid: bool,
    /// Switch id for the switch condition.
    pub switch_id: i32,
    /// Whether the switch condition is enabled.
    pub switch_valid: bool,
    /// Turn number `a` for the turn condition (`turn_a * n + turn_b`).
    pub turn_a: i32,
    /// Turn offset `b` for the turn condition.
    pub turn_b: i32,
    /// Whether the page fires at turn end.
    pub turn_ending: bool,
    /// Whether the turn condition is enabled.
    pub turn_valid: bool,
}

/// A battle event page within a troop.
///
/// Not `Reflect`: it holds an [`EventCommand`] list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TroopPage {
    /// Condition that must hold for the page to run.
    pub conditions: TroopPageConditions,
    /// The page's command list.
    pub list: Vec<EventCommand>,
    /// Re-trigger span: 0 battle, 1 turn, 2 moment.
    pub span: i32,
}

/// A single entry in `Troops.json`.
///
/// Not `Reflect`: its pages hold [`EventCommand`] lists.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Troop {
    /// Database id (1-based).
    pub id: i32,
    /// Enemy placements.
    pub members: Vec<TroopMember>,
    /// Display name.
    pub name: String,
    /// Battle event pages.
    pub pages: Vec<TroopPage>,
}
