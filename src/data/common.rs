//! Sub-objects shared across multiple RPG Maker MZ database files.
//!
//! RPG Maker MZ encodes several small records in a uniform shape and reuses them
//! across many of the top-level database files. Modeling them once here keeps the
//! per-table models (actors, items, skills, …) small and consistent.
//!
//! Field names mirror the MZ JSON (which is camelCase) via
//! `#[serde(rename_all = "camelCase")]`; Rust fields stay `snake_case`.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A trait line attached to actors, classes, weapons, armors, enemies, and states.
///
/// MZ encodes every trait uniformly as a `(code, dataId, value)` triple. The
/// meaning of `data_id` and `value` depends on `code` (e.g. element rate, state
/// resist, parameter multiplier); this type intentionally does not interpret it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Trait {
    /// Trait code identifying what the trait does.
    pub code: i32,
    /// Secondary identifier whose meaning depends on `code` (e.g. element id,
    /// state id, parameter index).
    pub data_id: i32,
    /// The trait's numeric value (rate, bonus, etc.), per `code`.
    pub value: f64,
}

/// An effect applied by an item or a skill (recover HP, add a state, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    /// Effect code identifying what the effect does.
    pub code: i32,
    /// Secondary identifier whose meaning depends on `code`.
    pub data_id: i32,
    /// First magnitude parameter, per `code`.
    pub value1: f64,
    /// Second magnitude parameter, per `code`.
    pub value2: f64,
}

/// The damage calculation for an item or skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Damage {
    /// Damage type: 0 none, 1 HP damage, 2 MP damage, 3 HP recover, 4 MP recover,
    /// 5 HP drain, 6 MP drain. Named `kind` because `type` is a Rust keyword.
    #[serde(rename = "type")]
    pub kind: i32,
    /// Element id applied to the damage (0 = none, -1 = normal attack element).
    pub element_id: i32,
    /// The damage formula, evaluated by the MZ engine at runtime.
    pub formula: String,
    /// Variance percentage applied to the rolled damage.
    pub variance: i32,
    /// Whether the damage can land a critical hit.
    pub critical: bool,
}

/// A single entry in an MZ event command list.
///
/// Event command lists drive common events, map events, and troop event pages.
/// A command's `parameters` are heterogeneous and their shape depends entirely on
/// `code`, so they are preserved as raw JSON values rather than being interpreted
/// here.
///
/// This type is deliberately **not** `Reflect`: [`serde_json::Value`] does not
/// implement [`bevy_reflect::Reflect`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
    /// Command code identifying the operation (e.g. 401 = show-text line).
    pub code: i32,
    /// Nesting/indentation level of this command within its list.
    pub indent: i32,
    /// Command arguments, whose number and types depend on `code`.
    pub parameters: Vec<serde_json::Value>,
}

/// A reference to an audio asset together with its playback parameters.
///
/// Used for BGM, BGS, ME, and SE throughout the database (system sounds, map
/// audio, animation sound timings, …).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct AudioFile {
    /// Audio file name (without directory or extension); empty means "none".
    pub name: String,
    /// Stereo pan, -100..=100.
    pub pan: i32,
    /// Playback pitch percentage (100 = normal).
    pub pitch: i32,
    /// Playback volume percentage (0..=100).
    pub volume: i32,
}

#[cfg(test)]
mod tests {
    use super::{Damage, Effect, EventCommand, Trait};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn deserializes_trait() {
        let t: Trait = serde_json::from_str(r#"{"code":11,"dataId":2,"value":1.5}"#).unwrap();
        assert_eq!(t.code, 11);
        assert_eq!(t.data_id, 2);
        assert!(approx(t.value, 1.5));
    }

    #[test]
    fn deserializes_effect() {
        let e: Effect =
            serde_json::from_str(r#"{"code":11,"dataId":0,"value1":0.4,"value2":100}"#).unwrap();
        assert_eq!(e.code, 11);
        assert_eq!(e.data_id, 0);
        assert!(approx(e.value1, 0.4));
        assert!(approx(e.value2, 100.0));
    }

    #[test]
    fn deserializes_damage_with_type_key() {
        let json = r#"{"type":1,"elementId":2,"formula":"a.atk * 4 - b.def * 2","variance":20,"critical":true}"#;
        let d: Damage = serde_json::from_str(json).unwrap();
        assert_eq!(d.kind, 1);
        assert_eq!(d.element_id, 2);
        assert_eq!(d.formula, "a.atk * 4 - b.def * 2");
        assert_eq!(d.variance, 20);
        assert!(d.critical);
    }

    #[test]
    fn deserializes_event_command_with_mixed_parameters() {
        let json = r#"{"code":401,"indent":0,"parameters":["Hello",1,true]}"#;
        let c: EventCommand = serde_json::from_str(json).unwrap();
        assert_eq!(c.code, 401);
        assert_eq!(c.indent, 0);
        assert_eq!(c.parameters.len(), 3);
        assert_eq!(c.parameters[0], "Hello");
        assert_eq!(c.parameters[1], 1);
        assert_eq!(c.parameters[2], true);
    }

    #[test]
    fn round_trips_trait_through_json() {
        let t = Trait {
            code: 22,
            data_id: 1,
            value: 0.25,
        };
        let s = serde_json::to_string(&t).unwrap();
        // Serializes back to the MZ camelCase key.
        assert!(s.contains("\"dataId\":1"));
        let back: Trait = serde_json::from_str(&s).unwrap();
        assert_eq!(t, back);
    }
}
