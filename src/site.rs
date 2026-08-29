//! Site-wide constants, shared by the UI (`rust_blog::site`), the RSS build
//! and the sitemap/robots generation (`build.rs` includes this file via
//! `#[path]`). One definition, no drift.

pub const TITLE: &str = "rust-blog";
pub const TAGLINE: &str = "Notes on Rust, WebAssembly, and building the web";
pub const DESCRIPTION: &str =
  "A blog written in Rust and Leptos, running in your browser as WebAssembly";
pub const AUTHOR: &str = "rust-blog";
pub const GITHUB_URL: &str = "https://github.com/rust-blog/rust-blog.github.io";
pub const SITE_URL: &str = "https://rust-blog.github.io";
