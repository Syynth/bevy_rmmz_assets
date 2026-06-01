//! Ahead-of-time binary processing of the database (feature `process`).
//!
//! Bevy's asset processor can convert each `data/*.json` file into a compact
//! [`postcard`] binary at build time, so release builds skip JSON parsing
//! entirely. This module supplies the pieces:
//!
//! - [`RmmzBinSaver`] — an [`AssetSaver`] that serializes an asset to postcard.
//! - [`RmmzBinLoader`] — an [`AssetLoader`] that loads the processed binary back.
//! - [`RmmzBinProcessor`] — the [`Process`](bevy_asset::processor::Process) tying
//!   JSON-in to binary-out (`LoadTransformAndSave<RmmzJsonLoader, _, RmmzBinSaver>`).
//!
//! **Note parsing is intentionally not part of processing.** The saver writes the
//! records verbatim — note strings included — so note metadata is parsed the
//! same way whether the data came from JSON (dev) or the processed binary
//! (release). Processing is purely an ahead-of-time *data* transform; it does no
//! on-demand work.
//!
//! ## Opting in
//!
//! Because every RPG Maker file shares the `.json` extension, you cannot select
//! one global default processor for them (each asset type needs its own
//! [`RmmzBinProcessor<A>`]). [`RmmzProcessingExt::register_rmmz_processing`]
//! registers all of them; enable processing per file with an asset `.meta` that
//! names the matching processor, and run with the `asset_processor` feature.

use core::marker::PhantomData;

use bevy_app::App;
use bevy_asset::io::{AsyncWriteExt, Reader, Writer};
use bevy_asset::processor::LoadTransformAndSave;
use bevy_asset::saver::{AssetSaver, SavedAsset};
use bevy_asset::transformer::IdentityAssetTransformer;
use bevy_asset::{Asset, AssetApp, AssetLoader, AssetPath, LoadContext};
use bevy_reflect::TypePath;
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset,
    TroopsAsset, WeaponsAsset,
};
use crate::loader::RmmzJsonLoader;

/// Errors produced by the binary saver/loader.
#[derive(Debug, Error)]
pub enum RmmzBinError {
    /// The bytes could not be read or written.
    #[error("failed to read/write processed RPG Maker MZ asset: {0}")]
    Io(#[from] std::io::Error),
    /// The asset could not be (de)serialized from the binary form.
    #[error("failed to (de)serialize processed RPG Maker MZ asset: {0}")]
    Postcard(#[from] postcard::Error),
}

/// Loads a processed (postcard binary) RPG Maker MZ asset of type `A`.
#[derive(TypePath)]
pub struct RmmzBinLoader<A>(PhantomData<fn() -> A>);

impl<A> Default for RmmzBinLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A> AssetLoader for RmmzBinLoader<A>
where
    A: Asset + DeserializeOwned,
{
    type Asset = A;
    type Settings = ();
    type Error = RmmzBinError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<A, RmmzBinError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(postcard::from_bytes(&bytes)?)
    }
}

/// Saves an RPG Maker MZ asset of type `A` as a compact postcard binary, to be
/// read back by [`RmmzBinLoader`].
#[derive(TypePath)]
pub struct RmmzBinSaver<A>(PhantomData<fn() -> A>);

impl<A> Default for RmmzBinSaver<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A> AssetSaver for RmmzBinSaver<A>
where
    A: Asset + Serialize + DeserializeOwned,
{
    type Asset = A;
    type Settings = ();
    type OutputLoader = RmmzBinLoader<A>;
    type Error = RmmzBinError;

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, '_, A>,
        _settings: &Self::Settings,
        _asset_path: AssetPath<'_>,
    ) -> Result<(), RmmzBinError> {
        let bytes = postcard::to_stdvec(asset.get())?;
        writer.write_all(&bytes).await?;
        Ok(())
    }
}

/// The [`Process`](bevy_asset::processor::Process) converting a `data/*.json`
/// file into the processed postcard binary form for asset type `A`.
pub type RmmzBinProcessor<A> =
    LoadTransformAndSave<RmmzJsonLoader<A>, IdentityAssetTransformer<A>, RmmzBinSaver<A>>;

/// App extension registering the binary processing pipeline.
pub trait RmmzProcessingExt {
    /// Registers a binary loader and a JSON→binary processor for every core
    /// asset type. Has effect only when the `asset_processor` feature is active.
    fn register_rmmz_processing(&mut self) -> &mut Self;
}

impl RmmzProcessingExt for App {
    fn register_rmmz_processing(&mut self) -> &mut Self {
        register::<ActorsAsset>(self);
        register::<ClassesAsset>(self);
        register::<SkillsAsset>(self);
        register::<ItemsAsset>(self);
        register::<WeaponsAsset>(self);
        register::<ArmorsAsset>(self);
        register::<EnemiesAsset>(self);
        register::<StatesAsset>(self);
        register::<TroopsAsset>(self);
        register::<AnimationsAsset>(self);
        register::<TilesetsAsset>(self);
        register::<CommonEventsAsset>(self);
        register::<MapInfosAsset>(self);
        register::<SystemAsset>(self);
        register::<MapAsset>(self);
        self
    }
}

/// Registers the binary loader and processor for one asset type.
fn register<A: Asset + Serialize + DeserializeOwned>(app: &mut App) {
    app.register_asset_loader(RmmzBinLoader::<A>::default());
    let processor: RmmzBinProcessor<A> = RmmzBinSaver::<A>::default().into();
    app.register_asset_processor::<RmmzBinProcessor<A>>(processor);
}

#[cfg(test)]
mod tests {
    use crate::asset::{ItemsAsset, Table};
    use crate::data::Item;

    fn sample(name: &str) -> ItemsAsset {
        Table(vec![
            None,
            Some(Item {
                id: 1,
                name: name.to_owned(),
                price: 50,
                ..Default::default()
            }),
        ])
    }

    #[test]
    fn asset_round_trips_through_postcard() {
        let asset = sample("Potion");
        let bytes = postcard::to_stdvec(&asset).unwrap();
        // Binary is meaningfully smaller / different from JSON, and lossless.
        let back: ItemsAsset = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back.count(), 1);
        let item = back.get(1).unwrap();
        assert_eq!(item.name, "Potion");
        assert_eq!(item.price, 50);
    }

    #[test]
    fn binary_loader_reads_processed_asset_through_app() {
        use std::path::Path;

        use bevy_app::{App, TaskPoolPlugin};
        use bevy_asset::io::memory::{Dir, MemoryAssetReader};
        use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
        use bevy_asset::{AssetApp, AssetPlugin, AssetServer, Assets, Handle};

        use super::RmmzBinLoader;

        let bytes = postcard::to_stdvec(&sample("Ether")).unwrap();

        let dir = Dir::default();
        dir.insert_asset(Path::new("Items.rmmzbin"), bytes);
        let reader_dir = dir.clone();

        let mut app = App::new();
        // Register only the binary loader for this type, so a typed load resolves
        // to it (no JSON loader in this app to compete).
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
        .init_asset::<ItemsAsset>()
        .register_asset_loader(RmmzBinLoader::<ItemsAsset>::default());

        let handle: Handle<ItemsAsset> =
            app.world().resource::<AssetServer>().load("Items.rmmzbin");

        let mut name = None;
        for _ in 0..1000 {
            app.update();
            if let Some(asset) = app.world().resource::<Assets<ItemsAsset>>().get(&handle) {
                name = asset.get(1).map(|i| i.name.clone());
                break;
            }
        }
        assert_eq!(name.as_deref(), Some("Ether"));
    }
}
