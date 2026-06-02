//! Snapshot of loaded custom assets for generic, type-keyed access.
//!
//! [`RmmzDatabase`](crate::database::RmmzDatabase) is a `SystemParam` with static
//! fields, so it cannot hold `Res<Assets<A>>` for a consumer-defined type `A` it
//! has never heard of. Custom assets are therefore mirrored into this resource on
//! load (as `Arc<A>`), and the database reads them back by [`TypeId`]. Built-in
//! types are **not** snapshotted — they read their `Assets<…>` collections
//! directly (zero-copy), avoiding any duplication of large data.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use bevy_asset::{Asset, AssetEvent, Assets};
use bevy_ecs::prelude::{MessageReader, Res, ResMut, Resource};

/// Type-keyed snapshot of loaded custom assets, each held as `Arc<A>`.
///
/// Populated by [`snapshot_asset`] (registered per custom type) and read through
/// [`RmmzDatabase::asset`](crate::database::RmmzDatabase::asset).
#[derive(Resource, Default)]
pub struct RmmzAssets {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl RmmzAssets {
    /// The snapshotted instance of `A`, if it has been loaded.
    pub(crate) fn get<A: Asset>(&self) -> Option<&A> {
        self.map
            .get(&TypeId::of::<A>())
            .and_then(|a| a.downcast_ref::<A>())
    }
}

/// Mirrors a custom asset `A` into [`RmmzAssets`] on load/reload and drops it on
/// removal. Registered once per custom type by
/// [`RmmzAppExt::register_rmmz`](crate::ext::RmmzAppExt::register_rmmz).
///
/// Clones the asset once per (re)load into an `Arc` — cheap to read thereafter.
pub(crate) fn snapshot_asset<A: Asset + Clone>(
    mut events: MessageReader<AssetEvent<A>>,
    assets: Res<Assets<A>>,
    mut snapshot: ResMut<RmmzAssets>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                if let Some(asset) = assets.get(*id) {
                    snapshot
                        .map
                        .insert(TypeId::of::<A>(), Arc::new(asset.clone()));
                }
            }
            AssetEvent::Removed { .. } | AssetEvent::Unused { .. } => {
                snapshot.map.remove(&TypeId::of::<A>());
            }
        }
    }
}
