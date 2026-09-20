//! Content asset fingerprinting, shared by `build.rs` (which stages the
//! fingerprinted files) and the host test suite (which verifies them).
//!
//! The fingerprint is the first 8 bytes of a file's SHA-256, formatted as 16
//! lowercase hex characters - the same shape Trunk gives the CSS/JS it
//! bundles, so `dist/` names stay consistent across the whole build.

/// Prefix every manifest key and markdown reference shares.
pub const ASSET_PREFIX: &str = "assets/";

/// Public URL prefix of the directory Trunk copies the staged files into.
pub const PUBLIC_URL_PREFIX: &str = "/assets/";

/// Normalize a markdown reference into a manifest key (`assets/<path>`).
///
/// Returns `None` for anything outside `content/assets/`: absolute URLs, data
/// URIs, anchors, and paths that climb out with `..`. Leading `./` and `/`
/// are accepted so `assets/x.png`, `./assets/x.png`, and `/assets/x.png` all
/// describe the same file.
pub fn normalize_ref(src: &str) -> Option<String> {
  let trimmed = src.trim();
  let without_dot = trimmed.strip_prefix("./").unwrap_or(trimmed);
  let without_slash = without_dot.strip_prefix('/').unwrap_or(without_dot);
  let normalized = without_slash.replace('\\', "/");
  let is_asset = normalized.starts_with(ASSET_PREFIX) && normalized.len() > ASSET_PREFIX.len();
  let escapes = normalized.split('/').any(|segment| segment == "..");
  (is_asset && !escapes).then_some(normalized)
}

#[cfg(not(target_arch = "wasm32"))]
pub use build_time::{asset_references, content_hash, fingerprinted_name};

/// Build-time helpers. The app never hashes file bytes at runtime: `build.rs`
/// runs these on the host, stages the results, and embeds only the small
/// reference -> URL manifest, so the WASM binary stays image-free.
#[cfg(not(target_arch = "wasm32"))]
mod build_time {
  use std::path::Path;

  use pulldown_cmark::{Event, Options, Parser, Tag};
  use sha2::{Digest, Sha256};

  /// Fingerprint raw bytes as 16 lowercase hex characters (the first 8 bytes
  /// of SHA-256, matching Trunk's `filehash` format).
  pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let first: [u8; 8] = digest[..8].try_into().expect("sha256 is 32 bytes");
    format!("{:016x}", u64::from_be_bytes(first))
  }

  /// Insert the fingerprint before the extension: `pics/a.png` becomes
  /// `pics/a-<hash>.png`. Extension-less names get `name-<hash>`; multi-dot
  /// names fingerprint the last extension (`archive.tar.gz` ->
  /// `archive.tar-<hash>.gz`).
  pub fn fingerprinted_name(name: &str, hash: &str) -> String {
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let file = match path
      .extension()
      .and_then(|e| e.to_str())
      .filter(|e| !e.is_empty())
    {
      Some(ext) => format!("{stem}-{hash}.{ext}"),
      None => format!("{stem}-{hash}"),
    };
    match parent {
      Some(dir) => format!("{}/{file}", dir.to_string_lossy().replace('\\', "/")),
      None => file,
    }
  }

  /// Every image destination in a markdown body that points into
  /// `content/assets/`, normalized to a manifest key. `build.rs` fails the
  /// build when one of these has no asset to resolve to.
  pub fn asset_references(md: &str) -> Vec<String> {
    Parser::new_ext(md, Options::all())
      .filter_map(|event| match event {
        Event::Start(Tag::Image { dest_url, .. }) => super::normalize_ref(&dest_url),
        _ => None,
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_matches_a_known_sha256_vector() {
    // First 8 bytes of SHA-256("hello").
    assert_eq!(content_hash(b"hello"), "2cf24dba5fb0a30e");
  }

  #[test]
  fn hash_changes_with_content() {
    assert_ne!(content_hash(b"one"), content_hash(b"two"));
    assert_ne!(content_hash(b"one"), content_hash(b"one "));
  }

  #[test]
  fn name_keeps_directories_and_extension() {
    assert_eq!(
      fingerprinted_name("pics/hello.png", "deadbeefdeadbeef"),
      "pics/hello-deadbeefdeadbeef.png"
    );
    assert_eq!(
      fingerprinted_name("archive.tar.gz", "deadbeefdeadbeef"),
      "archive.tar-deadbeefdeadbeef.gz"
    );
    assert_eq!(
      fingerprinted_name("LICENSE", "deadbeefdeadbeef"),
      "LICENSE-deadbeefdeadbeef"
    );
  }

  #[test]
  fn normalize_ref_accepts_local_shapes() {
    for src in ["assets/a.png", "./assets/a.png", "/assets/a.png"] {
      assert_eq!(normalize_ref(src).as_deref(), Some("assets/a.png"), "{src}");
    }
  }

  #[test]
  fn normalize_ref_rejects_everything_else() {
    for src in [
      "https://example.com/a.png",
      "//cdn.example.com/a.png",
      "data:image/png;base64,AAAA",
      "other/a.png",
      "assets/../secret.png",
      "assets/",
      "#anchor",
    ] {
      assert_eq!(normalize_ref(src), None, "{src}");
    }
  }

  #[test]
  fn asset_references_finds_local_images_only() {
    let md = "![a](assets/a.png) ![b](https://example.com/b.png) [c](assets/c.pdf)";
    assert_eq!(asset_references(md), vec!["assets/a.png"]);
  }
}
