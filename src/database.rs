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
    Actor, Animation, Armor, Class, CommonEvent, Enemy, Item, MapInfo, Skill, State, System,
    Tileset, Troop, Weapon,
};

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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy_app::{App, TaskPoolPlugin, Update};
    use bevy_asset::io::memory::{Dir, MemoryAssetReader};
    use bevy_asset::io::{AssetSourceBuilder, AssetSourceId};
    use bevy_asset::{AssetApp, AssetPlugin};
    use bevy_ecs::prelude::{ResMut, Resource};

    use super::{DatabaseStatus, RmmzDatabase};
    use crate::config::{CoreTable, RmmzConfig};
    use crate::ext::RmmzAppExt;

    #[derive(Resource, Default)]
    struct Probe {
        status: Option<DatabaseStatus>,
        item1: Option<String>,
        actor_count: usize,
        title: Option<String>,
    }

    fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
        out.status = Some(db.status());
        if db.is_loaded() {
            out.item1 = db.item(1).map(|i| i.name.clone());
            out.actor_count = db.actors().map_or(0, crate::asset::ActorsAsset::count);
            out.title = db.system().map(|s| s.game_title.clone());
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
}
