use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Embedded syntax definitions (fancy-regex backend, wasm32-safe).
///
/// TOML is not part of syntect's default set, so the official Sublime Text
/// TOML syntax is embedded (`assets/TOML.sublime-syntax`) and merged in.
fn syntax_set() -> &'static SyntaxSet {
  static SS: OnceLock<SyntaxSet> = OnceLock::new();
  SS.get_or_init(|| {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    let toml = include_str!("../assets/TOML.sublime-syntax");
    if let Ok(def) = syntect::parsing::SyntaxDefinition::load_from_str(toml, true, None) {
      builder.add(def);
    }
    builder.build()
  })
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

/// Render markdown source to an HTML string.
///
/// Enables GitHub-flavoured extensions (tables, strikethrough, task lists,
/// footnotes, and heading attributes) so authors can write rich posts.
/// Fenced code blocks are highlighted with `syntect` at render time - no
/// JavaScript, no CDN - and labelled with their language when syntect knows
/// it.
pub fn render(md: &str) -> String {
  let parser = Parser::new_ext(md, Options::all());
  let mut events = parser.collect::<Vec<_>>();
  let mut out = String::with_capacity(md.len() + md.len() / 2);

  let mut i = 0;
  while i < events.len() {
    if let Event::Start(Tag::Table(_)) = &events[i] {
      // Wrap tables so wide ones scroll inside a container instead of
      // overflowing the page on narrow screens.
      events[i] = Event::Html(pulldown_cmark::CowStr::from(
        "<div class=\"table-wrap\"><table>",
      ));
    } else if let Event::End(TagEnd::Table) = &events[i] {
      events[i] = Event::Html(pulldown_cmark::CowStr::from(
        "</tbody></table></div>\n",
      ));
    } else if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = &events[i] {
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
        // Info strings often carry attributes after a comma (rust,ignore);
        // use the first token syntect actually knows so highlighting works.
        let known = resolve_lang(&lang);
        let highlighted = highlight_code(known, &code);
        events[i] = Event::Html(pulldown_cmark::CowStr::Boxed(
          format!("<pre class=\"code-plate\">{highlighted}</pre>").into_boxed_str(),
        ));
        events.drain((i + 1)..=j.min(events.len() - 1));
      }
    }
    i += 1;
  }

  html::push_html(&mut out, events.into_iter());
  out
}

/// The first comma/space-separated token of a fence info string that syntect
/// knows (by token or extension), or "" for plain text.
fn resolve_lang(info: &str) -> &str {
  info
    .split(|c: char| c == ',' || c.is_ascii_whitespace())
    .find(|t| {
      !t.is_empty()
        && (syntax_set().find_syntax_by_token(t).is_some()
          || syntax_set().find_syntax_by_extension(t).is_some())
    })
    .unwrap_or("")
}

/// Highlight a code block body with syntect, keeping the background out so
/// the design system's `--code-bg` plate shows through.
fn highlight_code(lang: &str, code: &str) -> String {
  let syntax = syntax_set()
    .find_syntax_by_token(lang)
    .or_else(|| syntax_set().find_syntax_by_extension(lang))
    .unwrap_or_else(|| syntax_set().find_syntax_plain_text());
  let html = highlighted_html_for_string(code, syntax_set(), syntax, theme()).unwrap_or_default();
  // syntect wraps the output in a <pre> with an inline background; we already
  // own the <pre> plate, so keep only the highlighted <code> content.
  let trimmed = html
    .trim_start_matches("<pre")
    .trim_start_matches(|c| c != '>')
    .trim_start_matches('>')
    .trim_end();
  let inner = trimmed.strip_suffix("</pre>").unwrap_or(trimmed);
  scrub_backgrounds(inner)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn highlights_fenced_rust_block() {
    let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
    let html = render(md);
    assert!(html.contains("code-plate"), "missing plate class: {html}");
    assert!(html.contains("fn"), "missing content: {html}");
    assert!(!html.contains("highlight.js"), "no hljs references");
  }

  #[test]
  fn plain_code_block_uses_text_theme() {
    let md = "```\nplain text here\n```";
    let html = render(md);
    assert!(html.contains("code-plate"));
    assert!(html.contains("plain text here"));
  }

  #[test]
  fn fence_attributes_after_comma_still_highlight() {
    let md = "```rust,ignore\nfn main() {}\n```";
    let html = render(md);
    assert!(
      html.contains("<span style=\"color:#"),
      "rust,ignore must still be highlighted: {html}"
    );
  }

  #[test]
  fn toml_block_is_highlighted() {
    let md = "```toml\n[profile.release]\nopt-level = \"z\"\n```";
    let html = render(md);
    assert!(
      html.contains("<span style=\"color:#"),
      "toml must be highlighted: {html}"
    );
    assert!(html.contains("opt-level"), "missing content: {html}");
  }

  #[test]
  fn table_is_wrapped_for_mobile_scroll() {
    let md = "| a | b |\n|---|---|\n| 1 | 2 |";
    let html = render(md);
    assert!(
      html.contains("<div class=\"table-wrap\"><table>"),
      "missing table wrap open: {html}"
    );
    assert!(
      html.contains("</table></div>"),
      "missing table wrap close: {html}"
    );
  }

  #[test]
  fn keeps_prose_unchanged() {
    let md = "# Hello\n\nSome *emphasis* and a [link](https://rust-lang.org).";
    let html = render(md);
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<em>emphasis</em>"));
    assert!(!html.contains("code-plate"));
  }

  #[test]
  fn demo_block_emits_mount_point() {
    let md = "```demo\ncounter\n```";
    let html = render(md);
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
}
