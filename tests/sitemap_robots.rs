//! Sitemap + robots: the SEO artifacts `build.rs` writes must cover exactly
//! the published URLs and point crawlers at the right feed.

use std::fs;

const SITEMAP_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/sitemap.xml");
const ROBOTS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/robots.txt");

#[test]
fn sitemap_lists_home_about_and_every_post() {
  let posts = rust_blog::content::load_posts();
  let xml = fs::read_to_string(SITEMAP_PATH).expect("sitemap.xml must exist - build.rs writes it");

  assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
  assert!(xml.contains("<loc>https://rust-blog.github.io/</loc>"));
  assert!(xml.contains("<loc>https://rust-blog.github.io/about</loc>"));

  for post in posts.iter().filter(|p| !p.meta.draft) {
    let loc = format!("https://rust-blog.github.io/post/{}", post.slug);
    assert!(xml.contains(&loc), "sitemap must list {loc}");
    assert!(
      xml.contains(&format!("{loc}</loc><lastmod>{}</lastmod>", post.meta.date)),
      "sitemap must carry the post date as lastmod for {loc}"
    );
  }
}

#[test]
fn sitemap_url_count_matches_published_posts_plus_static_pages() {
  let posts = rust_blog::content::load_posts();
  let published = posts.iter().filter(|p| !p.meta.draft).count();
  let xml = fs::read_to_string(SITEMAP_PATH).unwrap();

  // home + about + one per published post
  assert_eq!(xml.matches("<url>").count(), published + 2);
}

#[test]
fn robots_points_at_sitemap() {
  let robots = fs::read_to_string(ROBOTS_PATH).expect("robots.txt must exist - build.rs writes it");
  assert!(robots.starts_with("User-agent: *\nAllow: /\n"));
  assert!(robots.contains("Sitemap: https://rust-blog.github.io/sitemap.xml"));
}
