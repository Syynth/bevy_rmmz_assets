//! Loads a **custom** (non-built-in) config file as a typed Bevy asset and prints
//! it. Demonstrates registering a consumer-defined type — here a string-keyed map
//! like RPG Maker community plugins ship, *not* an MZ id-array — and reading it
//! back through the same [`RmmzDatabase`] used for the built-in tables.
//!
//! Run with: `cargo run --example custom`
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "examples print their results"
)]

use std::collections::HashMap;
use std::process::ExitCode;

use bevy_app::{App, TaskPoolPlugin, Update};
use bevy_asset::{Asset, AssetPlugin};
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_reflect::TypePath;
use bevy_rmmz_assets::prelude::*;
use serde::{Deserialize, Serialize};

/// One entry of the animation map.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnimEntry {
    backend: String,
    config: String,
}

/// A custom `data/AnimationMap.json`: a string-keyed map (a whole-document config,
/// not an MZ 1-based id array). `rmmz_asset!` wires it for loading + access.
#[derive(Asset, TypePath, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct AnimationMap(HashMap<String, AnimEntry>);
rmmz_asset!(AnimationMap);

#[derive(Resource, Default)]
struct Reported(bool);

fn report(db: RmmzDatabase, mut reported: ResMut<Reported>) {
    if reported.0 {
        return;
    }
    // Snapshotted a frame or two after the file loads; wait for it.
    let Some(map) = db.asset::<AnimationMap>() else {
        return;
    };

    println!("AnimationMap has {} entries:", map.0.len());
    let mut keys: Vec<&String> = map.0.keys().collect();
    keys.sort();
    for key in keys {
        let entry = &map.0[key];
        println!("  {key}: backend={} config={}", entry.backend, entry.config);
    }
    reported.0 = true;
}

fn main() -> ExitCode {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin {
            file_path: format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR")),
            watch_for_changes_override: Some(false),
            ..Default::default()
        },
    ))
    .init_resource::<Reported>()
    // No built-in tables here — just the one custom type.
    .add_rmmz_with(RmmzConfig::default().with_tables([]))
    .register_rmmz::<AnimationMap>("AnimationMap.json")
    .add_systems(Update, report);

    // Headless: pump until we've printed, then stop.
    for _ in 0..1000 {
        app.update();
        if app.world().resource::<Reported>().0 {
            return ExitCode::SUCCESS;
        }
    }
    eprintln!("AnimationMap did not load");
    ExitCode::FAILURE
}
