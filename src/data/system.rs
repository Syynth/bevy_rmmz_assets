//! `System.json` — global game configuration.
//!
//! `System.json` is a single large object with many fields. These models cover
//! the commonly-used ones and use `#[serde(default)]` so unmodeled or
//! version-specific fields are ignored rather than failing the load. Validated
//! against a real MZ project's `System.json`; the long tail of less-common
//! fields (vehicles, title/gameover assets, advanced options) is intentionally
//! unmodeled and ignored on load.

use std::collections::HashMap;

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::AudioFile;

/// UI vocabulary from `System.terms`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(default)]
pub struct Terms {
    /// Basic terms (level, HP, MP, etc.), indexed by MZ's fixed order.
    pub basic: Vec<String>,
    /// Command labels; entries may be `null` when a command is hidden.
    pub commands: Vec<Option<String>>,
    /// Parameter names (max HP, attack, …).
    pub params: Vec<String>,
    /// Message templates keyed by MZ's message id (e.g. `"emerge"`, `"levelUp"`).
    pub messages: HashMap<String, String>,
}

/// A single (top-level) entry parsed from `System.json`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct System {
    /// Game title.
    pub game_title: String,
    /// Save-data version id used to invalidate old saves.
    pub version_id: i32,
    /// Currency unit label.
    pub currency_unit: String,
    /// Actor ids of the initial party.
    pub party_members: Vec<i32>,
    /// Element names; index 0 is the empty/normal-attack slot. Entries may be
    /// `null`/empty.
    pub elements: Vec<Option<String>>,
    /// Skill type names; entries may be `null`/empty.
    pub skill_types: Vec<Option<String>>,
    /// Weapon type names; entries may be `null`/empty.
    pub weapon_types: Vec<Option<String>>,
    /// Armor type names; entries may be `null`/empty.
    pub armor_types: Vec<Option<String>>,
    /// Equip slot type names; entries may be `null`/empty.
    pub equip_types: Vec<Option<String>>,
    /// Switch names indexed by switch id; entries may be `null`/empty.
    pub switches: Vec<Option<String>>,
    /// Variable names indexed by variable id; entries may be `null`/empty.
    pub variables: Vec<Option<String>>,
    /// UI vocabulary.
    pub terms: Terms,
    /// System sound effects, in MZ's fixed order (cursor, ok, buzzer, …).
    pub sounds: Vec<AudioFile>,
    /// Map the new game starts on.
    pub start_map_id: i32,
    /// Starting x position.
    pub start_x: i32,
    /// Starting y position.
    pub start_y: i32,
    /// Whether TP is displayed in battle.
    pub opt_display_tp: bool,
    /// Whether the title screen is drawn.
    pub opt_draw_title: bool,
    /// Whether EXP is gained for reserve members.
    pub opt_extra_exp: bool,
    /// Whether floor damage can reduce HP to 0.
    pub opt_floor_death: bool,
    /// Whether followers are shown.
    pub opt_followers: bool,
    /// Whether the side-view battle system is used.
    pub opt_side_view: bool,
    /// Whether slip (regen) damage can reduce HP to 0.
    pub opt_slip_death: bool,
    /// Whether the leader is transparent on the map.
    pub opt_transparent: bool,
}
