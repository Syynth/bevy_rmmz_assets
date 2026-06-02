//! Optional `Map###.json` loading and map note metadata (feature `maps`).
//!
//! Maps are loaded on request (or eagerly from `MapInfos`) into [`RmmzMaps`],
//! and read back through [`RmmzDatabase`](crate::database::RmmzDatabase). Map and
//! map-event notes are parsed once per map load into [`RmmzMapNotes`].
//!
//! Everything here is off unless you call
//! [`RmmzAppExt::enable_rmmz_maps`](crate::ext::RmmzAppExt::enable_rmmz_maps).

use std::collections::{HashMap, HashSet};

use bevy_asset::{AssetEvent, AssetServer, Assets, Handle};
use bevy_ecs::prelude::{Local, MessageReader, Res, ResMut, Resource};

use crate::asset::{MapAsset, MapInfosAsset};
use crate::config::{RmmzConfig, RmmzHandles};
use crate::data::HasNote;
use crate::notes::{NoteRegistry, NoteTokens, ParsedNote};

/// How map files are loaded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MapLoad {
    /// Maps are not loaded; [`RmmzMaps::request`] is ignored.
    #[default]
    None,
    /// Every map listed in `MapInfos.json` is loaded once it is available.
    Eager,
    /// Maps load only when [`RmmzMaps::request`]ed.
    OnDemand,
}

/// Tracks requested and loaded map assets.
///
/// Request a map with [`RmmzMaps::request`] (a plain resource mutation); a system
/// performs the actual load and records the handle. Off unless the strategy is
/// `Eager`/`OnDemand` (see [`RmmzAppExt::enable_rmmz_maps`](crate::ext::RmmzAppExt::enable_rmmz_maps)).
#[derive(Resource, Default)]
pub struct RmmzMaps {
    pub(crate) strategy: MapLoad,
    requested: HashSet<i32>,
    pub(crate) handles: HashMap<i32, Handle<MapAsset>>,
}

impl RmmzMaps {
    /// Requests that the map with this 1-based id be loaded (no-op under
    /// [`MapLoad::None`]). Idempotent.
    pub fn request(&mut self, id: i32) {
        self.requested.insert(id);
    }

    /// The handle for a requested/loaded map, if any.
    pub fn handle(&self, id: i32) -> Option<&Handle<MapAsset>> {
        self.handles.get(&id)
    }

    /// Whether the map with this id has been requested and its handle recorded.
    pub fn is_requested(&self, id: i32) -> bool {
        self.handles.contains_key(&id) || self.requested.contains(&id)
    }

    /// The configured loading strategy.
    pub fn strategy(&self) -> MapLoad {
        self.strategy
    }
}

/// Parsed note metadata for maps and map events, keyed by map id and
/// `(map id, event id)`.
#[derive(Resource, Default)]
pub struct RmmzMapNotes {
    maps: HashMap<i32, ParsedNote>,
    events: HashMap<(i32, i32), ParsedNote>,
}

impl RmmzMapNotes {
    /// Parsed note metadata for the given map.
    pub fn map(&self, map_id: i32) -> Option<&ParsedNote> {
        self.maps.get(&map_id)
    }

    /// Parsed note metadata for an event within a map.
    pub fn event(&self, map_id: i32, event_id: i32) -> Option<&ParsedNote> {
        self.events.get(&(map_id, event_id))
    }
}

/// Loads any requested-but-unloaded maps via the asset server.
pub(crate) fn load_requested_maps(
    mut maps: ResMut<RmmzMaps>,
    config: Res<RmmzConfig>,
    server: Res<AssetServer>,
) {
    if maps.strategy == MapLoad::None {
        return;
    }
    let pending: Vec<i32> = maps
        .requested
        .iter()
        .copied()
        .filter(|id| !maps.handles.contains_key(id))
        .collect();
    for id in pending {
        let handle = server.load(config.path(&format!("Map{id:03}.json")));
        maps.handles.insert(id, handle);
    }
}

/// Under [`MapLoad::Eager`], requests every map listed in `MapInfos` once it
/// loads (runs to completion once).
pub(crate) fn eager_request_maps(
    mut maps: ResMut<RmmzMaps>,
    handles: Res<RmmzHandles>,
    map_infos: Res<Assets<MapInfosAsset>>,
    mut done: Local<bool>,
) {
    if *done || maps.strategy != MapLoad::Eager {
        return;
    }
    let Some(infos) = handles.map_infos.as_ref().and_then(|h| map_infos.get(h)) else {
        return;
    };
    for info in infos.iter() {
        maps.request(info.id);
    }
    *done = true;
}

/// Parses map + map-event notes whenever a map asset is added or modified.
pub(crate) fn cache_map_notes(
    mut events: MessageReader<AssetEvent<MapAsset>>,
    maps: Res<RmmzMaps>,
    assets: Res<Assets<MapAsset>>,
    registry: Res<NoteRegistry>,
    mut cache: ResMut<RmmzMapNotes>,
) {
    for event in events.read() {
        let (asset_id, removed) = match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => (*id, false),
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => (*id, true),
        };

        // Which map does this asset belong to?
        let Some(map_id) = maps
            .handles
            .iter()
            .find_map(|(id, handle)| (handle.id() == asset_id).then_some(*id))
        else {
            continue;
        };

        // Drop this map's stale entries, then re-cache just this map (not all).
        cache.maps.remove(&map_id);
        cache.events.retain(|(m, _), _| *m != map_id);
        if removed {
            continue;
        }
        let Some(asset) = assets.get(asset_id) else {
            continue;
        };
        let map = &asset.0;

        let parsed = registry.parse_all(&NoteTokens::parse(map.note()));
        if !parsed.is_empty() {
            cache.maps.insert(map_id, parsed);
        }
        for event in map.events.iter().flatten() {
            let parsed = registry.parse_all(&NoteTokens::parse(event.note()));
            if !parsed.is_empty() {
                cache.events.insert((map_id, event.id), parsed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin, Update};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin};
    use bevy_ecs::prelude::{ResMut, Resource};
    use serde::{Deserialize, Serialize};

    use super::MapLoad;
    use crate::config::{CoreTable, RmmzConfig};
    use crate::database::RmmzDatabase;
    use crate::ext::RmmzAppExt;
    use crate::notes::{NoteParser, NoteTokens};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Biome(String);
    struct BiomeParser;
    impl NoteParser for BiomeParser {
        type Output = Biome;
        const TAG: &'static str = "biome";
        fn parse(&self, tokens: &NoteTokens) -> Option<Biome> {
            tokens.value("biome").map(|v| Biome(v.to_owned()))
        }
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Chest(String);
    struct ChestParser;
    impl NoteParser for ChestParser {
        type Output = Chest;
        const TAG: &'static str = "chest";
        fn parse(&self, tokens: &NoteTokens) -> Option<Chest> {
            tokens.value("chest").map(|v| Chest(v.to_owned()))
        }
    }

    #[derive(Resource, Default)]
    struct Probe {
        map_name: Option<String>,
        biome: Option<String>,
        chest: Option<String>,
        ids: Vec<i32>,
    }

    fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
        out.ids = db.map_ids();
        if let Some(map) = db.map(1) {
            out.map_name = Some(map.display_name.clone());
            out.biome = db.map_note::<Biome>(1).map(|b| b.0.clone());
            out.chest = db.map_event_note::<Chest>(1, 1).map(|c| c.0.clone());
        }
    }

    #[test]
    fn eager_loads_maps_and_parses_map_and_event_notes() {
        let dir = Dir::default();
        dir.insert_asset_text(
            Path::new("data/MapInfos.json"),
            r#"[null,{"id":1,"name":"Town"}]"#,
        );
        dir.insert_asset_text(
            Path::new("data/Map001.json"),
            r#"{"displayName":"Town","note":"<biome:forest>",
                "events":[null,{"id":1,"name":"Chest","note":"<chest:gold>"}]}"#,
        );
        let reader_dir = dir.clone();

        let mut app = App::new();
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(MemoryAssetReader {
                    root: reader_dir.clone(),
                })
            }),
        )
        .add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin {
                watch_for_changes_override: Some(false),
                ..Default::default()
            },
        ))
        .init_resource::<Probe>()
        .register_note_parser(BiomeParser)
        .register_note_parser(ChestParser)
        .add_rmmz_with(RmmzConfig::default().with_tables([CoreTable::MapInfos]))
        .enable_rmmz_maps(MapLoad::Eager)
        .add_systems(Update, probe);

        let mut chest = None;
        for _ in 0..2000 {
            app.update();
            chest = app.world().resource::<Probe>().chest.clone();
            if chest.is_some() {
                break;
            }
        }

        let probe = app.world().resource::<Probe>();
        assert_eq!(probe.ids, vec![1], "MapInfos should list map 1");
        assert_eq!(probe.map_name.as_deref(), Some("Town"), "map 1 should load");
        assert_eq!(probe.biome.as_deref(), Some("forest"), "map note parsed");
        assert_eq!(chest.as_deref(), Some("gold"), "event note parsed");
    }
}
