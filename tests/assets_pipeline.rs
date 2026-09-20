//! Content asset pipeline acceptance: every file under `content/assets/`
//! resolves to a fingerprinted `/assets/...` URL, the staged copy Trunk ships
//! is byte-identical to the source, stale files never linger, and markdown
//! images pick the hashed URL up.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use rust_blog::assets;
use rust_blog::fingerprint;
use rust_blog::markdown;

/// Recursively gather every file beneath a directory.
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
  let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
  for entry in entries {
    let path = entry.expect("readable directory entry").path();
    if path.is_dir() {
      collect_files(&path, out);
    } else {
      out.push(path);
    }
  }
}

fn relative(path: &Path, root: &Path) -> String {
  path
    .strip_prefix(root)
    .unwrap_or_else(|e| panic!("{} is not under {}: {e}", path.display(), root.display()))
    .to_string_lossy()
    .replace('\\', "/")
}

fn source_files(root: &Path) -> Vec<PathBuf> {
  let mut files = Vec::new();
  collect_files(&root.join("content/assets"), &mut files);
  files
}

#[test]
fn every_content_asset_resolves_to_a_staged_fingerprinted_file() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  let source_root = root.join("content/assets");
  let staged_root = root.join("dist-assets");

  let files = source_files(root);
  assert!(!files.is_empty(), "expected at least one content asset");

  for file in files {
    let relative = relative(&file, &source_root);
    let key = format!("assets/{relative}");
    let url = assets::resolve(&key).unwrap_or_else(|| panic!("no manifest entry for {key}"));

    // The name carries the 16-hex fingerprint of the file's own bytes.
    let bytes = fs::read(&file).unwrap();
    let hash = fingerprint::content_hash(&bytes);
    let hashed_relative = fingerprint::fingerprinted_name(&relative, &hash);
    assert_eq!(url, format!("/assets/{hashed_relative}"));
    assert!(url.contains(&hash), "{url} does not carry {hash}");

    // The staged file Trunk copies into dist/ is byte-identical to the source.
    let staged = staged_root.join(&hashed_relative);
    assert!(staged.is_file(), "missing staged file {}", staged.display());
    assert_eq!(fs::read(&staged).unwrap(), bytes, "staged copy differs");
  }
}

#[test]
fn staged_directory_contains_only_current_assets() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  let source_root = root.join("content/assets");
  let staged_root = root.join("dist-assets");

  let expected: BTreeSet<String> = source_files(root)
    .iter()
    .map(|file| {
      let relative = relative(file, &source_root);
      let hash = fingerprint::content_hash(&fs::read(file).unwrap());
      fingerprint::fingerprinted_name(&relative, &hash)
    })
    .collect();

  let mut staged = Vec::new();
  collect_files(&staged_root, &mut staged);
  let actual: BTreeSet<String> = staged
    .iter()
    .map(|file| relative(file, &staged_root))
    .collect();

  assert_eq!(
    actual, expected,
    "staged directory does not match content/assets"
  );
}

#[test]
fn markdown_images_resolve_to_fingerprinted_urls() {
  let url = assets::resolve("assets/rust-blog-gear.svg").expect("sample asset");
  for src in [
    "assets/rust-blog-gear.svg",
    "./assets/rust-blog-gear.svg",
    "/assets/rust-blog-gear.svg",
  ] {
    let html = markdown::render(&format!("![the gear]({src})"));
    assert!(html.contains(url), "{src} did not resolve: {html}");
  }
}
