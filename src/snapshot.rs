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
#[cfg(feature = "file_watcher")]
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::{MessageReader, ResMut, Resource};

use crate::config::RmmzRegistry;

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
/// removal, by **cloning** — used when `file_watcher` is on, so the live
/// `Assets<A>` copy stays available for hot-reload. Registered once per custom
/// type.
#[cfg(feature = "file_watcher")]
pub(crate) fn snapshot_asset<A: Asset + Clone>(
    mut events: MessageReader<AssetEvent<A>>,
    assets: Res<Assets<A>>,
    registry: Res<RmmzRegistry>,
    mut snapshot: ResMut<RmmzAssets>,
) {
    // Only the registered handle's asset represents this custom type; ignore
    // events for any other `Assets<A>` entry so a stray load/unload can't clobber
    // or clear the snapshot.
    let Some(registered) = registry.handle::<A>() else {
        return;
    };
    let registered = registered.id();
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id }
                if *id == registered =>
            {
                if let Some(asset) = assets.get(*id) {
                    snapshot
                        .map
                        .insert(TypeId::of::<A>(), Arc::new(asset.clone()));
                }
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } if *id == registered => {
                snapshot.map.remove(&TypeId::of::<A>());
            }
            _ => {}
        }
    }
}

/// Mirrors a custom asset `A` into [`RmmzAssets`] by **moving** it out of
/// `Assets<A>` and dropping its handle — used when `file_watcher` is off, so the
/// data is stored once (in the snapshot) rather than twice. Registered once per
/// custom type.
///
/// Note-bearing tables chain `cache_table_notes::<R>` *before* this, so notes are
/// parsed from `Assets<A>` while it still holds the asset. Once moved, the entry
/// is marked owned and this system no-ops (its handle is gone).
#[cfg(not(feature = "file_watcher"))]
pub(crate) fn own_snapshot_asset<A: Asset>(
    mut events: MessageReader<AssetEvent<A>>,
    mut assets: ResMut<Assets<A>>,
    mut registry: ResMut<RmmzRegistry>,
    mut snapshot: ResMut<RmmzAssets>,
) {
    let Some(registered) = registry.handle::<A>() else {
        return; // already owned (handle dropped), or not yet registered
    };
    let registered = registered.id();
    for event in events.read() {
        let owned = matches!(
            event,
            AssetEvent::Added { id } | AssetEvent::LoadedWithDependencies { id }
                if *id == registered
        ) && {
            if let Some(asset) = assets.remove(registered) {
                snapshot.map.insert(TypeId::of::<A>(), Arc::new(asset));
                registry.mark_owned::<A>();
                true
            } else {
                false
            }
        };
        if owned {
            break;
        }
    }
}
