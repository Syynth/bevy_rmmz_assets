//! `Animations.json` — battle/skill animations.
//!
//! RPG Maker MZ animations are Effekseer-based and the on-disk shape varies
//! (MZ-native vs. MV-compatibility entries). These models cover the documented
//! MZ fields and use `#[serde(default)]` so entries that omit fields still load.
//! **Validate against a real project's `Animations.json` before relying on the
//! finer fields.**

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::data::common::AudioFile;

/// A 3-axis rotation (degrees) applied to the animation effect.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct Rotation {
    /// Rotation about the x axis.
    pub x: f64,
    /// Rotation about the y axis.
    pub y: f64,
    /// Rotation about the z axis.
    pub z: f64,
}

/// A screen-flash keyframe.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct FlashTiming {
    /// Frame at which the flash occurs.
    pub frame: i32,
    /// Flash duration in frames.
    pub duration: i32,
    /// Flash color as `[r, g, b, intensity]`.
    pub color: Vec<i32>,
}

/// A sound-effect keyframe.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct SoundTiming {
    /// Frame at which the sound plays.
    pub frame: i32,
    /// The sound effect to play.
    pub se: AudioFile,
}

/// A single entry in `Animations.json`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase", default)]
pub struct Animation {
    /// Database id (1-based).
    pub id: i32,
    /// Where the effect is positioned: 0 head, 1 center, 2 whole screen.
    pub display_type: i32,
    /// Effekseer effect file name.
    pub effect_name: String,
    /// Screen-flash keyframes.
    pub flash_timings: Vec<FlashTiming>,
    /// Display name.
    pub name: String,
    /// Horizontal offset.
    pub offset_x: i32,
    /// Vertical offset.
    pub offset_y: i32,
    /// Effect rotation.
    pub rotation: Rotation,
    /// Uniform scale percentage (100 = normal).
    pub scale: i32,
    /// Sound-effect keyframes.
    pub sound_timings: Vec<SoundTiming>,
    /// Playback speed percentage (100 = normal).
    pub speed: i32,
    /// Whether the effect is aligned to the bottom of the target.
    pub align_bottom: bool,
    /// Whether the effect triggers a screen quake.
    pub quake_effect: bool,
}
