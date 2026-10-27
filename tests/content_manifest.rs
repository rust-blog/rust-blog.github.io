//! Regression tests for the build-time content manifest.
//!
//! `build.rs` is now the only place that parses and renders posts. These tests
//! pin the contract: every published source file must appear exactly once in
//! `content::load_posts()`, with the same metadata, the HTML produced by the
//! shared renderer, and the same reading-time estimate. A regression in the
//! generated manifest cannot silently ship a missing, stale, or differently
//! rendered post.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rust_blog::content::load_posts;
use rust_blog::frontmatter::{self, Frontmatter};
use rust_blog::reading_time;
use rust_blog::render_markdown;

struct Source {
  meta: Frontmatter,
  body: String,
  reading_time: usize,
}

#[test]
fn manifest_matches_every_published_source_post() {
  let sources = published_sources();
  let posts = load_posts();

  assert_eq!(
    posts.len(),
    sources.len(),
    "manifest must carry exactly the published posts"
  );

  for post in &posts {
    let source = sources.get(post.slug.as_str()).unwrap_or_else(|| {
      panic!(
        "manifest has post `{}` that no published source file defines",
        post.slug
      )
    });

    assert_eq!(post.meta.title, source.meta.title, "title");
    assert_eq!(post.meta.date, source.meta.date, "date");
    assert_eq!(
      post.meta.description, source.meta.description,
      "description"
    );
    assert_eq!(post.meta.tags, source.meta.tags, "tags");
    assert_eq!(post.meta.author, source.meta.author, "author");
    assert_eq!(post.meta.lang, source.meta.lang, "lang");
    assert!(!post.meta.draft, "drafts must never reach the manifest");
    assert_eq!(post.reading_time, source.reading_time, "reading time");
    assert_eq!(
      post.html,
      render_markdown(&source.body),
      "pre-rendered HTML for `{}` must match the shared renderer",
      post.slug
    );
  }
}

#[test]
fn manifest_order_is_date_desc_then_slug_asc() {
  let posts = load_posts();
  for pair in posts.windows(2) {
    let (newer, older) = (&pair[0], &pair[1]);
    let date_ordered = newer.meta.date > older.meta.date
      || (newer.meta.date == older.meta.date && newer.slug.as_str() < older.slug.as_str());
    assert!(
      date_ordered,
      "posts must be date-desc then slug-asc: `{}` came before `{}`",
      newer.slug, older.slug
    );
  }
}

#[test]
fn every_published_source_file_is_represented_once() {
  let sources = published_sources();
  let posts = load_posts();
  let manifest_slugs = posts
    .iter()
    .map(|post| post.slug.as_str())
    .collect::<std::collections::BTreeSet<_>>();

  assert_eq!(
    manifest_slugs.len(),
    posts.len(),
    "duplicate slugs in the manifest"
  );
  assert_eq!(
    manifest_slugs,
    sources.keys().map(String::as_str).collect(),
    "manifest slugs must match the published source files exactly"
  );
}

fn published_sources() -> BTreeMap<String, Source> {
  let posts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("content/posts");
  let mut sources = BTreeMap::new();

  for entry in fs::read_dir(&posts_dir).expect("content/posts must be readable") {
    let path = entry.expect("readable directory entry").path();
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
      continue;
    }
    let raw = fs::read_to_string(&path).expect("post must be readable");
    let parsed = frontmatter::parse(&raw).expect("build.rs would reject this post");
    if parsed.meta.draft {
      continue;
    }
    let stem = path.file_stem().map(|stem| stem.to_string_lossy());
    let slug = frontmatter::derive_slug(&parsed.meta, stem.as_deref())
      .unwrap_or_else(|e| panic!("invalid slug in {}: {e}", path.display()));
    let reading_time = reading_time::estimate_minutes(&parsed.body);
    let previous = sources.insert(
      slug.as_str().to_string(),
      Source {
        meta: parsed.meta,
        body: parsed.body,
        reading_time,
      },
    );
    assert!(
      previous.is_none(),
      "duplicate published slug `{}` in content/posts",
      slug
    );
  }

  sources
}
