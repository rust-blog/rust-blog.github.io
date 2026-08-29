#[path = "src/frontmatter.rs"]
mod frontmatter;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use rss::{CategoryBuilder, ChannelBuilder, Guid, ItemBuilder};

const SITE_URL: &str = "https://rust-blog.github.io";
const SITE_TITLE: &str = "rust-blog";
const SITE_DESCRIPTION: &str =
  "A blog written in Rust and Leptos, running in your browser as WebAssembly";

fn main() {
  println!("cargo:rerun-if-changed=content");

  let manifest = env!("CARGO_MANIFEST_DIR");
  let posts_dir = Path::new(manifest).join("content").join("posts");

  let mut posts: Vec<(PathBuf, frontmatter::Frontmatter, String)> = Vec::new();
  for entry in
    fs::read_dir(&posts_dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", posts_dir.display()))
  {
    let entry =
      entry.unwrap_or_else(|e| panic!("cannot read entry in {}: {e}", posts_dir.display()));
    let path = entry.path();
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
      continue;
    }
    let raw =
      fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    // Any malformed post fails the build loudly instead of shipping a
    // silently-broken page or a malformed feed.
    let parsed = frontmatter::parse(&raw)
      .unwrap_or_else(|e| panic!("invalid frontmatter in {}: {e}", path.display()));
    for warning in &parsed.warnings {
      eprintln!("warning: {}: {warning}", path.display());
    }
    let slug = slug_from(&parsed.meta, &path);
    if !parsed.meta.draft {
      posts.push((path, parsed.meta, slug));
    }
  }

  // Advisory linter (missing description, future date, single-use tags).
  let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
  let mut tag_counts: HashMap<String, usize> = HashMap::new();
  for (_, fm, _) in &posts {
    for tag in &fm.tags {
      *tag_counts.entry(tag.clone()).or_default() += 1;
    }
  }
  for (path, fm, _) in &posts {
    for warning in frontmatter::lint_post(fm, &today, &tag_counts) {
      eprintln!("warning: {}: {warning}", path.display());
    }
  }

  posts.sort_by(|a, b| date(&b.1).cmp(date(&a.1)));

  let items: Vec<rss::Item> = posts
    .iter()
    .map(|(_, fm, slug)| {
      let link = format!("{SITE_URL}/post/{slug}");
      let pub_date = parse_date(date(fm)).to_rfc2822();
      ItemBuilder::default()
        .title(fm.title.clone())
        .link(link.clone())
        .guid(Guid {
          value: link,
          permalink: true,
        })
        .description(fm.description.clone())
        .pub_date(pub_date)
        .categories(
          fm.tags
            .iter()
            .cloned()
            .map(|t| CategoryBuilder::default().name(t).build())
            .collect::<Vec<_>>(),
        )
        .build()
    })
    .collect();

  let channel = ChannelBuilder::default()
    .title(SITE_TITLE)
    .link(SITE_URL)
    .description(SITE_DESCRIPTION)
    .language(Some("th".to_string()))
    .generator(Some("rust-blog (Leptos + Trunk)".to_string()))
    .items(items)
    .build();

  let out = channel.to_string();
  let _ = fs::write(Path::new(manifest).join("rss.xml"), out);

  write_sitemap(manifest, &posts);
  write_robots(manifest);
}

/// `sitemap.xml`: home, about, and every published post with its date.
fn write_sitemap(manifest: &str, posts: &[(PathBuf, frontmatter::Frontmatter, String)]) {
  let mut xml = String::from(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
  );
  xml.push_str(&format!("  <url><loc>{SITE_URL}/</loc></url>\n"));
  xml.push_str(&format!("  <url><loc>{SITE_URL}/about</loc></url>\n"));
  for (_, fm, slug) in posts {
    xml.push_str(&format!(
      "  <url><loc>{SITE_URL}/post/{slug}</loc><lastmod>{}</lastmod></url>\n",
      fm.date
    ));
  }
  xml.push_str("</urlset>\n");
  let _ = fs::write(Path::new(manifest).join("sitemap.xml"), xml);
}

/// `robots.txt`: everything is crawlable; point crawlers at the sitemap.
fn write_robots(manifest: &str) {
  let robots = format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n");
  let _ = fs::write(Path::new(manifest).join("robots.txt"), robots);
}

/// The validated `YYYY-MM-DD` date, guaranteed present by `frontmatter::parse`.
fn date(fm: &frontmatter::Frontmatter) -> &str {
  fm.date.as_str()
}

/// A post's URL slug: the frontmatter `slug` override, else the file stem.
fn slug_from(fm: &frontmatter::Frontmatter, path: &Path) -> String {
  fm.slug
    .clone()
    .or_else(|| path.file_stem().map(|s| s.to_string_lossy().to_string()))
    .unwrap_or_default()
}

/// Format a validated `YYYY-MM-DD` date as an RFC 2822 pubDate.
fn parse_date(date: &str) -> DateTime<Utc> {
  NaiveDate::parse_from_str(date, "%Y-%m-%d")
    .unwrap_or_else(|e| panic!("date {date:?} failed calendar check: {e}"))
    .and_hms_opt(0, 0, 0)
    .expect("midnight always exists")
    .and_utc()
}
