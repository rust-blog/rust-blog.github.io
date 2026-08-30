#[path = "src/frontmatter.rs"]
mod frontmatter;

#[path = "src/site.rs"]
#[allow(dead_code)]
mod site;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use rss::{CategoryBuilder, ChannelBuilder, Guid, ItemBuilder};

/// A published post ready to feed the RSS feed, the sitemap, and the linter.
struct Published {
  path: PathBuf,
  meta: frontmatter::Frontmatter,
  slug: String,
}

fn main() {
  println!("cargo:rerun-if-changed=content");

  let manifest = env!("CARGO_MANIFEST_DIR");
  let posts_dir = Path::new(manifest).join("content").join("posts");

  let mut posts: Vec<Published> = Vec::new();
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
    let stem = path.file_stem().map(|s| s.to_string_lossy());
    let slug = frontmatter::derive_slug(&parsed.meta, stem.as_deref());
    if !parsed.meta.draft {
      posts.push(Published {
        path,
        meta: parsed.meta,
        slug,
      });
    }
  }

  // Advisory linter (missing description, future date, single-use tags).
  let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
  let mut tag_counts: HashMap<String, usize> = HashMap::new();
  for post in &posts {
    for tag in &post.meta.tags {
      *tag_counts.entry(tag.clone()).or_default() += 1;
    }
  }
  for post in &posts {
    for warning in frontmatter::lint_post(&post.meta, &today, &tag_counts) {
      eprintln!("warning: {}: {warning}", post.path.display());
    }
  }

  posts.sort_by(|a, b| date(&b.meta).cmp(date(&a.meta)));

  let items: Vec<rss::Item> = posts
    .iter()
    .map(|post| {
      let link = format!("{}/post/{}", site::SITE_URL, post.slug);
      let pub_date = parse_date(date(&post.meta)).to_rfc2822();
      ItemBuilder::default()
        .title(post.meta.title.clone())
        .link(link.clone())
        .guid(Guid {
          value: link,
          permalink: true,
        })
        .description(post.meta.description.clone())
        .pub_date(pub_date)
        .categories(
          post
            .meta
            .tags
            .iter()
            .cloned()
            .map(|t| CategoryBuilder::default().name(t).build())
            .collect::<Vec<_>>(),
        )
        .build()
    })
    .collect();

  let channel = ChannelBuilder::default()
    .title(site::TITLE)
    .link(site::SITE_URL)
    .description(site::DESCRIPTION)
    .language(Some("th".to_string()))
    .generator(Some("rust-blog (Leptos + Trunk)".to_string()))
    .items(items)
    .build();

  write(manifest, "rss.xml", &channel.to_string());
  write_sitemap(manifest, &posts);
  write_robots(manifest);
  write_manifest(manifest, &posts);
}

/// `posts-manifest.json`: a tiny sidecar consumed by the Trunk `post_build`
/// hook (`scripts/post_build.py`) so it can emit one static `index.html` per
/// post. That gives GitHub Pages (and Facebook's crawler) a real 200 page at
/// `/post/<slug>` instead of a 404, with per-post Open Graph tags.
fn write_manifest(manifest: &str, posts: &[Published]) {
  let mut buf = String::from("[\n");
  for (i, post) in posts.iter().enumerate() {
    if i > 0 {
      buf.push(',');
    }
    buf.push('\n');
    buf.push_str(&format!(
      "  {{\"slug\":{s}, \"title\":{t}, \"description\":{d}}}",
      s = json_str(&post.slug),
      t = json_str(&post.meta.title),
      d = json_str(&post.meta.description),
    ));
  }
  buf.push_str("\n]\n");
  write(manifest, "posts-manifest.json", &buf);
}

/// Minimal JSON string escaper - no external dependency needed.
fn json_str(s: &str) -> String {
  let mut o = String::with_capacity(s.len() + 2);
  o.push('"');
  for c in s.chars() {
    match c {
      '"' => o.push_str("\\\""),
      '\\' => o.push_str("\\\\"),
      '\n' => o.push_str("\\n"),
      '\r' => o.push_str("\\r"),
      '\t' => o.push_str("\\t"),
      c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
      c => o.push(c),
    }
  }
  o.push('"');
  o
}

/// Write a build artifact, failing the build loudly on any error - a
/// silently-missing file is exactly what the rest of this file refuses to ship.
fn write(manifest: &str, name: &str, contents: &str) {
  let path = Path::new(manifest).join(name);
  fs::write(&path, contents).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// `sitemap.xml`: home, about, and every published post with its date.
fn write_sitemap(manifest: &str, posts: &[Published]) {
  let mut xml = String::from(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
  );
  xml.push_str(&format!("  <url><loc>{}/</loc></url>\n", site::SITE_URL));
  xml.push_str(&format!(
    "  <url><loc>{}/about</loc></url>\n",
    site::SITE_URL
  ));
  for post in posts {
    xml.push_str(&format!(
      "  <url><loc>{}/post/{}</loc><lastmod>{}</lastmod></url>\n",
      site::SITE_URL,
      post.slug,
      post.meta.date
    ));
  }
  xml.push_str("</urlset>\n");
  write(manifest, "sitemap.xml", &xml);
}

/// `robots.txt`: everything is crawlable; point crawlers at the sitemap.
fn write_robots(manifest: &str) {
  let robots = format!(
    "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
    site::SITE_URL
  );
  write(manifest, "robots.txt", &robots);
}

/// The validated `YYYY-MM-DD` date, guaranteed present by `frontmatter::parse`.
fn date(fm: &frontmatter::Frontmatter) -> &str {
  fm.date.as_str()
}

/// Format a validated `YYYY-MM-DD` date as an RFC 2822 pubDate.
fn parse_date(date: &str) -> DateTime<Utc> {
  NaiveDate::parse_from_str(date, "%Y-%m-%d")
    .unwrap_or_else(|e| panic!("date {date:?} failed calendar check: {e}"))
    .and_hms_opt(0, 0, 0)
    .expect("midnight always exists")
    .and_utc()
}
