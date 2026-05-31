//! `Tilesets.json` — tileset definitions.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A single entry in `Tilesets.json`.
///
/// The file is a null-padded array (index 0 is `null`); padding is handled by
/// the asset layer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Tileset {
    /// Database id (1-based).
    pub id: i32,
    /// Passage/terrain/flag bits, indexed by tile id (one entry per tile;
    /// typically 8192 entries). Kept as a `Vec` because the length is large and
    /// effectively a bitmap rather than a fixed-arity record.
    pub flags: Vec<i32>,
    /// Tileset mode: 0 world type, 1 area type.
    pub mode: i32,
    /// Display name.
    pub name: String,
    /// Author note; the conventional home of `<tag:value>` metadata.
    pub note: String,
    /// The five-to-nine constituent tile sheet file names (A1–A5, B–E).
    pub tileset_names: Vec<String>,
}
