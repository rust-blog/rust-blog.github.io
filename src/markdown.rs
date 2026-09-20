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

/// Embedded syntax theme: Catppuccin Mocha, the palette the framed code
/// blocks are designed around (muted pastels over a `#1e1e2e` plate), so the
/// site keeps the dark code surface in both UI themes. Falls back to syntect's
/// default dark theme if the asset ever fails to parse - a broken theme must
/// never take the site down.
fn theme() -> &'static syntect::highlighting::Theme {
  static TH: OnceLock<syntect::highlighting::Theme> = OnceLock::new();
  TH.get_or_init(|| {
    let mut file = std::io::Cursor::new(include_bytes!("../assets/Catppuccin-Mocha.tmTheme"));
    ThemeSet::load_from_reader(&mut file)
      .unwrap_or_else(|_| ThemeSet::load_defaults().themes["base16-eighties.dark"].clone())
  })
}

/// Render markdown source to an HTML string.
///
/// Enables GitHub-flavoured extensions (tables, strikethrough, task lists,
/// footnotes, and heading attributes) so authors can write rich posts.
/// Fenced code blocks are highlighted with `syntect` at render time - no
/// JavaScript, no CDN - and wrapped in the framed code block: traffic-light
/// dots, an optional filename (` ```rust title="src/main.rs" `), the language
/// badge, and a copy button. Image references into `content/assets/` are
/// rewritten to their fingerprinted public URLs (see `src/assets.rs`).
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
      events[i] = Event::Html(pulldown_cmark::CowStr::from("</tbody></table></div>\n"));
    } else if let Event::Start(Tag::Image { dest_url, .. }) = &mut events[i] {
      // Content images live in `content/assets/` and ship with a content
      // hash in the file name; rewrite the markdown reference to the URL the
      // build staged. Anything else (external URLs, unknown paths) stays as
      // written.
      if let Some(url) = crate::assets::resolve(dest_url) {
        *dest_url = pulldown_cmark::CowStr::from(url);
      }
    } else if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = &events[i] {
      let info = info.to_string();
      // The fence language is the first comma/space-separated token, so
      // `rust,ignore` and `rust title="main.rs"` both resolve to `rust`.
      let lang = info
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .find(|t| !t.is_empty())
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
        let highlighted = highlight_code(resolve_lang(&lang), &code);
        let frame = code_frame(&lang, fence_title(&info), &highlighted);
        events[i] = Event::Html(pulldown_cmark::CowStr::Boxed(frame.into_boxed_str()));
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

/// Languages that read as shell sessions: without an explicit title their
/// frame is labelled "Terminal", mirroring the reference blog.
const TERMINAL_LANGUAGES: [&str; 9] = [
  "ansi",
  "bash",
  "console",
  "fish",
  "powershell",
  "sh",
  "shell",
  "shellsession",
  "zsh",
];

/// The copy button that sits at the right of every code frame. The label and
/// the clipboard wiring are handled by the post page (see `pages/post.rs`).
const COPY_BUTTON: &str = concat!(
  r#"<button type="button" class="code-copy" data-copy-code aria-label="Copy code">"#,
  r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">"#,
  r#"<rect x="8" y="8" width="11" height="11" rx="2"/>"#,
  r#"<path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>"#,
  r#"</svg>"#,
  r#"<span data-copy-label aria-live="polite">Copy</span>"#,
  r#"</button>"#,
);

/// Wrap highlighted code in the framed block: a header bar with traffic-light
/// dots, an optional title (filename or "Terminal"), the language badge, and
/// the copy button - then the code plate itself.
fn code_frame(lang: &str, title: Option<&str>, highlighted: &str) -> String {
  let badge = if lang.is_empty() { "text" } else { lang };
  let label = title.map(str::to_string).or_else(|| {
    TERMINAL_LANGUAGES
      .contains(&lang)
      .then(|| "Terminal".to_string())
  });

  let mut out = String::with_capacity(highlighted.len() + 512);
  out.push_str("<figure class=\"code-frame\"><figcaption class=\"code-frame-bar\">");
  out.push_str("<span class=\"code-frame-dots\" aria-hidden=\"true\"><i></i><i></i><i></i></span>");
  if let Some(label) = label {
    out.push_str("<span class=\"code-frame-title\">");
    out.push_str(&escape_html(&label));
    out.push_str("</span>");
  }
  out.push_str("<span class=\"code-frame-lang\">");
  out.push_str(&escape_html(badge));
  out.push_str("</span>");
  out.push_str(COPY_BUTTON);
  out.push_str("</figcaption><pre class=\"code-plate\"><code>");
  out.push_str(highlighted);
  out.push_str("</code></pre></figure>");
  out
}

/// Extract an optional `title="..."` from a fence info string (single quotes
/// and bare tokens also work) - the authoring convention for naming the file
/// shown in a code frame.
fn fence_title(info: &str) -> Option<&str> {
  let rest = info.split_once("title=")?.1.trim_start();
  let title = match rest
    .chars()
    .next()
    .filter(|c| matches!(c, '"' | '\'' | '`'))
  {
    Some(quote) => rest[quote.len_utf8()..]
      .split(quote)
      .next()
      .unwrap_or_default(),
    None => {
      let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
      &rest[..end]
    }
  };
  (!title.is_empty()).then_some(title)
}

/// Minimal HTML escaping for fence metadata (titles and language tokens).
fn escape_html(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for c in s.chars() {
    match c {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '"' => out.push_str("&quot;"),
      '\'' => out.push_str("&#39;"),
      _ => out.push(c),
    }
  }
  out
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
  // syntect opens with `<pre …>\n`; that cosmetic newline is ignored only when
  // it follows a `<pre>` start tag, so drop it - otherwise it becomes a real
  // blank first line inside our `<code>` and pushes the code down.
  let visible = trimmed.strip_suffix("</pre>").unwrap_or(trimmed);
  scrub_backgrounds(visible.strip_prefix('\n').unwrap_or(visible))
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
  fn code_frame_carries_dots_title_language_and_copy() {
    let md = "```rust title=\"src/main.rs\"\nfn main() {}\n```";
    let html = render(md);
    assert!(html.contains("<figure class=\"code-frame\">"), "{html}");
    assert!(html.contains("code-frame-dots"), "{html}");
    assert!(
      html.contains("<span class=\"code-frame-title\">src/main.rs</span>"),
      "{html}"
    );
    assert!(
      html.contains("<span class=\"code-frame-lang\">rust</span>"),
      "{html}"
    );
    assert!(html.contains("data-copy-code"), "{html}");
    assert!(html.contains("<code>"), "{html}");
    assert!(!html.contains("<code>\n"), "no blank first line: {html}");
  }

  #[test]
  fn shell_frame_is_labelled_terminal_when_untitled() {
    let html = render("```bash\ncargo test\n```");
    assert!(html.contains(">Terminal</span>"), "{html}");
  }

  #[test]
  fn plain_fence_is_badged_text() {
    let html = render("```\nplain\n```");
    assert!(html.contains(">text</span>"), "{html}");
  }

  #[test]
  fn fence_title_is_html_escaped() {
    let html = render("```rust title=\"a<b>&c\"\nfn main() {}\n```");
    assert!(html.contains("a&lt;b&gt;&amp;c"), "{html}");
  }

  #[test]
  fn embedded_theme_is_catppuccin_mocha() {
    // Mauve (`#cba6f7`) is a signature Mocha colour; the previous base16
    // theme coloured keywords `#cc99cc`.
    let html = render("```rust\nfn main() {}\n```");
    assert!(
      html.contains("#cba6f7"),
      "catppuccin mocha not loaded: {html}"
    );
    assert!(
      !html.contains("#cc99cc"),
      "old base16 theme still active: {html}"
    );
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
  fn content_asset_images_get_fingerprinted_urls() {
    let url = crate::assets::resolve("assets/rust-blog-gear.svg").expect("sample asset");
    let html = render("![gear](assets/rust-blog-gear.svg)");
    assert!(
      html.contains(&format!("<img src=\"{url}\" alt=\"gear\" />")),
      "{html}"
    );
  }

  #[test]
  fn external_images_are_left_alone() {
    let html = render("![remote](https://example.com/photo.png)");
    assert!(
      html.contains("<img src=\"https://example.com/photo.png\" alt=\"remote\" />"),
      "{html}"
    );
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
