use std::fs;
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use rss::{CategoryBuilder, ChannelBuilder, Guid, ItemBuilder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Frontmatter {
  title: String,
  date: String,
  #[serde(default)]
  description: String,
  #[serde(default)]
  tags: Vec<String>,
  #[serde(default)]
  draft: bool,
  #[serde(default)]
  slug: Option<String>,
}

const SITE_URL: &str = "https://rust-blog.github.io";
const SITE_TITLE: &str = "rust-blog";
const SITE_DESCRIPTION: &str =
  "A blog written in Rust and Leptos, running in your browser as WebAssembly";

fn main() {
  println!("cargo:rerun-if-changed=content");

  let manifest = env!("CARGO_MANIFEST_DIR");
  let posts_dir = Path::new(manifest).join("content").join("posts");

  let mut posts: Vec<(Frontmatter, String)> = Vec::new();
  if let Ok(entries) = fs::read_dir(&posts_dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.extension().and_then(|e| e.to_str()) != Some("md") {
        continue;
      }
      let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => continue,
      };
      if let Some((fm, slug)) = parse(&raw, &path)
        && !fm.draft
      {
        posts.push((fm, slug));
      }
    }
  }

  posts.sort_by(|a, b| b.0.date.cmp(&a.0.date));

  let items: Vec<rss::Item> = posts
    .iter()
    .map(|(fm, slug)| {
      let link = format!("{SITE_URL}/post/{slug}");
      let pub_date = parse_date(&fm.date).map(|d| d.to_rfc2822());
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
}

/// Split a post file into its frontmatter and derived slug.
fn parse(raw: &str, path: &Path) -> Option<(Frontmatter, String)> {
  let trimmed = raw.strip_prefix('\u{feff}').unwrap_or(raw);
  if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
    return None;
  }
  let rest = &trimmed[3..];
  let end = rest.find("\n---")?;
  let fm_text = &rest[..end];
  let fm: Frontmatter = serde_yaml::from_str(fm_text).ok()?;

  let slug = fm
    .slug
    .clone()
    .or_else(|| path.file_stem().map(|s| s.to_string_lossy().to_string()))
    .unwrap_or_default();

  Some((fm, slug))
}

/// Parse `YYYY-MM-DD` into a UTC datetime at midnight.
fn parse_date(date: &str) -> Option<DateTime<Utc>> {
  let combined = format!("{date} 00:00:00");
  NaiveDateTime::parse_from_str(&combined, "%Y-%m-%d %H:%M:%S")
    .ok()
    .map(|naive| naive.and_utc())
}
