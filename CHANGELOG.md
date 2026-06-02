# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Load RPG Maker MZ `data/*.json` database files as typed Bevy assets, with the
  `RmmzDatabase` resource layer for cheap id-based, hot-reload-aware lookups.
- Extensible note-metadata parsing: a `NoteParser` registry with a parse-once
  cache.
- Optional ahead-of-time binary (`postcard`) processing that bakes parsed note
  metadata into release builds (`process` feature).
- Optional `Map###.json` loading and map/event note metadata (`maps` feature).
- Filesystem hot-reload support (`file_watcher` feature).

### Changed

- `NoteRegistry::register` now refuses (and warns about) a second parser that
  reuses an existing `TAG` for a different output type, instead of silently
  clobbering the baked-note unbaker.

[Unreleased]: https://github.com/Syynth/bevy_rmmz_assets/commits/main
