//! The [`RmmzAppExt`] convenience trait for wiring up loading.

#[cfg(feature = "maps")]
use bevy_app::Update;
use bevy_app::{App, Startup};
use bevy_asset::AssetServer;
use bevy_ecs::prelude::{Res, ResMut};

use crate::RmmzAssetsPlugin;
use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset, TroopsAsset,
    WeaponsAsset,
};
use crate::config::{CoreTable, RmmzConfig, RmmzRegistry};
use crate::notes::{NoteParser, NoteRegistry};

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

        // Register the selected built-in tables into the (plugin-initialized)
        // registry, then load everything registered at startup.
        {
            let mut registry = self.world_mut().get_resource_or_init::<RmmzRegistry>();
            register_builtins(&mut registry, &config);
        }
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

    #[cfg(feature = "maps")]
    fn enable_rmmz_maps(&mut self, strategy: crate::maps::MapLoad) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<crate::maps::RmmzMaps>()
            .strategy = strategy;
        self
    }
}

/// Registers the selected built-in tables into the registry (at app-build time).
/// Files resolve under [`RmmzConfig::data_path`] when loaded.
fn register_builtins(registry: &mut RmmzRegistry, config: &RmmzConfig) {
    if config.loads(CoreTable::Actors) {
        registry.register::<ActorsAsset>("Actors.json");
    }
    if config.loads(CoreTable::Classes) {
        registry.register::<ClassesAsset>("Classes.json");
    }
    if config.loads(CoreTable::Skills) {
        registry.register::<SkillsAsset>("Skills.json");
    }
    if config.loads(CoreTable::Items) {
        registry.register::<ItemsAsset>("Items.json");
    }
    if config.loads(CoreTable::Weapons) {
        registry.register::<WeaponsAsset>("Weapons.json");
    }
    if config.loads(CoreTable::Armors) {
        registry.register::<ArmorsAsset>("Armors.json");
    }
    if config.loads(CoreTable::Enemies) {
        registry.register::<EnemiesAsset>("Enemies.json");
    }
    if config.loads(CoreTable::States) {
        registry.register::<StatesAsset>("States.json");
    }
    if config.loads(CoreTable::Troops) {
        registry.register::<TroopsAsset>("Troops.json");
    }
    if config.loads(CoreTable::Animations) {
        registry.register::<AnimationsAsset>("Animations.json");
    }
    if config.loads(CoreTable::Tilesets) {
        registry.register::<TilesetsAsset>("Tilesets.json");
    }
    if config.loads(CoreTable::CommonEvents) {
        registry.register::<CommonEventsAsset>("CommonEvents.json");
    }
    if config.loads(CoreTable::MapInfos) {
        registry.register::<MapInfosAsset>("MapInfos.json");
    }
    if config.loads(CoreTable::System) {
        registry.register::<SystemAsset>("System.json");
    }
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
