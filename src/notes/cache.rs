//! Parse-once cache of typed note metadata.
//!
//! Notes are parsed exactly once per record when a table loads (and again on
//! hot-reload), and the results are cached here — so reads never re-parse, even
//! in tight loops. The cache is keyed by record type and 1-based id.

use std::any::TypeId;
use std::collections::HashMap;

use bevy_asset::{AssetEvent, Assets};
use bevy_ecs::prelude::{MessageReader, Res, ResMut, Resource};
use bevy_reflect::TypePath;

use crate::asset::Table;
use crate::data::{HasId, HasNote};
use crate::notes::{NoteRegistry, NoteTokens, ParsedNote};

/// Caches the typed metadata parsed from each record's note, keyed by record
/// type and 1-based id.
///
/// Populated once per load (and re-populated on hot-reload) by
/// [`cache_table_notes`], so reads never re-parse. Look metadata up through
/// [`RmmzDatabase`](crate::database::RmmzDatabase).
#[derive(Resource, Default)]
pub struct RmmzNoteCache {
    by_record: HashMap<(TypeId, i32), ParsedNote>,
}

impl RmmzNoteCache {
    /// The parsed note metadata for record type `R` with the given 1-based id.
    pub fn get<R: 'static>(&self, id: i32) -> Option<&ParsedNote> {
        self.by_record.get(&(TypeId::of::<R>(), id))
    }

    /// Drops all cached entries for record type `R`.
    fn clear_type<R: 'static>(&mut self) {
        let ty = TypeId::of::<R>();
        self.by_record.retain(|(t, _), _| *t != ty);
    }
}

/// System that (re)builds the note cache for the table of record type `R` when
/// its asset is added or modified. Registered once per note-bearing table.
pub fn cache_table_notes<R>(
    mut events: MessageReader<AssetEvent<Table<R>>>,
    tables: Res<Assets<Table<R>>>,
    registry: Res<NoteRegistry>,
    mut cache: ResMut<RmmzNoteCache>,
) where
    R: HasNote + HasId + TypePath + Send + Sync + 'static,
{
    let dirty = events.read().any(|event| {
        matches!(
            event,
            AssetEvent::Added { .. }
                | AssetEvent::Modified { .. }
                | AssetEvent::LoadedWithDependencies { .. }
        )
    });
    if !dirty {
        return;
    }

    cache.clear_type::<R>();
    for (_, table) in tables.iter() {
        for record in table.iter() {
            let parsed = registry.parse_all(&NoteTokens::parse(record.note()));
            if !parsed.is_empty() {
                cache
                    .by_record
                    .insert((TypeId::of::<R>(), record.id()), parsed);
            }
        }
    }
}
