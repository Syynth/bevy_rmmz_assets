//! Loads the bundled fixture `data/` directory and prints a few records plus a
//! parsed note-metadata value.
//!
//! Run with: `cargo run --example load`
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "examples print their results"
)]

use std::process::ExitCode;

use bevy_app::{App, TaskPoolPlugin, Update};
use bevy_asset::AssetPlugin;
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_rmmz_assets::prelude::*;
use serde::{Deserialize, Serialize};

/// Example metadata parsed from `<role:...>` notes.
#[derive(Debug, Serialize, Deserialize)]
struct Role(String);

struct RoleParser;
impl NoteParser for RoleParser {
    type Output = Role;
    const TAG: &'static str = "role";
    fn parse(&self, tokens: &NoteTokens) -> Option<Role> {
        tokens.value("role").map(|v| Role(v.to_owned()))
    }
}

#[derive(Resource, Default)]
struct Reported(bool);

fn report(db: RmmzDatabase, mut reported: ResMut<Reported>) {
    if reported.0 || !db.is_loaded() {
        return;
    }
    // Note metadata is cached a frame or two after the data loads; wait for
    // actor 1's notes before printing so the role shows up.
    match db.actor(1) {
        Some(actor) if db.parsed_note(actor).is_some() => {}
        _ => return,
    }

    if let Some(system) = db.system() {
        println!("game: {}", system.game_title);
        println!("party: {:?}", system.party_members);
    }
    if let Some(items) = db.items() {
        for item in items.iter() {
            println!("item #{} {} ({}G)", item.id, item.name, item.price);
        }
    }
    if let Some(actor) = db.actor(1) {
        let role = db.note_meta::<Role, _>(actor).map(|r| r.0.as_str());
        println!("actor #{} {} (role: {:?})", actor.id, actor.name, role);
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
    .register_note_parser(RoleParser)
    .add_rmmz_with(RmmzConfig::default().with_tables([
        CoreTable::Items,
        CoreTable::Actors,
        CoreTable::System,
    ]))
    .add_systems(Update, report);

    // Headless: pump the schedule until the database has loaded and we've
    // printed, then stop.
    for _ in 0..1000 {
        app.update();
        if app.world().resource::<Reported>().0 {
            return ExitCode::SUCCESS;
        }
    }
    eprintln!("database did not finish loading");
    ExitCode::FAILURE
}
