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
//! provides an extensible registry ([`notes`]) for parsing those notes into
//! typed metadata. Notes are parsed once per record at load and cached, so
//! lookups never re-parse.
//!
//! # Quick start
//!
//! ```no_run
//! use bevy_app::App;
//! use bevy_rmmz_assets::prelude::*;
//!
//! let mut app = App::new();
//! // ... add Bevy's AssetPlugin (e.g. via DefaultPlugins) first ...
//! app.add_rmmz(); // load `data/*.json` and build the database
//!
//! fn read(db: RmmzDatabase) {
//!     if let Some(item) = db.item(1) {
//!         let _ = &item.name;
//!     }
//! }
//! ```
//!
//! # Hot-reload
//!
//! Because [`RmmzDatabase`] reads the live `Assets<…>` collections and the note
//! cache rebuilds on [`bevy_asset::AssetEvent::Modified`], edits to a `data`
//! file are reflected automatically — no extra bookkeeping. Enable Bevy's
//! filesystem watcher with the `file_watcher` feature (a passthrough to
//! `bevy_asset/file_watcher`) during development.
//!
//! # Ahead-of-time processing
//!
//! With the `process` feature, Bevy's asset processor pre-processes the database
//! into compact binary, **including** the parsed note metadata, so release
//! builds load it without any JSON or note parsing. See [`processing`].
//!
//! [RPG Maker MZ]: https://www.rpgmakerweb.com/products/rpg-maker-mz
//! [Bevy]: https://bevyengine.org
//! [`RmmzDatabase`]: crate::database::RmmzDatabase
#![warn(missing_docs)]

pub mod asset;
pub mod config;
pub mod data;
pub mod database;
pub mod ext;
pub mod loader;
#[cfg(feature = "maps")]
pub mod maps;
pub mod notes;
#[cfg(feature = "process")]
pub mod processing;

mod plugin;

pub mod prelude;

pub use plugin::RmmzAssetsPlugin;
