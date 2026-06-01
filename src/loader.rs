//! A generic [`AssetLoader`] that deserializes RPG Maker MZ JSON into a typed
//! asset.

use core::marker::PhantomData;

use bevy_asset::io::Reader;
use bevy_asset::{Asset, AssetLoader, LoadContext};
use bevy_reflect::TypePath;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Errors produced by [`RmmzJsonLoader`].
#[derive(Debug, Error)]
pub enum RmmzLoadError {
    /// The asset bytes could not be read from the source.
    #[error("failed to read RPG Maker MZ asset: {0}")]
    Io(#[from] std::io::Error),
    /// The JSON could not be deserialized into the target type.
    #[error("failed to parse RPG Maker MZ JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Loads an RPG Maker MZ `data/*.json` file into the asset type `A`.
///
/// The loader is agnostic to *which* file it reads — it simply deserializes the
/// JSON bytes into `A`. One loader is registered per asset type; because RPG
/// Maker uses the `.json` extension for every database file, typed loads
/// (`asset_server.load::<A>(path)`) are resolved by asset type rather than by
/// extension.
#[derive(TypePath)]
pub struct RmmzJsonLoader<A>(PhantomData<fn() -> A>);

impl<A> Default for RmmzJsonLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A> AssetLoader for RmmzJsonLoader<A>
where
    A: Asset + DeserializeOwned,
{
    type Asset = A;
    type Settings = ();
    type Error = RmmzLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<A, RmmzLoadError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin, AssetServer, Assets, Handle};

    use crate::RmmzAssetsPlugin;
    use crate::asset::ItemsAsset;

    fn app_with_data(file: &str, contents: &str) -> App {
        let dir = Dir::default();
        dir.insert_asset_text(Path::new(file), contents);
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
            RmmzAssetsPlugin,
        ));
        app
    }

    #[test]
    fn loads_a_database_file_into_its_asset() {
        let mut app = app_with_data(
            "Items.json",
            r#"[null,{"id":1,"name":"Potion","itypeId":1,"price":50},{"id":2,"name":"Ether","price":120}]"#,
        );

        let handle: Handle<ItemsAsset> = app.world().resource::<AssetServer>().load("Items.json");

        // Pump the schedule until the async load lands in `Assets<ItemsAsset>`.
        let mut loaded = false;
        for _ in 0..1000 {
            app.update();
            if app
                .world()
                .resource::<Assets<ItemsAsset>>()
                .get(&handle)
                .is_some()
            {
                loaded = true;
                break;
            }
        }
        assert!(loaded, "Items.json never finished loading");

        let items = app
            .world()
            .resource::<Assets<ItemsAsset>>()
            .get(&handle)
            .unwrap();
        assert_eq!(items.count(), 2);
        assert_eq!(items.get(1).unwrap().name, "Potion");
        assert_eq!(items.get(1).unwrap().price, 50);
        assert_eq!(items.get(2).unwrap().name, "Ether");
    }
}
