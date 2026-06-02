//! The crate's Bevy plugin.

use bevy_app::{App, Plugin, Update};
use bevy_asset::AssetApp;
use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::asset::RmmzAsset;
use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset,
    TroopsAsset, WeaponsAsset,
};
use crate::config::RmmzRegistry;
use crate::data::{Actor, Armor, Class, Enemy, Item, Skill, State, Tileset, Weapon};
use crate::database::{RmmzLoadStatus, load_status_unsettled, track_load_status};
use crate::loader::RmmzJsonLoader;
use crate::notes::{NoteRegistry, RmmzNoteCache, cache_table_notes};

/// Wires RPG Maker MZ database loading into a Bevy [`App`].
///
/// Registers every database asset type ([`ActorsAsset`], [`ItemsAsset`], …,
/// [`SystemAsset`], [`MapAsset`]) together with a JSON loader for each, plus the
/// note-parser registry and the parse-once note cache.
///
/// Requires Bevy's `AssetPlugin` (included in `DefaultPlugins`) to be added
/// **before** this plugin.
#[derive(Debug, Default, Clone, Copy)]
pub struct RmmzAssetsPlugin;

impl Plugin for RmmzAssetsPlugin {
    fn build(&self, app: &mut App) {
        register::<ActorsAsset>(app);
        register::<ClassesAsset>(app);
        register::<SkillsAsset>(app);
        register::<ItemsAsset>(app);
        register::<WeaponsAsset>(app);
        register::<ArmorsAsset>(app);
        register::<EnemiesAsset>(app);
        register::<StatesAsset>(app);
        register::<TroopsAsset>(app);
        register::<AnimationsAsset>(app);
        register::<TilesetsAsset>(app);
        register::<CommonEventsAsset>(app);
        register::<MapInfosAsset>(app);
        register::<SystemAsset>(app);
        register::<MapAsset>(app);

        app.init_resource::<NoteRegistry>()
            .init_resource::<RmmzNoteCache>()
            .init_resource::<RmmzLoadStatus>()
            .init_resource::<RmmzRegistry>();

        // Latch the aggregate load status once it settles, so steady-state
        // `RmmzDatabase::status`/`ready` calls don't re-poll every handle. The
        // run condition stops the tracker from running at all once settled.
        app.add_systems(Update, track_load_status.run_if(load_status_unsettled));

        // Parse each note-bearing table's notes once on load (and on reload),
        // caching the typed metadata.
        app.add_systems(
            Update,
            (
                cache_table_notes::<Actor>,
                cache_table_notes::<Class>,
                cache_table_notes::<Skill>,
                cache_table_notes::<Item>,
                cache_table_notes::<Weapon>,
                cache_table_notes::<Armor>,
                cache_table_notes::<Enemy>,
                cache_table_notes::<State>,
                cache_table_notes::<Tileset>,
            ),
        );
    }
}

/// Registers an asset type and its JSON loader.
fn register<A: RmmzAsset>(app: &mut App) {
    app.init_asset::<A>()
        .register_asset_loader(RmmzJsonLoader::<A>::default());
}
