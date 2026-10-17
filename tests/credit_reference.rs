//! One shared block closes every post that credits sources: a `---`
//! separator, the `**Credit & Reference:**` heading followed directly by
//! `- [label](url)` bullets (an optional short description may follow the
//! link). Legacy heading variants, a blank line after the heading, bullets
//! that do not start with a link, and stray prose after the block all fail
//! here, so the house pattern cannot drift again (issue #41).

use rust_blog::frontmatter;

const HEADING: &str = "**Credit & Reference:**";

/// Headings posts used before the pattern was unified.
const LEGACY_HEADINGS: [&str; 6] = [
  "## อ้างอิง",
  "## แหล่งข้อมูลอ้างอิง",
  "## References",
  "**References:**",
  "**References**",
  "**แหล่งอ้างอิงที่ใช้ในบทความนี้**",
];

fn posts() -> Vec<(String, String)> {
  let mut posts = Vec::new();
  for entry in std::fs::read_dir("content/posts").expect("content/posts must exist") {
    let path = entry.expect("directory entry must be readable").path();
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
      continue;
    }
    let raw = std::fs::read_to_string(&path).expect("post must be readable");
    let parsed = frontmatter::parse(&raw).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    posts.push((path.display().to_string(), parsed.body));
  }
  assert!(!posts.is_empty(), "content/posts must not be empty");
  posts
}

#[test]
fn posts_do_not_use_a_legacy_reference_heading() {
  for (path, body) in posts() {
    for legacy in LEGACY_HEADINGS {
      assert!(
        !body.contains(legacy),
        "{path} still uses the legacy heading {legacy:?}"
      );
    }
  }
}

#[test]
fn credit_reference_block_is_the_final_section() {
  for (path, body) in posts() {
    let Some(heading_at) = body.find(HEADING) else {
      continue;
    };
    assert_eq!(
      body.matches(HEADING).count(),
      1,
      "{path} must contain exactly one Credit & Reference block"
    );

    let before = &body[..heading_at];
    assert_eq!(
      before.trim_end().lines().next_back(),
      Some("---"),
      "{path} must separate the block with a `---` line"
    );

    let mut bullets = 0;
    for line in body[heading_at + HEADING.len()..].lines() {
      if line.is_empty() {
        continue;
      }
      assert!(
        line.starts_with("- ["),
        "{path}: every bullet must start with a markdown link, found {line:?}"
      );
      assert!(
        line.contains("]("),
        "{path}: every bullet must contain a markdown link, found {line:?}"
      );
      bullets += 1;
    }
    assert!(bullets > 0, "{path} block must list at least one link");
  }
}

#[test]
fn heading_is_followed_directly_by_the_first_bullet() {
  for (path, body) in posts() {
    let Some(heading_at) = body.find(HEADING) else {
      continue;
    };
    let rest = &body[heading_at + HEADING.len()..];
    let mut lines = rest.lines();
    assert_eq!(
      lines.next(),
      Some(""),
      "{path}: the heading must end its own line"
    );
    assert!(
      lines.next().is_some_and(|line| line.starts_with("- ")),
      "{path}: the first bullet must follow the heading with no blank line"
    );
  }
}
