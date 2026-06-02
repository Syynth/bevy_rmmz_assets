//! Configuration for which RPG Maker MZ data to load and from where, plus the
//! resource that holds the resulting asset handles.

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use bevy_asset::{Asset, AssetServer, Handle, UntypedHandle};
use bevy_ecs::prelude::Resource;

/// Identifies a core (non-map) RPG Maker MZ database file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreTable {
    /// `Actors.json`
    Actors,
    /// `Classes.json`
    Classes,
    /// `Skills.json`
    Skills,
    /// `Items.json`
    Items,
    /// `Weapons.json`
    Weapons,
    /// `Armors.json`
    Armors,
    /// `Enemies.json`
    Enemies,
    /// `States.json`
    States,
    /// `Troops.json`
    Troops,
    /// `Animations.json`
    Animations,
    /// `Tilesets.json`
    Tilesets,
    /// `CommonEvents.json`
    CommonEvents,
    /// `MapInfos.json`
    MapInfos,
    /// `System.json`
    System,
}

/// Selects which core tables to load.
#[derive(Debug, Clone, Default)]
pub enum TableSelection {
    /// Load every core table.
    #[default]
    All,
    /// Load only the listed tables.
    Only(HashSet<CoreTable>),
}

/// Describes what RPG Maker MZ data to load and from where.
///
/// Insert this before startup (e.g. via
/// [`RmmzAppExt::add_rmmz_with`](crate::ext::RmmzAppExt::add_rmmz_with)) to
/// control loading. Map loading is configured separately (and is feature-gated).
#[derive(Resource, Debug, Clone)]
pub struct RmmzConfig {
    /// Directory holding the `*.json` files, relative to the asset source root.
    /// Asset paths always use `/`, regardless of platform.
    pub data_path: String,
    /// Which core tables to load.
    pub tables: TableSelection,
}

impl Default for RmmzConfig {
    fn default() -> Self {
        Self {
            data_path: "data".to_owned(),
            tables: TableSelection::All,
        }
    }
}

impl RmmzConfig {
    /// Creates a config loading every core table from `data_path`.
    pub fn new(data_path: impl Into<String>) -> Self {
        Self {
            data_path: data_path.into(),
            ..Self::default()
        }
    }

    /// Restricts loading to the given tables.
    #[must_use]
    pub fn with_tables(mut self, tables: impl IntoIterator<Item = CoreTable>) -> Self {
        self.tables = TableSelection::Only(tables.into_iter().collect());
        self
    }

    /// Whether `table` should be loaded under this configuration.
    pub(crate) fn loads(&self, table: CoreTable) -> bool {
        match &self.tables {
            TableSelection::All => true,
            TableSelection::Only(set) => set.contains(&table),
        }
    }

    /// Builds the asset path for a database `file` under [`Self::data_path`].
    ///
    /// Tolerates a trailing slash on `data_path`, and treats an empty
    /// `data_path` as "load from the asset source root".
    pub(crate) fn path(&self, file: &str) -> String {
        let base = self.data_path.trim_end_matches('/');
        if base.is_empty() {
            file.to_owned()
        } else {
            format!("{base}/{file}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreTable, RmmzConfig};

    #[test]
    fn path_joins_without_double_slashes() {
        assert_eq!(
            RmmzConfig::new("data").path("Actors.json"),
            "data/Actors.json"
        );
        assert_eq!(
            RmmzConfig::new("data/").path("Actors.json"),
            "data/Actors.json"
        );
        assert_eq!(
            RmmzConfig::new("nested/dir/").path("System.json"),
            "nested/dir/System.json"
        );
    }

    #[test]
    fn empty_data_path_loads_from_root() {
        assert_eq!(RmmzConfig::new("").path("Items.json"), "Items.json");
    }

    #[test]
    fn selection_controls_which_tables_load() {
        let all = RmmzConfig::default();
        assert!(all.loads(CoreTable::Actors));
        assert!(all.loads(CoreTable::System));

        let only = RmmzConfig::default().with_tables([CoreTable::Items, CoreTable::System]);
        assert!(only.loads(CoreTable::Items));
        assert!(only.loads(CoreTable::System));
        assert!(!only.loads(CoreTable::Actors));
    }
}

/// One registered asset type: where it loads from, and (once loaded) its handle.
struct RegistryEntry {
    /// Filename relative to [`RmmzConfig::data_path`], e.g. `"Items.json"`.
    file: String,
    /// Loads the file as the concrete asset type, capturing `A` behind a closure
    /// so the (type-erased) registry can drive loading without knowing `A`.
    load: Box<dyn Fn(&AssetServer, &str) -> UntypedHandle + Send + Sync>,
    /// The handle, set once [`RmmzRegistry::load_all`] has run.
    handle: Option<UntypedHandle>,
}

/// A `TypeId`-keyed registry of every asset type to load — built-in and custom
/// alike.
///
/// Replaces per-type handle fields with one map, so loading, status aggregation
/// (and, in later phases, note caching and baking) all iterate a single
/// structure. Built-ins are registered by
/// [`RmmzAppExt::add_rmmz_with`](crate::ext::RmmzAppExt::add_rmmz_with) per the
/// [`RmmzConfig`] selection; holding the handles keeps the assets loaded.
#[derive(Resource, Default)]
pub struct RmmzRegistry {
    entries: HashMap<TypeId, RegistryEntry>,
}

impl RmmzRegistry {
    /// Registers asset type `A` to load from `file` (relative to the data path).
    /// Idempotent per type. Prefer the `App`-level helpers over calling this
    /// directly.
    pub fn register<A: Asset>(&mut self, file: impl Into<String>) {
        self.entries
            .entry(TypeId::of::<A>())
            .or_insert_with(|| RegistryEntry {
                file: file.into(),
                load: Box::new(|server, path| server.load::<A>(path.to_owned()).untyped()),
                handle: None,
            });
    }

    /// The typed handle for `A`, if `A` is registered and has been loaded.
    pub fn handle<A: Asset>(&self) -> Option<Handle<A>> {
        self.entries
            .get(&TypeId::of::<A>())
            .and_then(|e| e.handle.clone())
            .map(UntypedHandle::typed::<A>)
    }

    /// Loads every registered-but-unloaded type, recording its handle. Run once
    /// at startup; idempotent thereafter.
    pub(crate) fn load_all(&mut self, server: &AssetServer, config: &RmmzConfig) {
        for entry in self.entries.values_mut() {
            if entry.handle.is_none() {
                let path = config.path(&entry.file);
                entry.handle = Some((entry.load)(server, &path));
            }
        }
    }

    /// The current handle of each registered entry (`None` until loaded), for
    /// status aggregation.
    pub(crate) fn handles(&self) -> impl Iterator<Item = Option<&UntypedHandle>> + '_ {
        self.entries.values().map(|e| e.handle.as_ref())
    }
}
