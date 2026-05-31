//! `MapInfos.json` — the map tree shown in the editor.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A single entry in `MapInfos.json`.
///
/// The file is a null-padded array (index 0 is `null`); padding is handled by
/// the asset layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct MapInfo {
    /// Map id this entry describes (matches the `Map###.json` number).
    pub id: i32,
    /// Whether the node is expanded in the editor's map tree.
    pub expanded: bool,
    /// Display name in the map tree.
    pub name: String,
    /// Sort order among siblings.
    pub order: i32,
    /// Parent map id (0 = top level).
    pub parent_id: i32,
    /// Saved editor horizontal scroll position.
    pub scroll_x: f64,
    /// Saved editor vertical scroll position.
    pub scroll_y: f64,
}
