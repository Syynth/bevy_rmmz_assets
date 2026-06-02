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
    match db.ready() {
        None => return,             // still loading this frame
        Some(Err(_)) => return,     // a selected table failed to load — handle it
        Some(Ok(())) => {}          // ready
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

### Custom data types

Beyond the built-in tables you can load **your own** `data/*.json` files — the
kind community plugins ship — as typed assets, reachable through the same
`RmmzDatabase`. Define a type, wire it with `rmmz_asset!`, and register it:

```rust
use std::collections::HashMap;

use bevy_asset::Asset;
use bevy_reflect::TypePath;
use bevy_rmmz_assets::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct AnimationMap(HashMap<String, String>);
rmmz_asset!(AnimationMap);

// In your app setup, after add_rmmz():
//   app.register_rmmz::<AnimationMap>("AnimationMap.json");

fn use_anim(db: RmmzDatabase) {
    if let Some(map) = db.asset::<AnimationMap>() {
        let _ = map.0.get("furnitureBreak");
    }
}
```

The type just needs `#[derive(Asset, …)]` and `Deserialize + Clone`. Built-in
tables are reachable the same generic way — `db.asset::<ItemsAsset>()`,
`db.table::<Item>()`, `db.record::<Item>(id)` — so code can treat built-in and
custom data uniformly. A runnable version lives in
[`examples/custom.rs`](examples/custom.rs):

```sh
cargo run --example custom
```

> Custom **id-array** tables (the MZ `Table<R>` shape) work too: define a record
> type, `rmmz_table!(R)`, then `register_rmmz_table::<R>("File.json")` (or
> `register_rmmz_note_table` if its records carry `<tag:value>` notes), and read
> via `db.table::<R>()` / `db.record::<R>(id)`. Most plugin data is map- or
> object-shaped, so the singleton form above is the common case.

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

### Ahead-of-time processing (optional, `process` feature)

For release builds you can pre-process the JSON database into a compact
[`postcard`](https://docs.rs/postcard) binary so the game loads it without any
JSON or note parsing. Enable the `process` feature, register your note parsers,
then register the processing pipeline before startup:

```rust
use bevy_rmmz_assets::prelude::*;

// app.register_note_parser(ElementParser); // register parsers first
// app.register_rmmz_processing();          // then the JSON -> binary pipeline
```

Because every RPG Maker file uses the `.json` extension, there is no single
global default processor — each asset type has its own. `register_rmmz_processing`
registers all of them; you opt a file in with an asset `.meta` that names the
matching processor, and run Bevy with its asset processor enabled. Parsed note
metadata is baked into the binary at processing time, so processed builds skip
note parsing entirely. See the [`processing`] module docs for the full details.

Custom types bake too — `register_rmmz_bin::<T>()` (and `register_rmmz_bin_table`
/ `register_rmmz_bin_note_table` for tables) wire the same pipeline for a
consumer-defined type. Separately, with `file_watcher` off, custom assets are
moved into the snapshot and their `Assets<A>` copy freed, so release builds store
custom data once rather than twice.

[`processing`]: https://docs.rs/bevy_rmmz_assets/latest/bevy_rmmz_assets/processing/index.html

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
