//! The [`RmmzAppExt`] convenience trait for wiring up loading.

use bevy_app::{App, Startup, Update};
use bevy_asset::{AssetApp, AssetServer};
use bevy_ecs::prelude::{Res, ResMut};
use bevy_reflect::TypePath;

use crate::RmmzAssetsPlugin;
use crate::asset::{RmmzAsset, SystemAsset, Table};
use crate::config::{CoreTable, RmmzConfig, RmmzRegistry};
use crate::data::{
    Actor, Animation, Armor, Class, CommonEvent, Enemy, HasId, HasNote, Item, MapInfo, Skill,
    State, Tileset, Troop, Weapon,
};
use crate::database::RmmzFetch;
use crate::loader::RmmzJsonLoader;
use crate::notes::{NoteParser, NoteRegistry, cache_table_notes};
use crate::snapshot::snapshot_asset;

/// Convenience methods on [`App`] for setting up RPG Maker MZ loading.
///
/// These build on [`RmmzAssetsPlugin`] (asset/loader registration) by also
/// inserting a [`RmmzConfig`] and a startup system that kicks off loading the
/// configured tables registered in the
/// [`RmmzRegistry`](crate::config::RmmzRegistry).
///
/// Bevy's `AssetPlugin` (part of `DefaultPlugins`) must be added first.
pub trait RmmzAppExt {
    /// Registers everything and loads all core tables from the default
    /// `data/` directory.
    fn add_rmmz(&mut self) -> &mut Self;

    /// Registers everything and loads according to `config`.
    fn add_rmmz_with(&mut self, config: RmmzConfig) -> &mut Self;

    /// Registers a [`NoteParser`]. Its output becomes available through the
    /// cached note-metadata accessors. Register parsers before loading so the
    /// note cache includes them.
    fn register_note_parser<P: NoteParser>(&mut self, parser: P) -> &mut Self;

    /// Registers a custom single-document asset type `A`, loaded from `file`
    /// (relative to the data path) and reachable via `db.asset::<A>()`.
    ///
    /// Implement the required traits with [`rmmz_asset!`](crate::rmmz_asset).
    /// Call **after** [`Self::add_rmmz`] / [`Self::add_rmmz_with`], which install
    /// the startup loader that drives loading.
    fn register_rmmz<A>(&mut self, file: impl Into<String>) -> &mut Self
    where
        A: RmmzAsset + RmmzFetch + Clone;

    /// Enables map loading with the given strategy (default is off). Call after
    /// [`Self::add_rmmz`]. Under [`MapLoad::Eager`](crate::maps::MapLoad::Eager),
    /// `MapInfos` must be among the loaded tables.
    #[cfg(feature = "maps")]
    fn enable_rmmz_maps(&mut self, strategy: crate::maps::MapLoad) -> &mut Self;
}

impl RmmzAppExt for App {
    fn add_rmmz(&mut self) -> &mut Self {
        self.add_rmmz_with(RmmzConfig::default())
    }

    fn add_rmmz_with(&mut self, config: RmmzConfig) -> &mut Self {
        self.add_plugins(RmmzAssetsPlugin);

        // Register the selected built-in tables (registry entries + note-cache
        // systems for the note-bearing ones), then load everything at startup.
        register_builtins(self, &config);
        self.insert_resource(config)
            .add_systems(Startup, load_registered);

        #[cfg(feature = "maps")]
        {
            use crate::maps::{
                RmmzMapNotes, RmmzMaps, cache_map_notes, eager_request_maps, load_requested_maps,
            };
            self.init_resource::<RmmzMaps>()
                .init_resource::<RmmzMapNotes>()
                .add_systems(
                    Update,
                    (eager_request_maps, load_requested_maps, cache_map_notes),
                );
        }

        self
    }

    fn register_note_parser<P: NoteParser>(&mut self, parser: P) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<NoteRegistry>()
            .register(parser);
        self
    }

    fn register_rmmz<A>(&mut self, file: impl Into<String>) -> &mut Self
    where
        A: RmmzAsset + RmmzFetch + Clone,
    {
        self.init_asset::<A>()
            .register_asset_loader(RmmzJsonLoader::<A>::default())
            .add_systems(Update, snapshot_asset::<A>);
        {
            let mut registry = self.world_mut().get_resource_or_init::<RmmzRegistry>();
            registry.register::<A>(file);
        }
        self
    }

    #[cfg(feature = "maps")]
    fn enable_rmmz_maps(&mut self, strategy: crate::maps::MapLoad) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<crate::maps::RmmzMaps>()
            .strategy = strategy;
        self
    }
}

/// Registers the selected built-in tables (at app-build time): a registry entry
/// for each, plus a note-cache system for the note-bearing ones. Files resolve
/// under [`RmmzConfig::data_path`] when loaded.
fn register_builtins(app: &mut App, config: &RmmzConfig) {
    if config.loads(CoreTable::Actors) {
        note_table::<Actor>(app, "Actors.json");
    }
    if config.loads(CoreTable::Classes) {
        note_table::<Class>(app, "Classes.json");
    }
    if config.loads(CoreTable::Skills) {
        note_table::<Skill>(app, "Skills.json");
    }
    if config.loads(CoreTable::Items) {
        note_table::<Item>(app, "Items.json");
    }
    if config.loads(CoreTable::Weapons) {
        note_table::<Weapon>(app, "Weapons.json");
    }
    if config.loads(CoreTable::Armors) {
        note_table::<Armor>(app, "Armors.json");
    }
    if config.loads(CoreTable::Enemies) {
        note_table::<Enemy>(app, "Enemies.json");
    }
    if config.loads(CoreTable::States) {
        note_table::<State>(app, "States.json");
    }
    if config.loads(CoreTable::Tilesets) {
        note_table::<Tileset>(app, "Tilesets.json");
    }
    if config.loads(CoreTable::Troops) {
        data_table::<Troop>(app, "Troops.json");
    }
    if config.loads(CoreTable::Animations) {
        data_table::<Animation>(app, "Animations.json");
    }
    if config.loads(CoreTable::CommonEvents) {
        data_table::<CommonEvent>(app, "CommonEvents.json");
    }
    if config.loads(CoreTable::MapInfos) {
        data_table::<MapInfo>(app, "MapInfos.json");
    }
    if config.loads(CoreTable::System) {
        app.world_mut()
            .get_resource_or_init::<RmmzRegistry>()
            .register::<SystemAsset>("System.json");
    }
}

/// Registers a note-bearing table: a registry entry for `Table<R>` plus the
/// parse-once note-cache system. The loader is installed by [`RmmzAssetsPlugin`]
/// for built-ins.
fn note_table<R>(app: &mut App, file: &str)
where
    R: HasNote + HasId + TypePath + Send + Sync + 'static,
{
    app.world_mut()
        .get_resource_or_init::<RmmzRegistry>()
        .register::<Table<R>>(file);
    app.add_systems(Update, cache_table_notes::<R>);
}

/// Registers a plain (no indexed notes) table's registry entry.
fn data_table<R>(app: &mut App, file: &str)
where
    R: TypePath + Send + Sync + 'static,
{
    app.world_mut()
        .get_resource_or_init::<RmmzRegistry>()
        .register::<Table<R>>(file);
}

/// Startup system: loads every registered table, recording its handle.
fn load_registered(
    mut registry: ResMut<RmmzRegistry>,
    config: Res<RmmzConfig>,
    server: Res<AssetServer>,
) {
    registry.load_all(&server, &config);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin, Assets};

    use super::RmmzAppExt;
    use crate::asset::{ActorsAsset, ItemsAsset, SystemAsset};
    use crate::config::{CoreTable, RmmzConfig, RmmzRegistry};

    fn app_with(files: &[(&str, &str)]) -> App {
        let dir = Dir::default();
        for (path, contents) in files {
            dir.insert_asset_text(Path::new(path), contents);
        }
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
        ));
        app
    }

    #[test]
    fn add_rmmz_with_loads_configured_tables() {
        let mut app = app_with(&[
            ("game/Items.json", r#"[null,{"id":1,"name":"Potion"}]"#),
            ("game/System.json", r#"{"gameTitle":"Demo"}"#),
        ]);
        app.add_rmmz_with(RmmzConfig::new("game"));

        let items = run_until_loaded::<ItemsAsset>(&mut app, RmmzRegistry::handle::<ItemsAsset>)
            .expect("Items.json should load");
        assert_eq!(items_name(&app, &items), "Potion");

        // System loaded too, via the same config.
        assert!(
            app.world()
                .resource::<RmmzRegistry>()
                .handle::<SystemAsset>()
                .is_some()
        );
    }

    #[test]
    fn table_selection_only_loads_requested() {
        let mut app = app_with(&[("data/Items.json", r#"[null,{"id":1,"name":"Potion"}]"#)]);
        app.add_rmmz_with(RmmzConfig::default().with_tables([CoreTable::Items]));

        // Pump a few frames so the startup system runs.
        for _ in 0..5 {
            app.update();
        }
        let registry = app.world().resource::<RmmzRegistry>();
        assert!(registry.handle::<ItemsAsset>().is_some());
        assert!(registry.handle::<ActorsAsset>().is_none());
        assert!(registry.handle::<SystemAsset>().is_none());
    }

    #[test]
    fn register_rmmz_loads_and_exposes_a_custom_singleton() {
        use std::collections::HashMap;

        use bevy_app::Update;
        use bevy_asset::Asset;
        use bevy_ecs::prelude::{ResMut, Resource};
        use bevy_reflect::TypePath;
        use serde::{Deserialize, Serialize};

        use crate::database::RmmzDatabase;

        // A string-keyed config map (the real codetta `AnimationMap.json` shape).
        #[derive(Asset, TypePath, Serialize, Deserialize, Clone, Default)]
        #[serde(transparent)]
        struct AnimationMap(HashMap<String, String>);
        crate::rmmz_asset!(AnimationMap);

        #[derive(Resource, Default)]
        struct Probe {
            present: bool,
            value: Option<String>,
        }
        fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
            if let Some(map) = db.asset::<AnimationMap>() {
                out.present = true;
                out.value = map.0.get("furnitureBreak").cloned();
            }
        }

        let mut app = app_with(&[(
            "data/AnimationMap.json",
            r#"{"furnitureBreak":"break","test":"x"}"#,
        )]);
        app.init_resource::<Probe>()
            .add_rmmz_with(RmmzConfig::default().with_tables([])) // no built-ins
            .register_rmmz::<AnimationMap>("AnimationMap.json")
            .add_systems(Update, probe);

        let mut present = false;
        for _ in 0..1000 {
            app.update();
            if app.world().resource::<Probe>().present {
                present = true;
                break;
            }
        }
        assert!(present, "custom asset was never snapshotted");
        assert_eq!(
            app.world().resource::<Probe>().value.as_deref(),
            Some("break")
        );
    }

    fn run_until_loaded<A: bevy_asset::Asset>(
        app: &mut App,
        pick: impl Fn(&RmmzRegistry) -> Option<bevy_asset::Handle<A>>,
    ) -> Option<bevy_asset::Handle<A>> {
        for _ in 0..1000 {
            app.update();
            if let Some(handle) = app.world().get_resource::<RmmzRegistry>().and_then(&pick)
                && app.world().resource::<Assets<A>>().get(&handle).is_some()
            {
                return Some(handle);
            }
        }
        None
    }

    fn items_name(app: &App, handle: &bevy_asset::Handle<ItemsAsset>) -> String {
        app.world()
            .resource::<Assets<ItemsAsset>>()
            .get(handle)
            .unwrap()
            .get(1)
            .unwrap()
            .name
            .clone()
    }
}
