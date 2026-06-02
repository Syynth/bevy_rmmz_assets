//! Common imports for working with `bevy_rmmz_assets`.
//!
//! ```
//! use bevy_rmmz_assets::prelude::*;
//! ```

pub use crate::RmmzAssetsPlugin;
pub use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, Table,
    TilesetsAsset, TroopsAsset, WeaponsAsset,
};
pub use crate::config::{CoreTable, RmmzConfig, RmmzHandles, TableSelection};
pub use crate::data::{
    Actor, Animation, Armor, AudioFile, Class, CommonEvent, Damage, DropItem, Effect, Enemy,
    EnemyAction, EventCommand, Item, Learning, Map, MapEvent, MapInfo, Skill, State, System, Terms,
    Tileset, Trait, Troop, Weapon,
};
pub use crate::data::{HasId, HasNote};
pub use crate::database::{DatabaseLoadFailed, DatabaseStatus, RmmzDatabase};
pub use crate::ext::RmmzAppExt;
pub use crate::loader::{RmmzJsonLoader, RmmzLoadError};
#[cfg(feature = "maps")]
pub use crate::maps::{MapLoad, RmmzMapNotes, RmmzMaps};
pub use crate::notes::{NoteParser, NoteRegistry, NoteTag, NoteTokens, ParsedNote, RmmzNoteCache};
#[cfg(feature = "process")]
pub use crate::processing::{
    RmmzBinError, RmmzBinLoader, RmmzBinProcessor, RmmzBinSaver, RmmzProcessingExt,
};
