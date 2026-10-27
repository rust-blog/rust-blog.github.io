#[path = "src/fingerprint.rs"]
mod fingerprint;

#[path = "src/frontmatter.rs"]
#[allow(dead_code)]
mod frontmatter;

#[path = "src/markdown.rs"]
#[allow(dead_code)]
mod markdown;

#[path = "src/reading_time.rs"]
#[allow(dead_code)]
mod reading_time;

#[path = "src/site.rs"]
#[allow(dead_code)]
mod site;

#[path = "src/xml.rs"]
mod xml;

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use rss::{CategoryBuilder, ChannelBuilder, Guid, ItemBuilder};

/// A published post ready to feed the RSS feed, the sitemap, the linter, and
/// the generated app manifest.
struct Published {
  path: PathBuf,
  meta: frontmatter::Frontmatter,
  slug: frontmatter::Slug,
  body: String,
}

fn main() {
  println!("cargo:rerun-if-changed=content/posts");
  println!("cargo:rerun-if-changed=content/assets");
  println!("cargo:rerun-if-changed=assets/TOML.sublime-syntax");
  println!("cargo:rerun-if-changed=assets/Catppuccin-Mocha.tmTheme");

  let manifest = env!("CARGO_MANIFEST_DIR");
  let assets = stage_content_assets(manifest);
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
    // A markdown image that points into `content/assets/` but has no file to
    // resolve to would ship a broken image; fail the build loudly instead.
    for reference in fingerprint::asset_references(&parsed.body) {
      if !assets.contains_key(&reference) {
        panic!(
          "{} references missing content asset `{reference}`: expected content/{reference}",
          path.display()
        );
      }
    }
    let stem = path.file_stem().map(|s| s.to_string_lossy());
    // A slug becomes a URL path segment and a directory name, so an unsafe
    // one (absolute, `..`, separators) fails the build instead of escaping
    // `dist/` or corrupting the sitemap.
    let slug = frontmatter::derive_slug(&parsed.meta, stem.as_deref())
      .unwrap_or_else(|e| panic!("invalid post slug in {}: {e}", path.display()));
    if !parsed.meta.draft {
      posts.push(Published {
        path,
        meta: parsed.meta,
        slug,
        body: parsed.body,
      });
    }
  }

  // Two published posts sharing a slug would silently overwrite the same
  // dist/post/<slug>/ page and emit duplicate RSS/sitemap URLs; fail loudly.
  let mut seen: HashMap<&str, &Path> = HashMap::new();
  for post in &posts {
    if let Some(previous) = seen.insert(post.slug.as_str(), &post.path) {
      panic!(
        "duplicate post slug `{}`: {} and {}",
        post.slug,
        previous.display(),
        post.path.display()
      );
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

  // Newest first, with the slug as a deterministic tie-break: the generated
  // manifest fixes the order the app and the feed use, so same-date posts can
  // never flip between machines or builds.
  posts.sort_by(|a, b| {
    date(&b.meta)
      .cmp(date(&a.meta))
      .then_with(|| a.slug.as_str().cmp(b.slug.as_str()))
  });

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
  write_assets_manifest(&assets);
  write_posts_manifest(&posts, &assets);
}

/// Write a build artifact, failing the build loudly on any error - a
/// silently-missing file is exactly what the rest of this file refuses to ship.
fn write(manifest: &str, name: &str, contents: &str) {
  let path = Path::new(manifest).join(name);
  fs::write(&path, contents).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// `sitemap.xml`: home, about, and every published post with its date.
///
/// Every interpolated value is XML-escaped. Slugs are already restricted to
/// `[a-z0-9-]` by the type system, but the sitemap must not depend on an
/// invariant defined in another module to stay well-formed.
fn write_sitemap(manifest: &str, posts: &[Published]) {
  let mut out = String::from(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
  );
  let root = xml::escape(site::SITE_URL);
  out.push_str(&format!("  <url><loc>{root}/</loc></url>\n"));
  out.push_str(&format!("  <url><loc>{root}/about</loc></url>\n"));
  for post in posts {
    out.push_str(&format!(
      "  <url><loc>{root}/post/{}</loc><lastmod>{}</lastmod></url>\n",
      xml::escape(post.slug.as_str()),
      xml::escape(&post.meta.date)
    ));
  }
  out.push_str("</urlset>\n");
  write(manifest, "sitemap.xml", &out);
}

/// `robots.txt`: everything is crawlable; point crawlers at the sitemap.
fn write_robots(manifest: &str) {
  let robots = format!(
    "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
    site::SITE_URL
  );
  write(manifest, "robots.txt", &robots);
}

/// Fingerprint every file under `content/assets/` into `dist-assets/` and
/// return the manifest mapping logical references (`assets/<path>`) to the
/// public URLs the site serves (`/assets/<name>-<hash>.<ext>`).
///
/// The staged directory is rebuilt from scratch on every build so deleted or
/// renamed assets do not linger in `dist/` forever.
fn stage_content_assets(manifest: &str) -> BTreeMap<String, String> {
  let source_root = Path::new(manifest).join("content").join("assets");
  let staged_root = Path::new(manifest).join("dist-assets");
  if staged_root.exists() {
    fs::remove_dir_all(&staged_root)
      .unwrap_or_else(|e| panic!("cannot clear {}: {e}", staged_root.display()));
  }
  fs::create_dir_all(&staged_root)
    .unwrap_or_else(|e| panic!("cannot create {}: {e}", staged_root.display()));

  let mut files = Vec::new();
  if source_root.exists() {
    collect_files(&source_root, &mut files);
  }

  let mut entries = BTreeMap::new();
  for file in files {
    let relative = file
      .strip_prefix(&source_root)
      .expect("collected files live under content/assets")
      .to_string_lossy()
      .replace('\\', "/");
    let bytes = fs::read(&file).unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
    let hash = fingerprint::content_hash(&bytes);
    let hashed_relative = fingerprint::fingerprinted_name(&relative, &hash);
    let staged = staged_root.join(&hashed_relative);
    if let Some(parent) = staged.parent() {
      fs::create_dir_all(parent)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", parent.display()));
    }
    fs::write(&staged, &bytes).unwrap_or_else(|e| panic!("cannot write {}: {e}", staged.display()));
    entries.insert(
      format!("{}{relative}", fingerprint::ASSET_PREFIX),
      format!("{}{hashed_relative}", fingerprint::PUBLIC_URL_PREFIX),
    );
  }
  entries
}

/// Recursively gather every file beneath a directory.
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
  let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
  for entry in entries {
    let entry = entry.unwrap_or_else(|e| panic!("cannot read entry in {}: {e}", dir.display()));
    let path = entry.path();
    if path.is_dir() {
      collect_files(&path, out);
    } else {
      out.push(path);
    }
  }
}

/// Write the reference -> URL map as a Rust source file in `$OUT_DIR`, so the
/// app and its tests can `include!` it without a runtime fetch.
fn write_assets_manifest(entries: &BTreeMap<String, String>) {
  let out_dir = env::var("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
  let mut generated = String::from(
    "// @generated by build.rs: content asset reference -> fingerprinted public URL.\n\
     pub static CONTENT_ASSETS: &[(&str, &str)] = &[\n",
  );
  for (key, url) in entries {
    generated.push_str(&format!("  ({key:?}, {url:?}),\n"));
  }
  generated.push_str("];\n");
  let path = Path::new(&out_dir).join("content_assets.rs");
  fs::write(&path, generated).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// Render every published post at build time and write the app's entire
/// content model (metadata + HTML + reading time) into `$OUT_DIR`.
///
/// This is what keeps `pulldown-cmark` and `syntect` out of the WASM binary:
/// the app embeds a static manifest and never parses or renders markdown at
/// startup. Code blocks are highlighted once, here, instead of in every
/// reader's browser.
fn write_posts_manifest(posts: &[Published], assets: &BTreeMap<String, String>) {
  let resolve_asset = |src: &str| -> Option<String> {
    fingerprint::normalize_ref(src).and_then(|key| assets.get(&key).cloned())
  };

  let mut generated = String::from(
    "// @generated by build.rs: the full published-post manifest (metadata,\n\
     // pre-rendered HTML, and reading time). Do not edit by hand.\n\
     static POSTS: &[GeneratedPost] = &[\n",
  );
  for post in posts {
    let html = markdown::render_with(&post.body, &resolve_asset);
    let reading_time = reading_time::estimate_minutes(&post.body);
    let tags = post
      .meta
      .tags
      .iter()
      .map(|tag| format!("{tag:?}"))
      .collect::<Vec<_>>()
      .join(", ");
    let author = post
      .meta
      .author
      .as_deref()
      .map(|author| format!("Some({author:?})"))
      .unwrap_or_else(|| "None".to_string());
    let lang = match post.meta.lang {
      frontmatter::Lang::Th => "Lang::Th",
      frontmatter::Lang::En => "Lang::En",
    };
    generated.push_str("  GeneratedPost {\n");
    generated.push_str(&format!("    slug: {:?},\n", post.slug.as_str()));
    generated.push_str(&format!("    title: {:?},\n", post.meta.title));
    generated.push_str(&format!("    date: {:?},\n", post.meta.date));
    generated.push_str(&format!("    description: {:?},\n", post.meta.description));
    generated.push_str(&format!("    tags: &[{tags}],\n"));
    generated.push_str(&format!("    author: {author},\n"));
    generated.push_str(&format!("    lang: {lang},\n"));
    generated.push_str(&format!("    html: {html:?},\n"));
    generated.push_str(&format!("    reading_time: {reading_time},\n"));
    generated.push_str("  },\n");
  }
  generated.push_str("];\n");

  let out_dir = env::var("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
  let path = Path::new(&out_dir).join("posts_manifest.rs");
  fs::write(&path, generated).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
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
