//! Render fuzzing: whatever bytes an author drops into a post, the markdown
//! renderer and frontmatter parser must never panic. If pulldown-cmark or
//! syntect ever rejects an input, this test pins the reproducer.

use proptest::prelude::*;
use rust_blog::frontmatter;
use rust_blog::markdown;

fn any_string() -> impl Strategy<Value = String> {
  prop::collection::vec(any::<char>(), 0..1024).prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn any_markdown_renders_without_panicking(md in any_string()) {
        let _ = markdown::render(&md);
    }

    #[test]
    fn any_frontmatter_parses_to_result_without_panicking(raw in any_string()) {
        let _ = frontmatter::parse(&raw);
    }

    #[test]
    fn frontmatter_with_body_never_panics(head in any_string(), body in any_string()) {
        let raw = format!("---\n{head}\n---\n{body}");
        let _ = frontmatter::parse(&raw);
    }
}
