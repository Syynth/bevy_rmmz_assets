//! The [`RmmzAppExt`] convenience trait for wiring up loading.

use bevy_app::{App, Startup};
use bevy_asset::AssetServer;
use bevy_ecs::prelude::{Commands, Res};

use crate::RmmzAssetsPlugin;
use crate::config::{CoreTable, RmmzConfig, RmmzHandles};

/// Convenience methods on [`App`] for setting up RPG Maker MZ loading.
///
/// These build on [`RmmzAssetsPlugin`] (asset/loader registration) by also
/// inserting a [`RmmzConfig`] and a startup system that kicks off loading the
/// configured tables into [`RmmzHandles`].
///
/// Bevy's `AssetPlugin` (part of `DefaultPlugins`) must be added first.
pub trait RmmzAppExt {
    /// Registers everything and loads all core tables from the default
    /// `data/` directory.
    fn add_rmmz(&mut self) -> &mut Self;

    /// Registers everything and loads according to `config`.
    fn add_rmmz_with(&mut self, config: RmmzConfig) -> &mut Self;
}

impl RmmzAppExt for App {
    fn add_rmmz(&mut self) -> &mut Self {
        self.add_rmmz_with(RmmzConfig::default())
    }

    fn add_rmmz_with(&mut self, config: RmmzConfig) -> &mut Self {
        self.add_plugins(RmmzAssetsPlugin)
            .insert_resource(config)
            .add_systems(Startup, load_core_tables)
    }
}

/// Startup system: loads the configured core tables and records their handles.
fn load_core_tables(mut commands: Commands, config: Res<RmmzConfig>, server: Res<AssetServer>) {
    let mut handles = RmmzHandles::default();

    if config.loads(CoreTable::Actors) {
        handles.actors = Some(server.load(config.path("Actors.json")));
    }
    if config.loads(CoreTable::Classes) {
        handles.classes = Some(server.load(config.path("Classes.json")));
    }
    if config.loads(CoreTable::Skills) {
        handles.skills = Some(server.load(config.path("Skills.json")));
    }
    if config.loads(CoreTable::Items) {
        handles.items = Some(server.load(config.path("Items.json")));
    }
    if config.loads(CoreTable::Weapons) {
        handles.weapons = Some(server.load(config.path("Weapons.json")));
    }
    if config.loads(CoreTable::Armors) {
        handles.armors = Some(server.load(config.path("Armors.json")));
    }
    if config.loads(CoreTable::Enemies) {
        handles.enemies = Some(server.load(config.path("Enemies.json")));
    }
    if config.loads(CoreTable::States) {
        handles.states = Some(server.load(config.path("States.json")));
    }
    if config.loads(CoreTable::Troops) {
        handles.troops = Some(server.load(config.path("Troops.json")));
    }
    if config.loads(CoreTable::Animations) {
        handles.animations = Some(server.load(config.path("Animations.json")));
    }
    if config.loads(CoreTable::Tilesets) {
        handles.tilesets = Some(server.load(config.path("Tilesets.json")));
    }
    if config.loads(CoreTable::CommonEvents) {
        handles.common_events = Some(server.load(config.path("CommonEvents.json")));
    }
    if config.loads(CoreTable::MapInfos) {
        handles.map_infos = Some(server.load(config.path("MapInfos.json")));
    }
    if config.loads(CoreTable::System) {
        handles.system = Some(server.load(config.path("System.json")));
    }

    commands.insert_resource(handles);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin, Assets};

    use super::RmmzAppExt;
    use crate::asset::ItemsAsset;
    use crate::config::{CoreTable, RmmzConfig, RmmzHandles};

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

        let items = run_until_loaded::<ItemsAsset>(&mut app, |h| h.items.clone())
            .expect("Items.json should load");
        assert_eq!(items_name(&app, &items), "Potion");

        // System loaded too, via the same config.
        assert!(app.world().resource::<RmmzHandles>().system.is_some());
    }

    #[test]
    fn table_selection_only_loads_requested() {
        let mut app = app_with(&[("data/Items.json", r#"[null,{"id":1,"name":"Potion"}]"#)]);
        app.add_rmmz_with(RmmzConfig::default().with_tables([CoreTable::Items]));

        // Pump a few frames so the startup system runs.
        for _ in 0..5 {
            app.update();
        }
        let handles = app.world().resource::<RmmzHandles>();
        assert!(handles.items.is_some());
        assert!(handles.actors.is_none());
        assert!(handles.system.is_none());
    }

    fn run_until_loaded<A: bevy_asset::Asset>(
        app: &mut App,
        pick: impl Fn(&RmmzHandles) -> Option<bevy_asset::Handle<A>>,
    ) -> Option<bevy_asset::Handle<A>> {
        for _ in 0..1000 {
            app.update();
            if let Some(handle) = app.world().get_resource::<RmmzHandles>().and_then(&pick)
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
