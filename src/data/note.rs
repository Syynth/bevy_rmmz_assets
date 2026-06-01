//! The [`HasNote`] and [`HasId`] traits implemented by database records.

use crate::data::{Actor, Armor, Class, Enemy, Item, Map, MapEvent, Skill, State, Tileset, Weapon};

/// Implemented by database records that carry an author `note` field — the
/// conventional home of RPG Maker's `<tag:value>` metadata.
///
/// The note-metadata parsing layer uses this to find the raw text to parse,
/// without caring which concrete record type it came from.
pub trait HasNote {
    /// The record's raw note text.
    fn note(&self) -> &str;
}

/// Implemented by database records with a 1-based `id`.
///
/// The note-metadata cache keys parsed notes by record type and id, so a record
/// must expose its id to look its metadata up.
pub trait HasId {
    /// The record's 1-based database id.
    fn id(&self) -> i32;
}

macro_rules! impl_has_id {
    ($($t:ty),+ $(,)?) => {
        $(
            impl HasId for $t {
                fn id(&self) -> i32 {
                    self.id
                }
            }
        )+
    };
}

// Records that carry both a note and an id; these are the ones whose notes the
// cache indexes.
impl_has_id!(
    Actor, Class, Skill, Item, Weapon, Armor, Enemy, State, Tileset
);

macro_rules! impl_has_note {
    ($($t:ty),+ $(,)?) => {
        $(
            impl HasNote for $t {
                fn note(&self) -> &str {
                    self.note.as_str()
                }
            }
        )+
    };
}

impl_has_note!(
    Actor, Class, Skill, Item, Weapon, Armor, Enemy, State, Tileset, Map, MapEvent
);
