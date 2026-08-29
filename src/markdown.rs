use std::collections::HashMap;
use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Embedded syntax definitions (fancy-regex backend, wasm32-safe).
fn syntax_set() -> &'static SyntaxSet {
  static SS: OnceLock<SyntaxSet> = OnceLock::new();
  SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Embedded theme set; we render with a dark plate in both UI themes.
fn theme() -> &'static syntect::highlighting::Theme {
  static TH: OnceLock<syntect::highlighting::Theme> = OnceLock::new();
  TH.get_or_init(|| {
    let themes = ThemeSet::load_defaults();
    themes
      .themes
      .get("base16-eighties.dark")
      .unwrap_or(&themes.themes["InspiredGitHub"])
      .clone()
  })
}

/// One `##`/`###` heading for the table of contents.
#[derive(Debug, Clone, PartialEq)]
pub struct TocEntry {
  /// 2 or 3.
  pub level: u8,
  /// The slugified id attached to the rendered heading.
  pub id: String,
  /// The heading's plain text.
  pub text: String,
}

/// Output of [`render`]: the HTML plus the extracted table of contents.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
  pub html: String,
  pub toc: Vec<TocEntry>,
}

/// Render markdown source to HTML.
///
/// Enables GitHub-flavoured extensions (tables, strikethrough, task lists,
/// footnotes, and heading attributes) so authors can write rich posts.
/// Fenced code blocks are highlighted with `syntect` at render time - no
/// JavaScript, no CDN - and labelled with their language when syntect knows
/// it. `##`/`###` headings get stable ids and feed the table of contents.
pub fn render(md: &str) -> Rendered {
  let parser = Parser::new_ext(md, Options::all());
  let mut events = parser.collect::<Vec<_>>();
  let mut out = String::with_capacity(md.len() + md.len() / 2);
  let mut toc = Vec::new();
  let mut used_ids: HashMap<String, usize> = HashMap::new();

  let mut i = 0;
  while i < events.len() {
    if matches!(&events[i], Event::Start(Tag::Heading { .. })) {
      let (level_num, text) = heading_text(&events, i);
      if let Event::Start(Tag::Heading { id, .. }) = &mut events[i]
        && id.is_none()
      {
        let slug = unique_id(slugify(&text), &mut used_ids);
        *id = Some(slug.clone().into());
        if level_num == 2 || level_num == 3 {
          toc.push(TocEntry {
            level: level_num,
            id: slug,
            text,
          });
        }
      }
      i += 1;
      continue;
    }

    if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = &events[i] {
      let lang = info
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
      // Gather the code text up to the matching End event.
      let mut code = String::new();
      let mut j = i + 1;
      while j < events.len() && !matches!(&events[j], Event::End(TagEnd::CodeBlock)) {
        if let Event::Text(t) = &events[j] {
          code.push_str(t);
        }
        j += 1;
      }
      if lang == "demo" {
        // Emit a mount point that the post page turns into a live component.
        let name = code
          .trim()
          .chars()
          .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
          .collect::<String>();
        events[i] = Event::Html(pulldown_cmark::CowStr::Boxed(
          format!("<div class=\"demo-slot\" data-demo=\"{name}\"></div>").into_boxed_str(),
        ));
        events.drain((i + 1)..=j.min(events.len() - 1));
      } else {
        let (highlighted, known) = highlight_code(&lang, &code);
        let label = if known {
          format!("<span class=\"code-lang\">{}</span>", escape_html(&lang))
        } else {
          String::new()
        };
        events[i] = Event::Html(pulldown_cmark::CowStr::Boxed(
          format!("<pre class=\"code-plate\">{label}{highlighted}</pre>").into_boxed_str(),
        ));
        events.drain((i + 1)..=j.min(events.len() - 1));
      }
    }
    i += 1;
  }

  html::push_html(&mut out, events.into_iter());
  Rendered { html: out, toc }
}

/// The plain text and level of the heading starting at `events[i]`.
fn heading_text(events: &[Event<'_>], start: usize) -> (u8, String) {
  let level = match events[start] {
    Event::Start(Tag::Heading { level, .. }) => match level {
      pulldown_cmark::HeadingLevel::H1 => 1,
      pulldown_cmark::HeadingLevel::H2 => 2,
      pulldown_cmark::HeadingLevel::H3 => 3,
      pulldown_cmark::HeadingLevel::H4 => 4,
      pulldown_cmark::HeadingLevel::H5 => 5,
      pulldown_cmark::HeadingLevel::H6 => 6,
    },
    _ => unreachable!("heading_text called on a non-heading event"),
  };
  let mut text = String::new();
  for event in &events[start + 1..] {
    if let Event::Text(t) = event {
      text.push_str(t);
    }
    if matches!(event, Event::End(TagEnd::Heading(_))) {
      break;
    }
  }
  (level, text)
}

/// URL-safe slug from heading text: Thai and alphanumeric characters are
/// kept, everything else becomes a single hyphen.
fn slugify(text: &str) -> String {
  let mut slug = String::new();
  let mut last_was_sep = false;
  for c in text.chars() {
    let c = c.to_lowercase().next().unwrap_or(c);
    let keep = c.is_alphanumeric() || ('\u{e00}'..='\u{e7f}').contains(&c);
    if keep {
      slug.push(c);
      last_was_sep = false;
    } else if !last_was_sep {
      slug.push('-');
      last_was_sep = true;
    }
  }
  let slug = slug.trim_matches('-').to_string();
  if slug.is_empty() {
    "section".to_string()
  } else {
    slug
  }
}

/// Deduplicate ids: the second "foo" becomes "foo-2", and so on.
fn unique_id(base: String, used: &mut HashMap<String, usize>) -> String {
  let n = used.entry(base.clone()).or_insert(0);
  let id = if *n == 0 {
    base
  } else {
    format!("{base}-{}", *n + 1)
  };
  *n += 1;
  id
}

/// Highlight a code block body with syntect, keeping the background out so
/// the design system's `--code-bg` plate shows through. Returns the
/// highlighted inner HTML and whether syntect knew the language.
fn highlight_code(lang: &str, code: &str) -> (String, bool) {
  let known = !lang.is_empty()
    && (syntax_set().find_syntax_by_token(lang).is_some()
      || syntax_set().find_syntax_by_extension(lang).is_some());
  let syntax = if known {
    syntax_set()
      .find_syntax_by_token(lang)
      .or_else(|| syntax_set().find_syntax_by_extension(lang))
      .unwrap_or_else(|| syntax_set().find_syntax_plain_text())
  } else {
    syntax_set().find_syntax_plain_text()
  };
  let html = highlighted_html_for_string(code, syntax_set(), syntax, theme()).unwrap_or_default();
  // syntect wraps the output in a <pre> with an inline background; we already
  // own the <pre> plate, so keep only the highlighted <code> content.
  let trimmed = html
    .trim_start_matches("<pre")
    .trim_start_matches(|c| c != '>')
    .trim_start_matches('>')
    .trim_end();
  let inner = trimmed.strip_suffix("</pre>").unwrap_or(trimmed);
  (scrub_backgrounds(inner), known)
}

/// Remove `background-color:#…;` declarations from syntect's inline styles so
/// the CSS plate (`--code-bg`) stays the single source of the code surface.
fn scrub_backgrounds(html: &str) -> String {
  let mut out = String::with_capacity(html.len());
  let mut rest = html;
  while let Some(rel) = rest.find("background-color:") {
    out.push_str(&rest[..rel]);
    rest = &rest[rel + "background-color:".len()..];
    if let Some(semi) = rest.find(';') {
      rest = &rest[semi + 1..];
    }
  }
  out.push_str(rest);
  out
}

/// Escape the language label for safe embedding in HTML.
fn escape_html(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn highlights_fenced_rust_block() {
    let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
    let html = render(md).html;
    assert!(html.contains("code-plate"), "missing plate class: {html}");
    assert!(html.contains("fn"), "missing content: {html}");
    assert!(!html.contains("highlight.js"), "no hljs references");
  }

  #[test]
  fn known_language_gets_a_label() {
    let md = "```rust\nfn main() {}\n```";
    let html = render(md).html;
    assert!(
      html.contains("<span class=\"code-lang\">rust</span>"),
      "known languages must be labelled: {html}"
    );
  }

  #[test]
  fn unknown_language_gets_no_label() {
    let md = "```not-a-real-language\nwhatever\n```";
    let html = render(md).html;
    assert!(
      !html.contains("code-lang"),
      "unknown languages are unlabelled: {html}"
    );
    assert!(html.contains("code-plate"));
  }

  #[test]
  fn plain_code_block_uses_text_theme() {
    let md = "```\nplain text here\n```";
    let html = render(md).html;
    assert!(html.contains("code-plate"));
    assert!(html.contains("plain text here"));
    assert!(!html.contains("code-lang"));
  }

  #[test]
  fn keeps_prose_unchanged() {
    let md = "# Hello\n\nSome *emphasis* and a [link](https://rust-lang.org).";
    let html = render(md).html;
    assert!(html.contains("<h1 id=\"hello\">Hello</h1>"));
    assert!(html.contains("<em>emphasis</em>"));
    assert!(!html.contains("code-plate"));
  }

  #[test]
  fn demo_block_emits_mount_point() {
    let md = "```demo\ncounter\n```";
    let html = render(md).html;
    assert!(html.contains("demo-slot"), "missing slot: {html}");
    assert!(
      html.contains("data-demo=\"counter\""),
      "missing name: {html}"
    );
    assert!(
      !html.contains("code-plate"),
      "demo should not be highlighted"
    );
  }

  #[test]
  fn headings_get_ids_and_feed_the_toc() {
    let md = "# Title\n\n## First section\n\n### Sub section\n\n## Second section\n";
    let rendered = render(md);
    assert_eq!(
      rendered.toc,
      vec![
        TocEntry {
          level: 2,
          id: "first-section".into(),
          text: "First section".into()
        },
        TocEntry {
          level: 3,
          id: "sub-section".into(),
          text: "Sub section".into()
        },
        TocEntry {
          level: 2,
          id: "second-section".into(),
          text: "Second section".into()
        },
      ]
    );
    assert!(
      rendered
        .html
        .contains("<h2 id=\"first-section\">First section</h2>")
    );
    assert!(
      rendered
        .html
        .contains("<h3 id=\"sub-section\">Sub section</h3>")
    );
  }

  #[test]
  fn duplicate_heading_ids_are_deduplicated() {
    let md = "## Rust\n\n## Rust\n";
    let rendered = render(md);
    assert!(rendered.html.contains("<h2 id=\"rust\">Rust</h2>"));
    assert!(rendered.html.contains("<h2 id=\"rust-2\">Rust</h2>"));
  }

  #[test]
  fn thai_headings_slugify_without_losing_text() {
    let md = "## ยินดีต้อนรับ\n";
    let rendered = render(md);
    assert_eq!(rendered.toc[0].id, "ยินดีต้อนรับ");
    assert_eq!(rendered.toc[0].text, "ยินดีต้อนรับ");
  }
}
