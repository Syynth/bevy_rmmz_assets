# bevy_rmmz_assets

Load an [RPG Maker MZ](https://www.rpgmakerweb.com/products/rpg-maker-mz) "game
database" — the `data/*.json` files an MZ project ships — into
[Bevy](https://bevyengine.org) as assets, with an ergonomic resource layer on
top.

> **Status:** pre-release (`0.1`, unpublished). Targets Bevy `0.19.0-rc`; expect
> some churn until both stabilize.

## Usage

Add the plugin (after Bevy's `AssetPlugin` / `DefaultPlugins`) and query the
database from any system:

```rust
use bevy_rmmz_assets::prelude::*;

// In your app setup, after AssetPlugin:
//   app.add_rmmz();                       // load all core tables from `data/`
//   app.add_rmmz_with(RmmzConfig::new("data").with_tables([CoreTable::Items]));

fn use_database(db: RmmzDatabase) {
    if !db.is_loaded() {
        return;
    }
    if let Some(item) = db.item(1) {
        println!("item 1 is {}", item.name);
    }
}
```

Parse structured metadata out of the `note` field by registering a parser:

```rust
use bevy_rmmz_assets::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Element(String);

struct ElementParser;
impl NoteParser for ElementParser {
    type Output = Element;
    const TAG: &'static str = "element"; // stable tag, used in baked assets
    fn parse(&self, tokens: &NoteTokens) -> Option<Element> {
        tokens.value("element").map(|v| Element(v.to_owned()))
    }
}

// app.register_note_parser(ElementParser);
// then in a system: db.note_meta::<Element, _>(item)
```

Notes are parsed once per record at load and cached, so `note_meta` lookups are
cheap. With the `process` feature, the parsed metadata is baked into the
processed binary so release builds do no parsing at all.

A runnable example lives in [`examples/load.rs`](examples/load.rs):

```sh
cargo run --example load
```

### Maps (optional, `maps` feature)

`Map###.json` loading is opt-in. Enable it and choose a strategy, then read
maps and their notes by id:

```rust
use bevy_rmmz_assets::prelude::*;

// after add_rmmz():
//   app.enable_rmmz_maps(MapLoad::Eager);     // load all maps from MapInfos
//   app.enable_rmmz_maps(MapLoad::OnDemand);  // load on request

// Request maps in one system (RmmzDatabase reads RmmzMaps, so requesting —
// which needs &mut RmmzMaps — must be a separate system):
fn request_maps(mut maps: ResMut<RmmzMaps>) {
    maps.request(1); // (OnDemand) ask for Map001.json
}

// Read maps and their note metadata in another:
fn use_maps(db: RmmzDatabase) {
    if let Some(map) = db.map(1) {
        let _ = &map.display_name;
        // db.map_note::<Biome>(1);
        // db.map_event_note::<Chest>(1, event_id);
    }
}
```

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
