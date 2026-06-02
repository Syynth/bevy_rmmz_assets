//! [`RmmzDatabase`], the ergonomic cross-table access layer.

use bevy_asset::{AssetServer, Assets, UntypedHandle};
use bevy_ecs::system::SystemParam;
use thiserror::Error;

use crate::asset::{
    ActorsAsset, AnimationsAsset, ArmorsAsset, ClassesAsset, CommonEventsAsset, EnemiesAsset,
    ItemsAsset, MapInfosAsset, SkillsAsset, StatesAsset, SystemAsset, TilesetsAsset, TroopsAsset,
    WeaponsAsset,
};
use crate::config::RmmzRegistry;
use crate::data::{
    Actor, Animation, Armor, Class, CommonEvent, Enemy, HasId, Item, MapInfo, Skill, State, System,
    Tileset, Troop, Weapon,
};
use crate::notes::{ParsedNote, RmmzNoteCache};

/// A [`SystemParam`] giving ergonomic, id-based read access to the loaded
/// RPG Maker MZ database.
///
/// It reads the [`RmmzRegistry`](crate::config::RmmzRegistry) populated by
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
    registry: bevy_ecs::system::Res<'w, RmmzRegistry>,
    asset_server: bevy_ecs::system::Res<'w, AssetServer>,
    note_cache: bevy_ecs::system::Res<'w, RmmzNoteCache>,
    load_status: bevy_ecs::system::Res<'w, RmmzLoadStatus>,
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

/// Status of one registered handle. A `None` handle means "registered but not
/// yet loaded" (the startup load hasn't run), which counts as `Loading`.
fn handle_status(server: &AssetServer, handle: Option<&UntypedHandle>) -> DatabaseStatus {
    match handle {
        None => DatabaseStatus::Loading,
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

/// The aggregate status across every registered table, computed live. An empty
/// registry (nothing requested) is [`Loaded`](DatabaseStatus::Loaded).
fn aggregate_status(server: &AssetServer, registry: &RmmzRegistry) -> DatabaseStatus {
    registry.handles().fold(DatabaseStatus::Loaded, |acc, h| {
        acc.worse(handle_status(server, h))
    })
}

/// Caches the database's load status once it settles (becomes [`Loaded`] or
/// [`Failed`]), so steady-state callers of [`RmmzDatabase::status`] /
/// [`RmmzDatabase::is_loaded`] don't re-poll every table handle each frame.
///
/// The status is **latched at first settle**: it reflects the initial load
/// outcome and is not re-evaluated afterward, so a later hot-reload failure does
/// not flip a database that already reported [`Loaded`].
///
/// [`Loaded`]: DatabaseStatus::Loaded
/// [`Failed`]: DatabaseStatus::Failed
#[derive(bevy_ecs::resource::Resource, Debug, Default, Clone, Copy)]
pub struct RmmzLoadStatus(Option<DatabaseStatus>);

/// Run condition for [`track_load_status`]: `true` until the load settles. Gating
/// the tracker on this lets the scheduler skip it entirely after the database has
/// finished loading, instead of running it every frame just to early-return.
pub(crate) fn load_status_unsettled(latch: bevy_ecs::system::Res<RmmzLoadStatus>) -> bool {
    latch.0.is_none()
}

/// System that latches the database status once it has settled. Gated by
/// [`load_status_unsettled`] and added by
/// [`RmmzAssetsPlugin`](crate::RmmzAssetsPlugin), so it stops running once the
/// status is known.
pub(crate) fn track_load_status(
    server: bevy_ecs::system::Res<AssetServer>,
    registry: bevy_ecs::system::Res<RmmzRegistry>,
    mut latch: bevy_ecs::system::ResMut<RmmzLoadStatus>,
) {
    let status = aggregate_status(&server, &registry);
    if status != DatabaseStatus::Loading {
        latch.0 = Some(status);
    }
}

/// A Bevy run condition that is `true` once every selected table has loaded
/// successfully — gate database-dependent systems with
/// `my_system.run_if(rmmz_database_ready)` instead of an in-system readiness
/// check.
///
/// Equivalent to [`RmmzDatabase::is_loaded`]: `false` while still loading and on
/// failure. It reads the **latched** status (see [`RmmzLoadStatus`]), so it is
/// cheap to evaluate every frame — but, like the latch, it reflects the *initial*
/// load outcome: once the database has settled it does not flip back to `false`
/// on a later hot-reload failure (nor recover from an initial failure).
pub fn rmmz_database_ready(db: RmmzDatabase) -> bool {
    db.is_loaded()
}

/// Error returned by [`RmmzDatabase::ready`] when at least one selected table
/// failed to load (missing file, parse error, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("one or more selected RPG Maker MZ database tables failed to load")]
pub struct DatabaseLoadFailed;

macro_rules! table_accessors {
    ($($plural:ident / $single:ident => $asset:ty [$record:ty]),+ $(,)?) => {
        $(
            #[doc = concat!("Returns the loaded `", stringify!($plural), "` table, or `None`.")]
            pub fn $plural(&self) -> Option<&$asset> {
                self.registry.handle::<$asset>().and_then(|h| self.$plural.get(&h))
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
        self.registry
            .handle::<SystemAsset>()
            .and_then(|h| self.system.get(&h))
            .map(|asset| &asset.0)
    }

    /// The aggregate load status across all selected tables.
    ///
    /// Returns [`DatabaseStatus::Failed`] if any selected table failed to load,
    /// otherwise [`DatabaseStatus::Loading`] while any are still pending,
    /// otherwise [`DatabaseStatus::Loaded`]. Tables that were not selected for
    /// loading do not affect the result.
    pub fn status(&self) -> DatabaseStatus {
        // Fast path: once loading has settled, the tracker latches the result, so
        // steady-state callers avoid re-polling every handle on each call.
        if let Some(settled) = self.load_status.0 {
            return settled;
        }
        aggregate_status(&self.asset_server, &self.registry)
    }

    /// Polls the load as a misuse-resistant `Result`:
    ///
    /// - `None` — still loading.
    /// - `Some(Ok(()))` — every selected table loaded successfully.
    /// - `Some(Err(`[`DatabaseLoadFailed`]`))` — at least one table failed.
    ///
    /// Prefer this over [`Self::is_loaded`] in systems that gate on readiness: a
    /// failed load surfaces as `Err` rather than masquerading as "still loading",
    /// so it can't be silently waited on forever.
    ///
    /// ```no_run
    /// use bevy_rmmz_assets::prelude::*;
    ///
    /// fn use_db(db: RmmzDatabase) {
    ///     match db.ready() {
    ///         None => return,                 // still loading this frame
    ///         Some(Err(_)) => return,         // load failed — handle/log it
    ///         Some(Ok(())) => {}              // ready
    ///     }
    ///     let _ = db.item(1);
    /// }
    /// ```
    pub fn ready(&self) -> Option<Result<(), DatabaseLoadFailed>> {
        match self.status() {
            DatabaseStatus::Loading => None,
            DatabaseStatus::Loaded => Some(Ok(())),
            DatabaseStatus::Failed => Some(Err(DatabaseLoadFailed)),
        }
    }

    /// Whether every selected table has loaded successfully (i.e. status is
    /// [`DatabaseStatus::Loaded`]).
    ///
    /// Returns `false` while still loading **and** on failure. To avoid waiting
    /// forever on a failed load, prefer [`Self::ready`] (or check
    /// [`Self::status`] / [`Self::is_failed`]).
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
        self.registry
            .handle::<MapInfosAsset>()
            .and_then(|handle| self.map_infos.get(&handle))
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

    use super::{DatabaseLoadFailed, DatabaseStatus, RmmzDatabase, RmmzLoadStatus};
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
        ready: Option<Result<(), DatabaseLoadFailed>>,
        item1: Option<String>,
        actor_count: usize,
        title: Option<String>,
        element: Option<String>,
    }

    fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
        out.status = Some(db.status());
        out.ready = db.ready();
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
        // `ready()` resolves to `Ok` once loaded, and the status latches.
        assert_eq!(probe.ready, Some(Ok(())));
        assert_eq!(
            app.world().resource::<RmmzLoadStatus>().0,
            Some(DatabaseStatus::Loaded),
            "settled status should be latched"
        );
    }

    #[test]
    fn ready_run_condition_gates_dependent_systems() {
        use bevy_ecs::schedule::IntoScheduleConfigs;

        use super::rmmz_database_ready;

        #[derive(Resource, Default)]
        struct Ran(u32);
        fn bump(mut c: ResMut<Ran>) {
            c.0 += 1;
        }

        let mut app = build_app(
            &[("data/Items.json", r#"[null,{"id":1,"name":"Potion"}]"#)],
            &[CoreTable::Items],
        );
        app.init_resource::<Ran>()
            .add_systems(Update, bump.run_if(rmmz_database_ready));

        // While still loading, the gated system must not run.
        let mut status = DatabaseStatus::Loading;
        for _ in 0..2000 {
            app.update();
            status = app
                .world()
                .resource::<Probe>()
                .status
                .unwrap_or(DatabaseStatus::Loading);
            if status != DatabaseStatus::Loading {
                break;
            }
            assert_eq!(
                app.world().resource::<Ran>().0,
                0,
                "gated system ran before the database was ready"
            );
        }
        assert_eq!(status, DatabaseStatus::Loaded);

        // Once ready, it runs on every subsequent frame.
        let at_ready = app.world().resource::<Ran>().0;
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(app.world().resource::<Ran>().0, at_ready + 5);
    }

    #[test]
    fn missing_file_reports_failed_not_stuck_loading() {
        // Items is selected but no Items.json exists in the source.
        let mut app = build_app(&[], &[CoreTable::Items]);
        assert_eq!(run_until_settled(&mut app), DatabaseStatus::Failed);
        // A failed load surfaces through `ready()` as `Err`, not as "still
        // loading" (`None`) — the footgun `is_loaded()` invites.
        assert_eq!(
            app.world().resource::<Probe>().ready,
            Some(Err(DatabaseLoadFailed))
        );
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
        use crate::config::RmmzRegistry;
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
        let handle = app
            .world()
            .resource::<RmmzRegistry>()
            .handle::<ItemsAsset>()
            .unwrap();
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
