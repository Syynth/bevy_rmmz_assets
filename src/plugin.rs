//! The crate's Bevy plugin.

use bevy_app::{App, Plugin};
use bevy_asset::{Asset, AssetApp};
use serde::de::DeserializeOwned;

use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset,
    TroopsAsset, WeaponsAsset,
};
use crate::loader::RmmzJsonLoader;

/// Wires RPG Maker MZ database loading into a Bevy [`App`].
///
/// Registers every database asset type ([`ActorsAsset`], [`ItemsAsset`], …,
/// [`SystemAsset`], [`MapAsset`]) together with a JSON loader for each, so they
/// can be loaded with `asset_server.load::<ActorsAsset>("Actors.json")`.
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
    }
}

/// Registers an asset type and its JSON loader.
fn register<A: Asset + DeserializeOwned>(app: &mut App) {
    app.init_asset::<A>()
        .register_asset_loader(RmmzJsonLoader::<A>::default());
}
