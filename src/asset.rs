//! Bevy [`Asset`] wrappers over the parsed RPG Maker MZ database files.
//!
//! Loaders deserialize each `data/*.json` file into one of these assets. The
//! array files use the generic [`Table`] (with per-file type aliases like
//! [`ActorsAsset`]); the single-object `System.json` and `Map###.json` use
//! [`SystemAsset`] and [`MapAsset`].
//!
//! These types are intentionally thin: they own the parsed data and offer
//! id-based access. The ergonomic cross-table [`crate`] resource layer is built
//! on top of them separately.

use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::data::{
    Actor, Animation, Armor, Class, CommonEvent, Enemy, Item, Map, MapInfo, Skill, State, System,
    Tileset, Troop, Weapon,
};

/// A null-padded list of database records, mirroring the MZ array files.
///
/// MZ array files begin with a `null` at index 0 (ids are 1-based) and may
/// contain further `null` gaps for deleted entries. Index by id with
/// [`Table::get`], or iterate the present records with [`Table::iter`].
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Table<T: TypePath + Send + Sync + 'static>(pub Vec<Option<T>>);

impl<T: TypePath + Send + Sync + 'static> Table<T> {
    /// Returns the record with the given 1-based `id`, or `None` if the id is
    /// out of range or that slot is a `null` gap.
    pub fn get(&self, id: usize) -> Option<&T> {
        self.0.get(id).and_then(Option::as_ref)
    }

    /// Iterates over the present (non-`null`) records, skipping gaps.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().filter_map(Option::as_ref)
    }

    /// Number of present (non-`null`) records, ignoring the leading `null` and
    /// any gaps. For the raw slot count (including padding), use `self.0.len()`.
    pub fn count(&self) -> usize {
        self.iter().count()
    }

    /// Whether the table contains no records, ignoring `null` padding. (A table
    /// holding only the leading `null` reports `true`.)
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}

/// `Actors.json` as an asset.
pub type ActorsAsset = Table<Actor>;
/// `Classes.json` as an asset.
pub type ClassesAsset = Table<Class>;
/// `Skills.json` as an asset.
pub type SkillsAsset = Table<Skill>;
/// `Items.json` as an asset.
pub type ItemsAsset = Table<Item>;
/// `Weapons.json` as an asset.
pub type WeaponsAsset = Table<Weapon>;
/// `Armors.json` as an asset.
pub type ArmorsAsset = Table<Armor>;
/// `Enemies.json` as an asset.
pub type EnemiesAsset = Table<Enemy>;
/// `States.json` as an asset.
pub type StatesAsset = Table<State>;
/// `Troops.json` as an asset.
pub type TroopsAsset = Table<Troop>;
/// `Animations.json` as an asset.
pub type AnimationsAsset = Table<Animation>;
/// `Tilesets.json` as an asset.
pub type TilesetsAsset = Table<Tileset>;
/// `CommonEvents.json` as an asset.
pub type CommonEventsAsset = Table<CommonEvent>;
/// `MapInfos.json` as an asset.
pub type MapInfosAsset = Table<MapInfo>;

/// The parsed `System.json` (a single object, not an array).
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SystemAsset(pub System);

/// A parsed `Map###.json` (a single object, not an array).
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapAsset(pub Map);

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use super::{ActorsAsset, CommonEventsAsset, MapAsset, SystemAsset, TroopsAsset};
    use crate::data::HasNote;

    // Compile-time proof the wrappers implement `Asset` (covers both the generic
    // `Table` path and the single-object newtypes, Reflect and non-Reflect).
    fn assert_asset<T: bevy_asset::Asset>(_: PhantomData<T>) {}

    #[test]
    fn wrappers_are_assets() {
        assert_asset(PhantomData::<ActorsAsset>);
        assert_asset(PhantomData::<CommonEventsAsset>);
        assert_asset(PhantomData::<TroopsAsset>);
        assert_asset(PhantomData::<SystemAsset>);
        assert_asset(PhantomData::<MapAsset>);
    }

    #[test]
    fn table_indexes_by_one_based_id_and_skips_gaps() {
        let t: ActorsAsset =
            serde_json::from_str(r#"[null,{"id":1,"name":"A"},null,{"id":3,"name":"C"}]"#).unwrap();
        assert!(t.get(0).is_none());
        assert_eq!(t.get(1).unwrap().name, "A");
        assert!(t.get(2).is_none());
        assert_eq!(t.get(3).unwrap().name, "C");
        assert!(t.get(4).is_none());
        assert_eq!(t.iter().count(), 2);
        assert_eq!(t.count(), 2);
        assert_eq!(t.0.len(), 4); // raw slots, including the null gaps
        assert!(!t.is_empty());
    }

    #[test]
    fn empty_table_reports_zero_records() {
        // The leading null must not be mistaken for a record.
        let t: ActorsAsset = serde_json::from_str("[null]").unwrap();
        assert_eq!(t.count(), 0);
        assert!(t.is_empty());
        assert_eq!(t.0.len(), 1);
    }

    #[test]
    fn system_asset_is_transparent() {
        let s: SystemAsset = serde_json::from_str(r#"{"gameTitle":"T"}"#).unwrap();
        assert_eq!(s.0.game_title, "T");
    }

    #[test]
    fn has_note_reads_the_note_field() {
        let a = crate::data::Actor {
            note: "<cls:warrior>".into(),
            ..Default::default()
        };
        assert_eq!(a.note(), "<cls:warrior>");
    }
}
