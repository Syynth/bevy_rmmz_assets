//! `States.json` — battler states (buffs/debuffs/conditions).

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// A single state entry from `States.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// Database id (1-based).
    pub id: i32,
    /// When the state is auto-removed: 0 never, 1 by turn count, 2 by action end.
    pub auto_removal_timing: i32,
    /// Chance (percent) the state is removed when the bearer takes damage.
    pub chance_by_damage: i32,
    /// Innate traits applied while the state is active.
    pub traits: Vec<Trait>,
    /// Icon index.
    pub icon_index: i32,
    /// Maximum duration in turns (with `min_turns`).
    pub max_turns: i32,
    /// Message shown when an actor receives the state.
    pub message1: String,
    /// Message shown when an enemy receives the state.
    pub message2: String,
    /// Message shown when the state persists.
    pub message3: String,
    /// Message shown when the state is removed.
    pub message4: String,
    /// Minimum duration in turns (with `max_turns`).
    pub min_turns: i32,
    /// SV motion played while afflicted.
    pub motion: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// SV overlay graphic shown while afflicted.
    pub overlay: i32,
    /// Display priority when multiple states are active.
    pub priority: i32,
    /// Whether the removal-by-damage roll is checked.
    pub release_by_damage: bool,
    /// Whether the state is removed when the battle ends.
    pub remove_at_battle_end: bool,
    /// Whether the state can be removed by taking damage.
    pub remove_by_damage: bool,
    /// Whether the state is removed when its restriction would change.
    pub remove_by_restriction: bool,
    /// Whether the state is removed by walking.
    pub remove_by_walking: bool,
    /// Action restriction imposed: 0 none .. 4 cannot move.
    pub restriction: i32,
    /// Steps required to remove the state when `remove_by_walking` is set.
    pub steps_to_remove: i32,
}
