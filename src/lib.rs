//! Library surface for rust-blog: the build-time content pipeline
//! (frontmatter parsing, markdown rendering, asset fingerprinting) and the
//! app's generated content model.
//!
//! The binary (`main.rs`) owns the Leptos UI and depends on this library for
//! the embedded post manifest, while the same parsing/rendering code is
//! reachable from `tests/` integration tests (golden HTML, RSS round-trip,
//! render fuzzing).

pub mod assets;
pub mod content;
pub mod fingerprint;
pub mod frontmatter;
#[cfg(not(target_arch = "wasm32"))]
pub mod markdown;
pub mod reading_time;
pub mod site;
pub mod xml;

/// Render a markdown body with the embedded content asset manifest.
///
/// Host-only: the app ships pre-rendered HTML from `build.rs`, so this is used
/// by the build script, `post_build`, and the host test suite.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_markdown(md: &str) -> String {
  markdown::render_with(md, &|src| assets::resolve(src).map(str::to_string))
}
