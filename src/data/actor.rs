//! `Actors.json` — the playable/party actors.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::Trait;

/// A single actor entry from `Actors.json`.
///
/// In the file these are stored in a null-padded array (index 0 is `null`); that
/// padding is handled by the asset layer, not this struct.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Actor {
    /// Database id (1-based).
    pub id: i32,
    /// SV battler graphic file name.
    pub battler_name: String,
    /// Index into the character sheet (`character_name`).
    pub character_index: i32,
    /// Walking-character sheet file name.
    pub character_name: String,
    /// Initial class id.
    pub class_id: i32,
    /// Initial equipment, as data ids per equip slot (0 = empty).
    pub equips: Vec<i32>,
    /// Index into the face sheet (`face_name`).
    pub face_index: i32,
    /// Face graphic file name.
    pub face_name: String,
    /// Innate traits.
    pub traits: Vec<Trait>,
    /// Level the actor starts at.
    pub initial_level: i32,
    /// Maximum level the actor can reach.
    pub max_level: i32,
    /// Display name.
    pub name: String,
    /// Secondary display name (title/epithet).
    pub nickname: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Multi-line profile/biography text.
    pub profile: String,
}
