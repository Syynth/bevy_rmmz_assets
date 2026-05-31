//! `Classes.json` — actor classes.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// A skill the class learns at a given level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Learning {
    /// Level at which the skill is learned.
    pub level: i32,
    /// Author note for this learning entry.
    pub note: String,
    /// Id of the learned skill.
    pub skill_id: i32,
}

/// A single class entry from `Classes.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    /// Database id (1-based).
    pub id: i32,
    /// EXP curve parameters: `[basis, extra, accelerationA, accelerationB]`.
    pub exp_params: Vec<i32>,
    /// Innate traits.
    pub traits: Vec<Trait>,
    /// Skills learned by level.
    pub learnings: Vec<Learning>,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Parameter growth curves, indexed `params[param_id][level]` (8 params).
    pub params: Vec<Vec<i32>>,
}
