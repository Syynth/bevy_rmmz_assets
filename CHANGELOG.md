# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-03

### Added

- Load RPG Maker MZ `data/*.json` database files as typed Bevy assets, with the
  `RmmzDatabase` resource layer for cheap id-based, hot-reload-aware lookups.
- Extensible note-metadata parsing: a `NoteParser` registry with a parse-once
  cache.
- Optional ahead-of-time binary (`postcard`) processing that bakes parsed note
  metadata into release builds (`process` feature).
- Optional `Map###.json` loading and map/event note metadata (`maps` feature).
- Filesystem hot-reload support (`file_watcher` feature).
- `RmmzDatabase::ready()` — a misuse-resistant load accessor returning
  `Option<Result<(), DatabaseLoadFailed>>` so a failed load surfaces as `Err`
  instead of masquerading as "still loading" (the footgun `is_loaded()` invites).
- The aggregate load status is latched once it settles, so steady-state
  `status()`/`is_loaded()`/`ready()` calls no longer re-poll every table handle.
- `rmmz_database_ready` run condition, so database-dependent systems can be gated
  with `my_system.run_if(rmmz_database_ready)`.
- Generic typed access on `RmmzDatabase`: `asset::<A>()`, `table::<R>()`,
  `record::<R>(id)` (built-in named accessors are now thin wrappers over these).
- Custom single-document asset types: `register_rmmz::<A>(file)` + the
  `rmmz_asset!` macro load a consumer-defined `data/*.json` into a typed asset,
  reachable via `db.asset::<A>()`. See `examples/custom.rs`.
- Custom id-indexed **table** types: `register_rmmz_table::<R>(file)` /
  `register_rmmz_note_table::<R>(file)` + the `rmmz_table!` macro, reachable via
  `db.table::<R>()` / `db.record::<R>(id)`. Note-bearing custom tables participate
  in the shared note cache (`db.note_meta`) just like built-in tables.
- Custom binary processing (`process` feature): `register_rmmz_bin::<A>()` /
  `register_rmmz_bin_table::<R>()` / `register_rmmz_bin_note_table::<R>()` bake
  custom types into the processed binary (note tables bake their notes too).
- With `file_watcher` off, custom assets are moved into the snapshot and their
  `Assets<A>` copy is freed, so release builds store custom data once.

### Changed

- `NoteRegistry::register` now refuses (and warns about) a second parser that
  reuses an existing `TAG` for a different output type, instead of silently
  clobbering the baked-note unbaker.

[Unreleased]: https://github.com/Syynth/bevy_rmmz_assets/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Syynth/bevy_rmmz_assets/releases/tag/v0.1.0
