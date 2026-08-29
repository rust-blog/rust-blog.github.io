use crate::frontmatter::{self, Frontmatter};
use crate::markdown;
use include_dir::{Dir, File, include_dir};

/// Embedded content directory. Drop a new `content/posts/<slug>.md` file in and
/// it is picked up automatically at compile time - no code changes required.
static CONTENT_DIR: Dir = include_dir!("content");

/// A fully processed blog post, ready to render.
#[derive(Debug, Clone, PartialEq)]
pub struct Post {
  pub slug: String,
  pub meta: Frontmatter,
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
///
/// Uses the shared [`frontmatter::parse`] so the in-app render and the RSS
/// feed (built in `build.rs`) can never disagree about a post. Errors cannot
/// reach runtime: `build.rs` already fails the build on any malformed post.
fn parse_post(raw: &str, path: &std::path::Path) -> Option<Post> {
  let parsed = frontmatter::parse(raw).ok()?;

  let stem = path.file_stem().map(|s| s.to_string_lossy());
  let slug = frontmatter::derive_slug(&parsed.meta, stem.as_deref());

  let html = markdown::render(&parsed.body);
  let reading_time =
    ((parsed.body.split_whitespace().count() as f64 / 200.0).ceil() as usize).max(1);

  Some(Post {
    slug,
    meta: parsed.meta,
    body: parsed.body,
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
