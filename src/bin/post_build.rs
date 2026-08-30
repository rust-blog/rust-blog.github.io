//! Emit one static `dist/post/<slug>/index.html` per published post.
//!
//! GitHub Pages (and Facebook's crawler) only serve files that physically
//! exist, so a history-mode SPA route like `/post/<slug>` would otherwise hit
//! the generic `404.html` fallback. This binary copies the already-built
//! `dist/index.html` (with its hashed JS/CSS) into `dist/post/<slug>/` and
//! swaps the static Open Graph / Twitter / `<title>` tags for per-post values.
//! Real browsers still boot the SPA; crawlers now get a 200 page with the
//! right share card.
//!
//! Posts are loaded through the same `rust_blog::content` code the app uses,
//! so slugs and frontmatter can never drift between the site and these pages.
//! Run as a CI step after `trunk build` (see `.github/workflows/deploy.yml`).

use std::fs;
use std::path::Path;

use rust_blog::content::load_posts;
use rust_blog::site;

fn main() {
  let template = fs::read_to_string("dist/index.html")
    .unwrap_or_else(|e| panic!("cannot read dist/index.html: {e}"));

  for post in load_posts() {
    let url = format!("{}/post/{}", site::SITE_URL, post.slug);
    let page = render(&template, &post.meta.title, &post.meta.description, &url);
    let dir = Path::new("dist").join("post").join(&post.slug);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let path = dir.join("index.html");
    fs::write(&path, page).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    println!("post_build: wrote {}", path.display());
  }
}

fn render(template: &str, title: &str, description: &str, url: &str) -> String {
  let mut out = set_title(template, title);
  out = set_meta(&out, "property", "og:title", title);
  out = set_meta(&out, "property", "og:description", description);
  out = set_meta(&out, "property", "og:type", "article");
  out = set_meta(&out, "property", "og:url", url);
  out = set_meta(&out, "name", "twitter:title", title);
  out = set_meta(&out, "name", "twitter:description", description);
  out
}

/// Replace the contents of `<title>...</title>`.
fn set_title(html: &str, title: &str) -> String {
  let start = html.find("<title>").expect("template has a <title> tag");
  let end = html[start..]
    .find("</title>")
    .expect("template closes </title>")
    + start;
  let mut out = String::with_capacity(html.len() + 64);
  out.push_str(&html[..start]);
  out.push_str("<title>");
  out.push_str(&escape_html(title));
  out.push_str(" - rust-blog");
  out.push_str(&html[end..]);
  out
}

/// Replace the `content="..."` value of a `<meta {kind}="{name}" .../>` tag.
fn set_meta(html: &str, kind: &str, name: &str, content: &str) -> String {
  let needle = format!("{kind}=\"{name}\"");
  let mut out = String::with_capacity(html.len() + 64);
  let mut rest = html;
  while let Some(idx) = rest.find(&needle) {
    let tag_start = rest[..idx]
      .rfind("<meta")
      .expect("meta tag before attribute");
    let tag_end = idx
      + rest[idx..]
        .find('>')
        .expect("meta tag closes after attribute");
    out.push_str(&rest[..tag_start]);
    let tag = &rest[tag_start..=tag_end];
    let cpos = tag
      .find("content=\"")
      .expect("meta tag has a content attribute");
    let value_start = cpos + "content=\"".len();
    let value_end = tag[value_start..].find('"').expect("content value closes") + value_start;
    let mut new_tag = String::with_capacity(tag.len() + 64);
    new_tag.push_str(&tag[..value_start]);
    new_tag.push_str(&escape_html(content));
    new_tag.push_str(&tag[value_end..]);
    out.push_str(&new_tag);
    rest = &rest[tag_end + 1..];
  }
  out.push_str(rest);
  out
}

/// Minimal attribute-value escaping for HTML text.
fn escape_html(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 16);
  for c in s.chars() {
    match c {
      '&' => out.push_str("&amp;"),
      '"' => out.push_str("&quot;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      _ => out.push(c),
    }
  }
  out
}
