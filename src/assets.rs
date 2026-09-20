//! Resolve markdown references into `content/assets/` to the fingerprinted
//! URLs served from `dist/assets/`.
//!
//! `build.rs` fingerprints every file under `content/assets/`, stages the
//! results in `dist-assets/` (copied verbatim by Trunk), and writes the
//! reference -> URL map into `$OUT_DIR` at build time. Only that small map is
//! embedded here; the image bytes never enter the WASM binary.

use crate::fingerprint::normalize_ref;

include!(concat!(env!("OUT_DIR"), "/content_assets.rs"));

/// The fingerprinted public URL for a markdown reference, or `None` when the
/// reference is not a content asset (external URL, plain path, or a file that
/// is missing from `content/assets/`; `build.rs` already fails the build on
/// missing files, so runtime `None` means "leave the reference untouched").
pub fn resolve(src: &str) -> Option<&'static str> {
  let key = normalize_ref(src)?;
  CONTENT_ASSETS
    .iter()
    .find(|(logical, _)| *logical == key)
    .map(|(_, url)| *url)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolves_the_sample_asset_to_a_hashed_url() {
    let url = resolve("assets/rust-blog-gear.svg").expect("sample asset is in the manifest");
    assert!(url.starts_with("/assets/rust-blog-gear-"), "{url}");
    assert!(url.ends_with(".svg"), "{url}");
  }

  #[test]
  fn leaves_external_and_unknown_references_alone() {
    assert_eq!(resolve("https://example.com/photo.png"), None);
    assert_eq!(resolve("assets/does-not-exist.png"), None);
    assert_eq!(resolve("images/photo.png"), None);
  }
}
