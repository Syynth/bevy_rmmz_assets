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
use bevy_asset::transformer::{AssetTransformer, IdentityAssetTransformer, TransformedAsset};
use bevy_asset::{Asset, AssetApp, AssetLoader, AssetPath, LoadContext};
use bevy_reflect::TypePath;
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::asset::{
    AnimationsAsset, BakedRecordNotes, CommonEventsAsset, MapAsset, MapInfosAsset, RmmzAsset,
    SystemAsset, Table, TroopsAsset,
};
use crate::data::{
    Actor, Armor, Class, Enemy, HasId, HasNote, Item, Skill, State, Tileset, Weapon,
};
use crate::loader::RmmzJsonLoader;
use crate::notes::{NoteBaker, NoteRegistry, NoteTokens};

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

/// Bakes parsed note metadata into a table at processing time, so the runtime
/// loads it (deserialize) instead of re-parsing note strings.
#[derive(TypePath)]
pub struct NoteBakingTransformer<R> {
    baker: NoteBaker,
    _marker: core::marker::PhantomData<fn() -> R>,
}

impl<R> NoteBakingTransformer<R> {
    fn new(baker: NoteBaker) -> Self {
        Self {
            baker,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<R> AssetTransformer for NoteBakingTransformer<R>
where
    R: HasNote + HasId + TypePath + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type AssetInput = Table<R>;
    type AssetOutput = Table<R>;
    type Settings = ();
    type Error = core::convert::Infallible;

    async fn transform(
        &self,
        mut asset: TransformedAsset<Table<R>>,
        _settings: &Self::Settings,
    ) -> Result<TransformedAsset<Table<R>>, Self::Error> {
        let baked: Vec<BakedRecordNotes> = asset
            .get()
            .iter()
            .map(|record| BakedRecordNotes {
                id: record.id(),
                tags: self.baker.bake(&NoteTokens::parse(record.note())),
            })
            .filter(|entry| !entry.tags.is_empty())
            .collect();
        asset.get_mut().set_baked(baked);
        Ok(asset)
    }
}

/// The [`Process`](bevy_asset::processor::Process) for a note-bearing table:
/// loads JSON, bakes parsed note metadata, and saves the compact binary.
pub type RmmzBinNoteProcessor<R> = LoadTransformAndSave<
    RmmzJsonLoader<Table<R>>,
    NoteBakingTransformer<R>,
    RmmzBinSaver<Table<R>>,
>;

/// App extension registering the binary processing pipeline.
pub trait RmmzProcessingExt {
    /// Registers a binary loader and a JSON→binary processor for every core
    /// asset type. Has effect only when the `asset_processor` feature is active.
    fn register_rmmz_processing(&mut self) -> &mut Self;
}

impl RmmzProcessingExt for App {
    fn register_rmmz_processing(&mut self) -> &mut Self {
        // Snapshot the registered parsers so the (World-free) transformer can
        // bake notes. Register parsers before calling this.
        let baker = self
            .world()
            .get_resource::<NoteRegistry>()
            .map(NoteRegistry::baker)
            .unwrap_or_default();

        // Note-bearing tables: bake parsed metadata into the binary.
        register_baked::<Actor>(self, &baker);
        register_baked::<Class>(self, &baker);
        register_baked::<Skill>(self, &baker);
        register_baked::<Item>(self, &baker);
        register_baked::<Weapon>(self, &baker);
        register_baked::<Armor>(self, &baker);
        register_baked::<Enemy>(self, &baker);
        register_baked::<State>(self, &baker);
        register_baked::<Tileset>(self, &baker);

        // Remaining tables/objects have no indexed notes: plain data processing.
        register::<TroopsAsset>(self);
        register::<AnimationsAsset>(self);
        register::<CommonEventsAsset>(self);
        register::<MapInfosAsset>(self);
        register::<SystemAsset>(self);
        register::<MapAsset>(self);
        self
    }
}

/// Registers a binary loader + identity (data-only) processor for one asset type.
fn register<A: RmmzAsset + Serialize + DeserializeOwned>(app: &mut App) {
    app.register_asset_loader(RmmzBinLoader::<A>::default());
    let processor: RmmzBinProcessor<A> = RmmzBinSaver::<A>::default().into();
    app.register_asset_processor::<RmmzBinProcessor<A>>(processor);
}

/// Registers a binary loader + note-baking processor for a note-bearing table.
fn register_baked<R>(app: &mut App, baker: &NoteBaker)
where
    R: HasNote + HasId + TypePath + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    app.register_asset_loader(RmmzBinLoader::<Table<R>>::default());
    let processor = RmmzBinNoteProcessor::<R>::new(
        NoteBakingTransformer::<R>::new(baker.clone()),
        RmmzBinSaver::<Table<R>>::default(),
    );
    app.register_asset_processor::<RmmzBinNoteProcessor<R>>(processor);
}

#[cfg(test)]
mod tests {
    use crate::asset::{ItemsAsset, Table};
    use crate::data::Item;

    fn sample(name: &str) -> ItemsAsset {
        Table::new(vec![
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

    #[test]
    fn baked_notes_load_into_cache_without_reparsing() {
        use std::path::Path;

        use bevy_app::{App, TaskPoolPlugin, Update};
        use bevy_asset::io::memory::{Dir, MemoryAssetReader};
        use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
        use bevy_asset::{AssetApp, AssetPlugin, AssetServer, Assets, Handle};
        use serde::{Deserialize, Serialize};

        use super::{BakedRecordNotes, NoteTokens, RmmzBinLoader};
        use crate::asset::ItemsAsset;
        use crate::data::{HasId, HasNote, Item};
        use crate::notes::{NoteParser, NoteRegistry, RmmzNoteCache, cache_table_notes};

        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Element(String);

        struct ElementParser;
        impl NoteParser for ElementParser {
            type Output = Element;
            fn parse(&self, tokens: &NoteTokens) -> Option<Element> {
                tokens.value("element").map(|v| Element(v.to_owned()))
            }
        }

        // Bake the item's note ahead of time (as the processing transformer does).
        let mut bake_registry = NoteRegistry::default();
        bake_registry.register(ElementParser);
        let mut table = Table::new(vec![
            None,
            Some(Item {
                id: 1,
                name: "Ember".to_owned(),
                note: "<element:fire>".to_owned(),
                ..Default::default()
            }),
        ]);
        let baked: Vec<BakedRecordNotes> = table
            .iter()
            .map(|record| BakedRecordNotes {
                id: record.id(),
                tags: bake_registry.bake(&NoteTokens::parse(record.note())),
            })
            .collect();
        table.set_baked(baked);
        let bytes = postcard::to_stdvec(&table).unwrap();

        let dir = Dir::default();
        dir.insert_asset(Path::new("Items.rmmzbin"), bytes);
        let reader_dir = dir.clone();

        // Runtime registry knows how to *deserialize* the baked metadata (same
        // parser registered), but the cache should not need to parse the note.
        let mut runtime_registry = NoteRegistry::default();
        runtime_registry.register(ElementParser);

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
        .init_asset::<ItemsAsset>()
        .register_asset_loader(RmmzBinLoader::<ItemsAsset>::default())
        .insert_resource(runtime_registry)
        .init_resource::<RmmzNoteCache>()
        .add_systems(Update, cache_table_notes::<Item>);

        let _handle: Handle<ItemsAsset> =
            app.world().resource::<AssetServer>().load("Items.rmmzbin");

        let mut element = None;
        for _ in 0..1000 {
            app.update();
            // The asset must be loaded *and* the cache populated from baked data.
            if app
                .world()
                .resource::<Assets<ItemsAsset>>()
                .iter()
                .next()
                .is_some()
                && let Some(note) = app.world().resource::<RmmzNoteCache>().get::<Item>(1)
            {
                element = note.get::<Element>().map(|e| e.0.clone());
                break;
            }
        }
        assert_eq!(element.as_deref(), Some("fire"));
    }
}
