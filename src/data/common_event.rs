//! `CommonEvents.json` — reusable event command lists.

use serde::{Deserialize, Serialize};

use crate::data::common::EventCommand;

/// A single entry in `CommonEvents.json`.
///
/// Not `Reflect`: it holds an [`EventCommand`] list, whose parameters are
/// `serde_json::Value`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommonEvent {
    /// Database id (1-based).
    pub id: i32,
    /// The event's command list.
    pub list: Vec<EventCommand>,
    /// Display name.
    pub name: String,
    /// Switch that gates an autorun/parallel common event.
    pub switch_id: i32,
    /// Trigger: 0 none, 1 autorun, 2 parallel.
    pub trigger: i32,
}
