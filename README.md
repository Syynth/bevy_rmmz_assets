# bevy_rmmz_assets

Load an [RPG Maker MZ](https://www.rpgmakerweb.com/products/rpg-maker-mz) "game
database" — the `data/*.json` files an MZ project ships — into
[Bevy](https://bevyengine.org) as assets, with an ergonomic resource layer on
top.

> **Status:** early scaffold. The API is being built out issue-by-issue; expect
> rapid change.

## Goals

- **Assets first.** Each MZ database file (`Actors.json`, `Items.json`,
  `System.json`, …) loads into a strongly-typed Bevy `Asset`. Loaders are
  agnostic to which file they read.
- **Resource layer.** A `RmmzDatabase` resource builds indices over the loaded
  assets for cheap id-based lookups, staying in sync via `AssetEvent`s.
- **Hot-reload** in development via Bevy's filesystem asset watcher.
- **Optional binary processing.** Leverage Bevy's asset processor to pre-process
  the JSON into a compact binary form for release builds.
- **Extensible note metadata.** RPG Maker games conventionally embed structured
  metadata in the free-text `note` field (`<tag:value>`). A registry lets you
  plug in parsers that turn notes into typed metadata.
- **Maps are optional** — supported, but not the primary use case, and gated
  behind a feature.

## Bevy compatibility

| `bevy_rmmz_assets` | Bevy        |
| ------------------ | ----------- |
| `0.1` (unreleased) | `0.19.0-rc` |

The crate depends on the individual `bevy_app` / `bevy_asset` / `bevy_ecs` /
`bevy_reflect` sub-crates rather than the `bevy` umbrella.

## Cargo features

| Feature        | Description                                                       |
| -------------- | ----------------------------------------------------------------- |
| `maps`         | Helpers for loading `Map###.json` map assets.                     |
| `file_watcher` | Enable Bevy's filesystem watcher for dev hot-reload.              |
| `process`      | Enable Bevy's asset processor + compact binary (postcard) output. |

## License

MIT — see [LICENSE](LICENSE).
