//! RSS round-trip: the feed `build.rs` generates must agree with the posts
//! the app renders. Both sides parse with the same shared frontmatter parser
//! (`src/frontmatter.rs`), so a divergence here is a build bug, not a
//! content bug.

use std::fs;

use rss::Channel;

/// The build script (`build.rs`) writes this into the crate root before any
/// test binary compiles.
const RSS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/rss.xml");

#[test]
fn feed_item_count_matches_published_posts() {
  let posts = rust_blog::content::load_posts();
  let published = posts.iter().filter(|p| !p.meta.draft).count();

  let xml = fs::read_to_string(RSS_PATH).expect("rss.xml must exist - build.rs writes it");
  let channel = Channel::read_from(xml.as_bytes()).expect("rss.xml must be valid RSS");
  let items = channel.items();

  assert_eq!(
    items.len(),
    published,
    "feed must carry exactly the published posts"
  );
  assert_eq!(items.len(), channel.items.len());
}

#[test]
fn every_feed_item_matches_a_post() {
  let posts = rust_blog::content::load_posts();
  let xml = fs::read_to_string(RSS_PATH).unwrap();
  let channel = Channel::read_from(xml.as_bytes()).unwrap();

  for item in channel.items() {
    let title = item.title().expect("item needs a title");
    let link = item.link().expect("item needs a link");
    let slug = link.rsplit('/').next().expect("link needs a slug");
    let post = posts
      .iter()
      .find(|p| p.slug == slug)
      .unwrap_or_else(|| panic!("feed slug {slug:?} has no matching post"));
    assert_eq!(&post.meta.title, title, "feed title must match post title");
    assert!(
      item.pub_date().is_some(),
      "every published post needs a pubDate"
    );
  }
}

#[test]
fn feed_has_required_channel_fields() {
  let xml = fs::read_to_string(RSS_PATH).unwrap();
  let channel = Channel::read_from(xml.as_bytes()).unwrap();
  assert_eq!(channel.language.as_deref(), Some("th"));
  assert!(!channel.title().is_empty());
  assert!(!channel.link().is_empty());
}
