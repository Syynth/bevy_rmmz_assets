//! The crate's Bevy plugin.

use bevy_app::{App, Plugin};

/// Wires RPG Maker MZ database loading into a Bevy [`App`].
///
/// Add it with [`App::add_plugins`]. At this stage the plugin is a placeholder;
/// asset types, loaders, the configuration resource, the note-parser registry,
/// and the database resource are registered here as they are implemented.
#[derive(Debug, Default, Clone, Copy)]
pub struct RmmzAssetsPlugin;

impl Plugin for RmmzAssetsPlugin {
    fn build(&self, _app: &mut App) {
        // Registration is added incrementally as features land.
    }
}
