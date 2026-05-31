//! `Map###.json` — individual map data (tiles + events).
//!
//! Maps are large and event-heavy. These models cover the documented MZ fields
//! and use `#[serde(default)]` for resilience. Types that hold event command
//! lists (or move-route commands) are **not** `Reflect`, since their parameters
//! are `serde_json::Value`. Validated against a real MZ project's maps;
//! rarely-used or plugin-specific fields may remain unmodeled and are ignored on
//! load.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::{AudioFile, EventCommand};

/// A single encounter entry on a map.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Encounter {
    /// Troop id encountered.
    pub troop_id: i32,
    /// Relative encounter weight.
    pub weight: i32,
    /// Region ids the encounter is restricted to (empty = whole map).
    pub region_set: Vec<i32>,
}

/// A single command within a move route.
///
/// Not `Reflect`: its parameters are `serde_json::Value`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveCommand {
    /// Move command code.
    pub code: i32,
    /// Command arguments, whose shape depends on `code`.
    pub parameters: Vec<serde_json::Value>,
}

/// A character move route.
///
/// Not `Reflect`: it holds [`MoveCommand`]s.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveRoute {
    /// The ordered move commands.
    pub list: Vec<MoveCommand>,
    /// Whether the route repeats.
    pub repeat: bool,
    /// Whether the route may be skipped if movement is impossible.
    pub skippable: bool,
    /// Whether the event waits for the route to finish.
    pub wait: bool,
}

/// The condition gating a map event page.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct MapEventPageConditions {
    /// Actor id for the actor-in-party condition.
    pub actor_id: i32,
    /// Whether the actor condition is enabled.
    pub actor_valid: bool,
    /// Item id for the item-possessed condition.
    pub item_id: i32,
    /// Whether the item condition is enabled.
    pub item_valid: bool,
    /// Self-switch channel (`"A"`..=`"D"`) for the self-switch condition.
    pub self_switch_ch: String,
    /// Whether the self-switch condition is enabled.
    pub self_switch_valid: bool,
    /// First switch id condition.
    pub switch1_id: i32,
    /// Whether the first switch condition is enabled.
    pub switch1_valid: bool,
    /// Second switch id condition.
    pub switch2_id: i32,
    /// Whether the second switch condition is enabled.
    pub switch2_valid: bool,
    /// Variable id for the variable condition.
    pub variable_id: i32,
    /// Whether the variable condition is enabled.
    pub variable_valid: bool,
    /// Threshold the variable must meet or exceed.
    pub variable_value: i32,
}

/// The graphic shown for a map event page.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct MapEventPageImage {
    /// Character sheet index.
    pub character_index: i32,
    /// Character sheet file name.
    pub character_name: String,
    /// Facing direction (2 down, 4 left, 6 right, 8 up).
    pub direction: i32,
    /// Animation pattern column.
    pub pattern: i32,
    /// Tile id when the event is drawn as a tile instead of a character.
    pub tile_id: i32,
}

/// A single page of a map event.
///
/// Not `Reflect`: it holds an [`EventCommand`] list and a [`MoveRoute`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MapEventPage {
    /// Activation condition.
    pub conditions: MapEventPageConditions,
    /// Whether the event's facing is fixed.
    pub direction_fix: bool,
    /// Page graphic.
    pub image: MapEventPageImage,
    /// The page's command list.
    pub list: Vec<EventCommand>,
    /// Autonomous movement frequency.
    pub move_frequency: i32,
    /// Autonomous move route (used when `move_type` is custom).
    pub move_route: MoveRoute,
    /// Autonomous movement speed.
    pub move_speed: i32,
    /// Autonomous movement type: 0 fixed, 1 random, 2 approach, 3 custom.
    pub move_type: i32,
    /// Stacking priority: 0 below, 1 same, 2 above the player.
    pub priority_type: i32,
    /// Whether stepping animation plays.
    pub step_anime: bool,
    /// Whether the event passes through other characters.
    pub through: bool,
    /// Trigger: 0 action button, 1 player touch, 2 event touch, 3 autorun,
    /// 4 parallel.
    pub trigger: i32,
    /// Whether walking animation plays.
    pub walk_anime: bool,
}

/// A single event placed on a map.
///
/// Not `Reflect`: its pages hold [`EventCommand`] lists.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MapEvent {
    /// Event id (unique within the map).
    pub id: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Event pages, evaluated top-to-bottom for the first satisfied condition.
    pub pages: Vec<MapEventPage>,
    /// Tile x position.
    pub x: i32,
    /// Tile y position.
    pub y: i32,
}

/// A single `Map###.json` file.
///
/// Not `Reflect`: its events hold [`EventCommand`] lists.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Map {
    /// Whether the map BGM autoplays on entry.
    pub autoplay_bgm: bool,
    /// Whether the map BGS autoplays on entry.
    pub autoplay_bgs: bool,
    /// First battleback graphic name.
    pub battleback1_name: String,
    /// Second battleback graphic name.
    pub battleback2_name: String,
    /// Map BGM (used when `autoplay_bgm` is set).
    pub bgm: AudioFile,
    /// Map BGS (used when `autoplay_bgs` is set).
    pub bgs: AudioFile,
    /// Whether dashing is disabled on this map.
    pub disable_dashing: bool,
    /// In-game display name (empty hides the name window).
    pub display_name: String,
    /// Random encounter table.
    pub encounter_list: Vec<Encounter>,
    /// Average steps between random encounters.
    pub encounter_step: i32,
    /// Map height in tiles.
    pub height: i32,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// Whether the parallax scrolls horizontally.
    pub parallax_loop_x: bool,
    /// Whether the parallax scrolls vertically.
    pub parallax_loop_y: bool,
    /// Parallax background file name.
    pub parallax_name: String,
    /// Whether the parallax is shown in the editor.
    pub parallax_show: bool,
    /// Parallax horizontal scroll speed.
    pub parallax_sx: i32,
    /// Parallax vertical scroll speed.
    pub parallax_sy: i32,
    /// Scroll type: 0 none, 1 loop vertical, 2 loop horizontal, 3 loop both.
    pub scroll_type: i32,
    /// Whether the map specifies its own battleback.
    pub specify_battleback: bool,
    /// Tileset id used by the map.
    pub tileset_id: i32,
    /// Map width in tiles.
    pub width: i32,
    /// Flattened tile id grid (`width * height * layers`).
    pub data: Vec<i32>,
    /// Map events, indexed by event id with `null` gaps (index 0 is `null`).
    pub events: Vec<Option<MapEvent>>,
}
