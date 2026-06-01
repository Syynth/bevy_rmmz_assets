//! [`RmmzDatabase`], the ergonomic cross-table access layer.

use bevy_asset::{Asset, Assets, Handle};
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

/// Returns whether a handle (if any) resolves to a loaded asset. A `None`
/// handle counts as ready, since nothing was requested.
fn ready<A: Asset>(handle: Option<&Handle<A>>, assets: &Assets<A>) -> bool {
    handle.is_none_or(|h| assets.get(h).is_some())
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

    /// Whether every table selected for loading has finished loading.
    ///
    /// Tables that were not selected do not block readiness.
    pub fn is_loaded(&self) -> bool {
        ready(self.handles.actors.as_ref(), &self.actors)
            && ready(self.handles.classes.as_ref(), &self.classes)
            && ready(self.handles.skills.as_ref(), &self.skills)
            && ready(self.handles.items.as_ref(), &self.items)
            && ready(self.handles.weapons.as_ref(), &self.weapons)
            && ready(self.handles.armors.as_ref(), &self.armors)
            && ready(self.handles.enemies.as_ref(), &self.enemies)
            && ready(self.handles.states.as_ref(), &self.states)
            && ready(self.handles.troops.as_ref(), &self.troops)
            && ready(self.handles.animations.as_ref(), &self.animations)
            && ready(self.handles.tilesets.as_ref(), &self.tilesets)
            && ready(self.handles.common_events.as_ref(), &self.common_events)
            && ready(self.handles.map_infos.as_ref(), &self.map_infos)
            && ready(self.handles.system.as_ref(), &self.system)
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

    use super::RmmzDatabase;
    use crate::config::{CoreTable, RmmzConfig};
    use crate::ext::RmmzAppExt;

    #[derive(Resource, Default)]
    struct Probe {
        loaded: bool,
        item1: Option<String>,
        actor_count: usize,
        title: Option<String>,
    }

    fn probe(db: RmmzDatabase, mut out: ResMut<Probe>) {
        if db.is_loaded() {
            out.loaded = true;
            out.item1 = db.item(1).map(|i| i.name.clone());
            out.actor_count = db.actors().map_or(0, crate::asset::ActorsAsset::count);
            out.title = db.system().map(|s| s.game_title.clone());
        }
    }

    #[test]
    fn database_reads_records_across_tables() {
        let dir = Dir::default();
        dir.insert_asset_text(
            Path::new("data/Items.json"),
            r#"[null,{"id":1,"name":"Potion"}]"#,
        );
        dir.insert_asset_text(
            Path::new("data/Actors.json"),
            r#"[null,{"id":1,"name":"Harold"},{"id":2,"name":"Therese"}]"#,
        );
        dir.insert_asset_text(Path::new("data/System.json"), r#"{"gameTitle":"Demo"}"#);
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
        .add_rmmz_with(RmmzConfig::default().with_tables([
            CoreTable::Items,
            CoreTable::Actors,
            CoreTable::System,
        ]))
        .add_systems(Update, probe);

        let mut done = false;
        for _ in 0..1000 {
            app.update();
            if app.world().resource::<Probe>().loaded {
                done = true;
                break;
            }
        }
        assert!(done, "database never reported loaded");

        let probe = app.world().resource::<Probe>();
        assert_eq!(probe.item1.as_deref(), Some("Potion"));
        assert_eq!(probe.actor_count, 2);
        assert_eq!(probe.title.as_deref(), Some("Demo"));
    }
}
