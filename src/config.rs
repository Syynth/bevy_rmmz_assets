//! Configuration for which RPG Maker MZ data to load and from where, plus the
//! resource that holds the resulting asset handles.

use std::collections::HashSet;

use bevy_asset::Handle;
use bevy_ecs::prelude::Resource;

use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset, TroopsAsset,
    WeaponsAsset,
};

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

/// Strong handles to the loaded core database assets.
///
/// Populated on startup according to [`RmmzConfig`]. Holding the handles keeps
/// the assets loaded; the resource layer reads them to build its indices. Each
/// field is `Some` only if the corresponding table was selected for loading.
#[derive(Resource, Debug, Default, Clone)]
pub struct RmmzHandles {
    /// Handle to the loaded `Actors.json`.
    pub actors: Option<Handle<ActorsAsset>>,
    /// Handle to the loaded `Classes.json`.
    pub classes: Option<Handle<ClassesAsset>>,
    /// Handle to the loaded `Skills.json`.
    pub skills: Option<Handle<SkillsAsset>>,
    /// Handle to the loaded `Items.json`.
    pub items: Option<Handle<ItemsAsset>>,
    /// Handle to the loaded `Weapons.json`.
    pub weapons: Option<Handle<WeaponsAsset>>,
    /// Handle to the loaded `Armors.json`.
    pub armors: Option<Handle<ArmorsAsset>>,
    /// Handle to the loaded `Enemies.json`.
    pub enemies: Option<Handle<EnemiesAsset>>,
    /// Handle to the loaded `States.json`.
    pub states: Option<Handle<StatesAsset>>,
    /// Handle to the loaded `Troops.json`.
    pub troops: Option<Handle<TroopsAsset>>,
    /// Handle to the loaded `Animations.json`.
    pub animations: Option<Handle<AnimationsAsset>>,
    /// Handle to the loaded `Tilesets.json`.
    pub tilesets: Option<Handle<TilesetsAsset>>,
    /// Handle to the loaded `CommonEvents.json`.
    pub common_events: Option<Handle<CommonEventsAsset>>,
    /// Handle to the loaded `MapInfos.json`.
    pub map_infos: Option<Handle<MapInfosAsset>>,
    /// Handle to the loaded `System.json`.
    pub system: Option<Handle<SystemAsset>>,
}
