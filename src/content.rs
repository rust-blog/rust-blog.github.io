use crate::frontmatter::{Frontmatter, Lang, Slug};

/// A published blog post, ready to render.
#[derive(Debug, Clone, PartialEq)]
pub struct Post {
  pub slug: Slug,
  pub meta: Frontmatter,
  /// HTML rendered once by `build.rs` and embedded verbatim in the WASM
  /// binary. The app never runs a markdown renderer at startup.
  pub html: &'static str,
  /// Estimated reading time in minutes (computed at build time; Thai and CJK
  /// are counted by character, see `src/reading_time.rs`).
  pub reading_time: usize,
}

/// One row of the build-time generated manifest.
struct GeneratedPost {
  slug: &'static str,
  title: &'static str,
  date: &'static str,
  description: &'static str,
  tags: &'static [&'static str],
  author: Option<&'static str>,
  lang: Lang,
  html: &'static str,
  reading_time: usize,
}

include!(concat!(env!("OUT_DIR"), "/posts_manifest.rs"));

/// Every published post embedded at build time, newest first (the manifest is
/// generated in that order, with the slug as a deterministic tie-break).
///
/// No parsing or rendering happens here: `build.rs` already validated each
/// file and rendered its HTML, so loading a post is a handful of allocations.
pub fn load_posts() -> Vec<Post> {
  POSTS
    .iter()
    .map(|post| Post {
      slug: Slug::new(post.slug).expect("build.rs validated every slug"),
      meta: Frontmatter {
        title: post.title.to_string(),
        date: post.date.to_string(),
        description: post.description.to_string(),
        tags: post.tags.iter().map(|tag| tag.to_string()).collect(),
        author: post.author.map(str::to_string),
        lang: post.lang,
        draft: false,
        slug: None,
      },
      html: post.html,
      reading_time: post.reading_time,
    })
    .collect()
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
    assert!(posts.iter().all(|p| !p.html.is_empty()));
    assert!(posts.iter().all(|p| !p.meta.draft));
  }

  #[test]
  fn embedded_slugs_are_unique() {
    let posts = load_posts();
    let mut seen = std::collections::HashSet::new();
    for post in &posts {
      assert!(
        seen.insert(post.slug.as_str()),
        "duplicate post slug `{}` - two posts would overwrite the same dist/post/<slug>/ page",
        post.slug
      );
    }
  }

  #[test]
  fn manifest_ships_pre_rendered_html_with_fingerprinted_assets() {
    let posts = load_posts();
    let welcome = posts
      .iter()
      .find(|p| p.slug == "welcome")
      .expect("welcome post is embedded");
    assert!(welcome.html.contains("<h2>Powered by Rust 100%</h2>"));
    assert!(
      !welcome.html.contains("assets/rust-blog-gear.svg"),
      "content asset references must already be fingerprinted at build time"
    );
    assert!(welcome.reading_time >= 1);
  }

  #[test]
  fn same_date_posts_have_a_deterministic_order() {
    let posts = load_posts();
    for pair in posts.windows(2) {
      if pair[0].meta.date == pair[1].meta.date {
        assert!(
          pair[0].slug.as_str() <= pair[1].slug.as_str(),
          "same-date posts must be ordered by slug: {} then {}",
          pair[0].slug,
          pair[1].slug
        );
      }
    }
  }
}
