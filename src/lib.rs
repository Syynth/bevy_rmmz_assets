//! `bevy_rmmz_assets` loads an [RPG Maker MZ] "game database" — the collection of
//! `data/*.json` files an MZ project ships — into [Bevy] as assets, and layers an
//! ergonomic resource on top for lookups.
//!
//! # Design
//!
//! The crate is **layered**:
//!
//! - **Assets** are the source of truth. Each database file deserializes into a
//!   strongly-typed [`bevy_asset::Asset`]. Loaders are agnostic to *which* file
//!   they are reading — they just turn JSON into the requested asset type.
//! - A **resource** layer builds indices on top of the loaded assets for cheap
//!   id-based access, staying in sync via [`bevy_asset::AssetEvent`]s.
//! - A **config resource** plus an `App` **extension trait** decide *what* is
//!   loaded and *how* (e.g. whether and how map files are loaded).
//!
//! RPG Maker games conventionally stash structured metadata in the free-text
//! `note` field of database entries (the `<tag:value>` convention). This crate
//! provides an extensible registry for parsing those notes into typed metadata.
//!
//! # Status
//!
//! Early scaffold. The plugin currently wires up nothing; functionality lands
//! incrementally (see the project's issue tracker).
//!
//! [RPG Maker MZ]: https://www.rpgmakerweb.com/products/rpg-maker-mz
//! [Bevy]: https://bevyengine.org

mod plugin;

pub mod prelude;

pub use plugin::RmmzAssetsPlugin;
