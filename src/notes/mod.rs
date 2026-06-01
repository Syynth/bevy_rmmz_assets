//! Parsing structured metadata out of RPG Maker `note` fields.
//!
//! [`tokens`] provides the low-level [`NoteTokens`] tokenizer for the
//! `<tag:value>` convention. A registry that turns tokens into typed metadata is
//! layered on top separately.

pub mod cache;
pub mod registry;
pub mod tokens;

pub use cache::{RmmzNoteCache, cache_table_notes};
pub use registry::{NoteParser, NoteRegistry, ParsedNote};
pub use tokens::{NoteTag, NoteTokens};
