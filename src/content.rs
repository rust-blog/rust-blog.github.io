use crate::markdown;
use include_dir::{Dir, File, include_dir};
use serde::Deserialize;

/// Embedded content directory. Drop a new `content/posts/<slug>.md` file in and
/// it is picked up automatically at compile time - no code changes required.
static CONTENT_DIR: Dir = include_dir!("content");

/// Site-wide constants used across the UI and the RSS feed.
#[allow(dead_code)]
pub mod site {
  pub const TITLE: &str = "rust-blog";
  pub const TAGLINE: &str = "Notes on Rust, WebAssembly, and building the web";
  pub const DESCRIPTION: &str =
    "A blog written in Rust and Leptos, running in your browser as WebAssembly";
  pub const AUTHOR: &str = "rust-blog";
  pub const GITHUB_URL: &str = "https://github.com/rust-blog/rust-blog.github.io";
  pub const SITE_URL: &str = "https://rust-blog.github.io";
}

/// Frontmatter parsed from the top of each post markdown file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PostMeta {
  pub title: String,
  pub date: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default)]
  pub author: Option<String>,
  #[serde(default)]
  pub draft: bool,
  #[serde(default)]
  pub slug: Option<String>,
}

/// A fully processed blog post, ready to render.
#[derive(Debug, Clone, PartialEq)]
pub struct Post {
  pub slug: String,
  pub meta: PostMeta,
  /// Raw markdown body (kept for RSS / future use).
  pub body: String,
  /// Rendered HTML body.
  pub html: String,
  /// Estimated reading time in minutes.
  pub reading_time: usize,
}

/// Load, parse and sort every published post from the embedded content tree.
pub fn load_posts() -> Vec<Post> {
  let mut posts = Vec::new();

  // include_dir's `files()` is non-recursive, so collect every file in the
  // tree (including the nested `posts/` directory) first.
  let mut files = Vec::new();
  collect_files(&CONTENT_DIR, &mut files);

  for file in files {
    let path = file.path();
    let is_markdown = path
      .extension()
      .and_then(|e| e.to_str())
      .map(|e| e.eq_ignore_ascii_case("md"))
      .unwrap_or(false);
    let in_posts = path
      .components()
      .any(|c| c.as_os_str().to_string_lossy() == "posts");

    if is_markdown
      && in_posts
      && let Some(raw) = file.contents_utf8()
      && let Some(post) = parse_post(raw, path)
      && !post.meta.draft
    {
      posts.push(post);
    }
  }

  posts.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
  posts
}

/// Recursively gather every file beneath a directory.
fn collect_files(dir: &Dir<'static>, out: &mut Vec<&File<'static>>) {
  for file in dir.files() {
    out.push(file);
  }
  for sub in dir.dirs() {
    collect_files(sub, out);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn loads_embedded_posts() {
    let posts = load_posts();
    assert!(
      posts.len() >= 2,
      "expected at least 2 embedded posts, found {}",
      posts.len()
    );
    assert!(posts.iter().any(|p| p.slug == "welcome"));
    assert!(posts.iter().any(|p| p.slug == "rust-variables"));
    // newest first
    assert!(posts[0].meta.date >= posts[1].meta.date);
  }
}

/// Parse a single markdown file into a [`Post`].
fn parse_post(raw: &str, path: &std::path::Path) -> Option<Post> {
  let trimmed = raw.strip_prefix('\u{feff}').unwrap_or(raw);
  if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
    return None;
  }

  let rest = &trimmed[3..];
  let end = rest.find("\n---")?;
  let frontmatter = &rest[..end];
  let mut body = rest[end + 4..].to_string();
  if let Some(stripped) = body.strip_prefix('\n') {
    body = stripped.to_string();
  }

  let meta: PostMeta = serde_yaml::from_str(frontmatter).ok()?;

  let slug = meta
    .slug
    .clone()
    .or_else(|| path.file_stem().map(|s| s.to_string_lossy().to_string()))
    .unwrap_or_default();

  let html = markdown::render(&body);
  let reading_time = ((body.split_whitespace().count() as f64 / 200.0).ceil() as usize).max(1);

  Some(Post {
    slug,
    meta,
    body,
    html,
    reading_time,
  })
}

/// Collect the unique, sorted list of tags across all posts.
pub fn all_tags(posts: &[Post]) -> Vec<String> {
  let mut tags: Vec<String> = posts
    .iter()
    .flat_map(|p| p.meta.tags.iter().cloned())
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect();
  tags.sort();
  tags
}
