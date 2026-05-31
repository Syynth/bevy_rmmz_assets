//! The [`HasNote`] trait, implemented by records carrying an author `note`.

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
