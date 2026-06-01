//! Parsing structured metadata out of RPG Maker `note` fields.
//!
//! [`tokens`] provides the low-level [`NoteTokens`] tokenizer for the
//! `<tag:value>` convention. A registry that turns tokens into typed metadata is
//! layered on top separately.

pub mod tokens;

pub use tokens::{NoteTag, NoteTokens};
