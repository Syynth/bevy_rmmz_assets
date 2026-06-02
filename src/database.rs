//! [`RmmzDatabase`], the ergonomic cross-table access layer.

use bevy_asset::{Asset, AssetServer, Assets, Handle};
use bevy_ecs::system::SystemParam;

use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset, TroopsAsset,
    WeaponsAsset,
};
use crate::config::RmmzHandles;
use crate::data::{
    Actor, Animation, Armor, Class, CommonEvent, Enemy, HasId, Item, MapInfo, Skill, State, System,
    Tileset, Troop, Weapon,
};
use crate::notes::{ParsedNote, RmmzNoteCache};

/// A [`SystemParam`] giving ergonomic, id-based read access to the loaded
/// RPG Maker MZ database.
///
/// It reads the [`RmmzHandles`] populated by
/// [`RmmzAppExt`](crate::ext::RmmzAppExt) together with the underlying
/// `Assets<…>` collections, so access is always live — hot-reloads are reflected
/// without any extra bookkeeping. Accessors return `None` when a table was not
/// selected for loading or has not finished loading yet.
///
/// ```no_run
/// use bevy_rmmz_assets::prelude::*;
///
/// fn print_potion(db: RmmzDatabase) {
///     if let Some(item) = db.item(1) {
///         println!("item 1 is {}", item.name);
///     }
/// }
/// ```
///
/// Requires [`RmmzAssetsPlugin`](crate::RmmzAssetsPlugin) (which registers the
/// asset collections) to have been added.
#[derive(SystemParam)]
pub struct RmmzDatabase<'w> {
    handles: bevy_ecs::system::Res<'w, RmmzHandles>,
    asset_server: bevy_ecs::system::Res<'w, AssetServer>,
    note_cache: bevy_ecs::system::Res<'w, RmmzNoteCache>,
    actors: bevy_ecs::system::Res<'w, Assets<ActorsAsset>>,
    classes: bevy_ecs::system::Res<'w, Assets<ClassesAsset>>,
    skills: bevy_ecs::system::Res<'w, Assets<SkillsAsset>>,
    items: bevy_ecs::system::Res<'w, Assets<ItemsAsset>>,
    weapons: bevy_ecs::system::Res<'w, Assets<WeaponsAsset>>,
    armors: bevy_ecs::system::Res<'w, Assets<ArmorsAsset>>,
    enemies: bevy_ecs::system::Res<'w, Assets<EnemiesAsset>>,
    states: bevy_ecs::system::Res<'w, Assets<StatesAsset>>,
    troops: bevy_ecs::system::Res<'w, Assets<TroopsAsset>>,
    animations: bevy_ecs::system::Res<'w, Assets<AnimationsAsset>>,
    tilesets: bevy_ecs::system::Res<'w, Assets<TilesetsAsset>>,
    common_events: bevy_ecs::system::Res<'w, Assets<CommonEventsAsset>>,
    map_infos: bevy_ecs::system::Res<'w, Assets<MapInfosAsset>>,
    system: bevy_ecs::system::Res<'w, Assets<SystemAsset>>,
    #[cfg(feature = "maps")]
    rmmz_maps: bevy_ecs::system::Res<'w, crate::maps::RmmzMaps>,
    #[cfg(feature = "maps")]
    map_assets: bevy_ecs::system::Res<'w, Assets<crate::asset::MapAsset>>,
    #[cfg(feature = "maps")]
    map_notes: bevy_ecs::system::Res<'w, crate::maps::RmmzMapNotes>,
}

/// Aggregate load status of the selected database tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseStatus {
    /// At least one selected table is still loading (and none have failed).
    Loading,
    /// Every selected table has loaded successfully.
    Loaded,
    /// At least one selected table failed to load (missing file, parse error, …).
    Failed,
}

impl DatabaseStatus {
    /// Combines two statuses, keeping the worst: `Failed` > `Loading` > `Loaded`.
    fn worse(self, other: Self) -> Self {
        match (self, other) {
            (Self::Failed, _) | (_, Self::Failed) => Self::Failed,
            (Self::Loading, _) | (_, Self::Loading) => Self::Loading,
            _ => Self::Loaded,
        }
    }
}

/// Status of a single (optional) table handle, via the asset server's load
/// state. A `None` handle counts as `Loaded`, since nothing was requested.
fn table_status<A: Asset>(server: &AssetServer, handle: Option<&Handle<A>>) -> DatabaseStatus {
    match handle {
        None => DatabaseStatus::Loaded,
        Some(h) => {
            let state = server.load_state(h.id());
            if state.is_failed() {
                DatabaseStatus::Failed
            } else if state.is_loaded() {
                DatabaseStatus::Loaded
            } else {
                DatabaseStatus::Loading
            }
        }
    }
}

macro_rules! table_accessors {
    ($($plural:ident / $single:ident => $asset:ty [$record:ty]),+ $(,)?) => {
        $(
            #[doc = concat!("Returns the loaded `", stringify!($plural), "` table, or `None`.")]
            pub fn $plural(&self) -> Option<&$asset> {
                self.handles.$plural.as_ref().and_then(|h| self.$plural.get(h))
            }

            #[doc = concat!("Returns the `", stringify!($single), "` with the given 1-based id.")]
            pub fn $single(&self, id: usize) -> Option<&$record> {
                self.$plural().and_then(|table| table.get(id))
            }
        )+
    };
}

impl RmmzDatabase<'_> {
    table_accessors! {
        actors / actor => ActorsAsset [Actor],
        classes / class => ClassesAsset [Class],
        skills / skill => SkillsAsset [Skill],
        items / item => ItemsAsset [Item],
        weapons / weapon => WeaponsAsset [Weapon],
        armors / armor => ArmorsAsset [Armor],
        enemies / enemy => EnemiesAsset [Enemy],
        states / state => StatesAsset [State],
        troops / troop => TroopsAsset [Troop],
        animations / animation => AnimationsAsset [Animation],
        tilesets / tileset => TilesetsAsset [Tileset],
        common_events / common_event => CommonEventsAsset [CommonEvent],
        map_infos / map_info => MapInfosAsset [MapInfo],
    }

    /// Returns the loaded [`System`] settings, or `None`.
    pub fn system(&self) -> Option<&System> {
        self.handles
            .system
            .as_ref()
            .and_then(|h| self.system.get(h))
            .map(|asset| &asset.0)
    }

    /// The aggregate load status across all selected tables.
    ///
    /// Returns [`DatabaseStatus::Failed`] if any selected table failed to load,
    /// otherwise [`DatabaseStatus::Loading`] while any are still pending,
    /// otherwise [`DatabaseStatus::Loaded`]. Tables that were not selected for
    /// loading do not affect the result.
    pub fn status(&self) -> DatabaseStatus {
        let s = &self.asset_server;
        table_status(s, self.handles.actors.as_ref())
            .worse(table_status(s, self.handles.classes.as_ref()))
            .worse(table_status(s, self.handles.skills.as_ref()))
            .worse(table_status(s, self.handles.items.as_ref()))
            .worse(table_status(s, self.handles.weapons.as_ref()))
            .worse(table_status(s, self.handles.armors.as_ref()))
            .worse(table_status(s, self.handles.enemies.as_ref()))
            .worse(table_status(s, self.handles.states.as_ref()))
            .worse(table_status(s, self.handles.troops.as_ref()))
            .worse(table_status(s, self.handles.animations.as_ref()))
            .worse(table_status(s, self.handles.tilesets.as_ref()))
            .worse(table_status(s, self.handles.common_events.as_ref()))
            .worse(table_status(s, self.handles.map_infos.as_ref()))
            .worse(table_status(s, self.handles.system.as_ref()))
    }

    /// Whether every selected table has loaded successfully.
    ///
    /// Returns `false` while still loading **and** on failure — use
    /// [`Self::status`] or [`Self::is_failed`] to distinguish the two.
    pub fn is_loaded(&self) -> bool {
        self.status() == DatabaseStatus::Loaded
    }

    /// Whether any selected table failed to load.
    pub fn is_failed(&self) -> bool {
        self.status() == DatabaseStatus::Failed
    }

    /// The metadata of type `O` parsed from `record`'s note, or `None`.
    ///
    /// This is a cache lookup — the note was parsed once when the table loaded,
    /// not on this call — so it is cheap to call in hot loops. Requires a
    /// [`NoteParser`](crate::notes::NoteParser) producing `O` to be registered.
    pub fn note_meta<O, R>(&self, record: &R) -> Option<&O>
    where
        O: Send + Sync + 'static,
        R: HasId + 'static,
    {
        self.note_cache.get::<R>(record.id())?.get::<O>()
    }

    /// All cached note metadata for `record`, or `None` if it carries none.
    pub fn parsed_note<R>(&self, record: &R) -> Option<&ParsedNote>
    where
        R: HasId + 'static,
    {
        self.note_cache.get::<R>(record.id())
    }
}

#[cfg(feature = "maps")]
impl RmmzDatabase<'_> {
    /// Returns the loaded map with the given id, or `None` if it hasn't loaded.
    pub fn map(&self, id: i32) -> Option<&crate::data::Map> {
        self.rmmz_maps
            .handle(id)
            .and_then(|handle| self.map_assets.get(handle))
            .map(crate::asset::MapAsset::map)
    }

    /// The map ids listed in `MapInfos.json` (whether or not each is loaded).
    pub fn map_ids(&self) -> Vec<i32> {
        self.handles
            .map_infos
            .as_ref()
            .and_then(|handle| self.map_infos.get(handle))
            .map(|infos| infos.iter().map(|info| info.id).collect())
            .unwrap_or_default()
    }

    /// Parsed note metadata of type `O` for the given map.
    pub fn map_note<O: Send + Sync + 'static>(&self, map_id: i32) -> Option<&O> {
        self.map_notes.map(map_id)?.get::<O>()
    }

    /// Parsed note metadata of type `O` for an event within a map.
    pub fn map_event_note<O: Send + Sync + 'static>(
        &self,
        map_id: i32,
        event_id: i32,
    ) -> Option<&O> {
        self.map_notes.event(map_id, event_id)?.get::<O>()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin, Update};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin};
    use bevy_ecs::prelude::{ResMut, Resource};
    use serde::{Deserialize, Serialize};

    use super::{DatabaseStatus, RmmzDatabase};
    use crate::config::{CoreTable, RmmzConfig};
    use crate::ext::RmmzAppExt;
    use crate::notes::{NoteParser, NoteTokens};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Element(String);

    struct ElementParser;
    impl NoteParser for ElementParser {
        type Output = Element;
        const TAG: &'static str = "element";
        fn parse(&self, tokens: &NoteTokens) -> Option<Element> {
            tokens.value("element").map(|v| Element(v.to_owned()))
        }
    }

    #[derive(Resource, Default)]
    struct Probe {
        status: Option<DatabaseStatus>,
        item1: Option<String>,
        actor_count: usize,
        title: Option<String>,
        element: Option<String>,
    }

    fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
        out.status = Some(db.status());
        if db.is_loaded() {
            out.item1 = db.item(1).map(|i| i.name.clone());
            out.actor_count = db.actors().map_or(0, crate::asset::ActorsAsset::count);
            out.title = db.system().map(|s| s.game_title.clone());
            out.element = db
                .item(1)
                .and_then(|i| db.note_meta::<Element, _>(i))
                .map(|e| e.0.clone());
        }
    }

    fn build_app(files: &[(&str, &str)], tables: &[CoreTable]) -> App {
        let dir = Dir::default();
        for (path, contents) in files {
            dir.insert_asset_text(Path::new(path), contents);
        }
        let reader_dir = dir.clone();

        let mut app = App::new();
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(MemoryAssetReader {
                    root: reader_dir.clone(),
                })
            }),
        )
        .add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin {
                watch_for_changes_override: Some(false),
                ..Default::default()
            },
        ))
        .init_resource::<Probe>()
        .add_rmmz_with(RmmzConfig::default().with_tables(tables.iter().copied()))
        .add_systems(Update, probe);
        app
    }

    /// Pumps the app until the probed status is no longer `Loading`.
    fn run_until_settled(app: &mut App) -> DatabaseStatus {
        for _ in 0..1000 {
            app.update();
            if let Some(status) = app.world().resource::<Probe>().status
                && status != DatabaseStatus::Loading
            {
                return status;
            }
        }
        DatabaseStatus::Loading
    }

    #[test]
    fn database_reads_records_across_tables() {
        let mut app = build_app(
            &[
                ("data/Items.json", r#"[null,{"id":1,"name":"Potion"}]"#),
                (
                    "data/Actors.json",
                    r#"[null,{"id":1,"name":"Harold"},{"id":2,"name":"Therese"}]"#,
                ),
                ("data/System.json", r#"{"gameTitle":"Demo"}"#),
            ],
            &[CoreTable::Items, CoreTable::Actors, CoreTable::System],
        );

        assert_eq!(run_until_settled(&mut app), DatabaseStatus::Loaded);

        let probe = app.world().resource::<Probe>();
        assert_eq!(probe.item1.as_deref(), Some("Potion"));
        assert_eq!(probe.actor_count, 2);
        assert_eq!(probe.title.as_deref(), Some("Demo"));
    }

    #[test]
    fn missing_file_reports_failed_not_stuck_loading() {
        // Items is selected but no Items.json exists in the source.
        let mut app = build_app(&[], &[CoreTable::Items]);
        assert_eq!(run_until_settled(&mut app), DatabaseStatus::Failed);
    }

    #[test]
    fn note_metadata_is_parsed_once_and_cached() {
        let mut app = build_app(
            &[(
                "data/Items.json",
                r#"[null,{"id":1,"name":"Ember","note":"<element:fire>"}]"#,
            )],
            &[CoreTable::Items],
        );
        app.register_note_parser(ElementParser);

        // Pump until the cache system has parsed the note (a frame or two after
        // the asset loads).
        let mut element = None;
        for _ in 0..1000 {
            app.update();
            element = app.world().resource::<Probe>().element.clone();
            if element.is_some() {
                break;
            }
        }
        assert_eq!(element.as_deref(), Some("fire"));
    }

    #[test]
    fn hot_reload_reparses_notes_and_updates_data() {
        use bevy_asset::Assets;

        use crate::asset::{ItemsAsset, Table};
        use crate::config::RmmzHandles;
        use crate::data::Item;

        let mut app = build_app(
            &[(
                "data/Items.json",
                r#"[null,{"id":1,"name":"Ember","note":"<element:fire>"}]"#,
            )],
            &[CoreTable::Items],
        );
        app.register_note_parser(ElementParser);

        let mut element = None;
        for _ in 0..1000 {
            app.update();
            element = app.world().resource::<Probe>().element.clone();
            if element.is_some() {
                break;
            }
        }
        assert_eq!(element.as_deref(), Some("fire"));

        // Simulate a hot-reload: replacing the asset fires AssetEvent::Modified,
        // which both updates the live data and re-runs the note cache.
        let handle = app.world().resource::<RmmzHandles>().items.clone().unwrap();
        {
            let mut items = app.world_mut().resource_mut::<Assets<ItemsAsset>>();
            items
                .insert(
                    handle.id(),
                    Table::new(vec![
                        None,
                        Some(Item {
                            id: 1,
                            name: "Cinder".to_owned(),
                            note: "<element:ice>".to_owned(),
                            ..Default::default()
                        }),
                    ]),
                )
                .unwrap();
        }

        let mut reparsed = None;
        for _ in 0..1000 {
            app.update();
            reparsed = app.world().resource::<Probe>().element.clone();
            if reparsed.as_deref() == Some("ice") {
                break;
            }
        }
        assert_eq!(
            reparsed.as_deref(),
            Some("ice"),
            "note cache did not re-parse"
        );
        // Live data reflects the reload too.
        assert_eq!(
            app.world().resource::<Probe>().item1.as_deref(),
            Some("Cinder")
        );
    }
}
