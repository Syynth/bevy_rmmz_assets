//! Strongly-typed serde models for the RPG Maker MZ database files.
//!
//! Each MZ `data/*.json` file maps onto types in this module. Sub-objects that
//! are reused across several files live in [`common`].

pub mod common;

pub use common::{Damage, Effect, EventCommand, Trait};
