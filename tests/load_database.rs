//! End-to-end integration test: load a fixture `data/` directory from disk
//! through a real Bevy `App` and query it via the public API.

use bevy_app::{App, TaskPoolPlugin, Update};
use bevy_asset::AssetPlugin;
use bevy_ecs::prelude::{ResMut, Resource};
use bevy_rmmz_assets::prelude::*;
use serde::{Deserialize, Serialize};

/// A note-metadata type parsed from `<role:...>`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
struct Probe {
    status: Option<DatabaseStatus>,
    item1: Option<String>,
    item_count: usize,
    actor1: Option<String>,
    title: Option<String>,
    party: Vec<i32>,
    actor1_role: Option<String>,
}

fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
    out.status = Some(db.status());
    if db.is_loaded() {
        out.item1 = db.item(1).map(|i| i.name.clone());
        out.item_count = db.items().map_or(0, ItemsAsset::count);
        out.actor1 = db.actor(1).map(|a| a.name.clone());
        out.title = db.system().map(|s| s.game_title.clone());
        out.party = db
            .system()
            .map(|s| s.party_members.clone())
            .unwrap_or_default();
        out.actor1_role = db
            .actor(1)
            .and_then(|a| db.note_meta::<Role, _>(a))
            .map(|r| r.0.clone());
    }
}

#[test]
fn loads_fixture_database_from_disk() {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin {
            // Fixtures live at <crate root>/tests/fixtures/data/*.json.
            file_path: format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR")),
            watch_for_changes_override: Some(false),
            ..Default::default()
        },
    ))
    .init_resource::<Probe>()
    .register_note_parser(RoleParser)
    .add_rmmz_with(RmmzConfig::default().with_tables([
        CoreTable::Items,
        CoreTable::Actors,
        CoreTable::System,
    ]))
    .add_systems(Update, probe);

    // Pump until note metadata is cached (a frame or two after the data loads)
    // or loading fails.
    let mut settled = false;
    for _ in 0..2000 {
        app.update();
        let probe = app.world().resource::<Probe>();
        if probe.status == Some(DatabaseStatus::Failed) || probe.actor1_role.is_some() {
            settled = true;
            break;
        }
    }
    assert!(settled, "database never settled");

    let probe = app.world().resource::<Probe>();
    assert_eq!(
        probe.status,
        Some(DatabaseStatus::Loaded),
        "failed to load fixtures"
    );
    assert_eq!(probe.item1.as_deref(), Some("Potion"));
    assert_eq!(probe.item_count, 2);
    assert_eq!(probe.actor1.as_deref(), Some("Harold"));
    assert_eq!(probe.title.as_deref(), Some("Fixture Quest"));
    assert_eq!(probe.party, vec![1, 2]);
    // Note metadata parsed from `<role:hero>` on actor 1.
    assert_eq!(probe.actor1_role.as_deref(), Some("hero"));
}
