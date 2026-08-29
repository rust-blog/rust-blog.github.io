//! Library surface for rust-blog: content loading, the shared frontmatter
//! parser, and markdown rendering.
//!
//! The binary (`main.rs`) owns the Leptos UI and depends on this library so
//! the same parsing/rendering code is also reachable from `tests/`
//! integration tests (golden HTML, RSS round-trip, render fuzzing).

pub mod content;
pub mod frontmatter;
pub mod markdown;
pub mod site;
