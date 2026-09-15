//! Golden content tests: the exact rendered HTML for a representative post
//! is pinned here, so any markdown or rendering change that alters the
//! output fails the test instead of silently shipping different pages.

use rust_blog::frontmatter;
use rust_blog::markdown;

const GOLDEN_HTML: &str = r##"<h1>Golden fixture</h1>
<p>A paragraph with <em>emphasis</em>, <strong>strong</strong>, and a <a href="https://rust-lang.org">link</a>.</p>
<h2>Heading two</h2>
<figure class="code-frame"><figcaption class="code-frame-bar"><span class="code-frame-dots" aria-hidden="true"><i></i><i></i><i></i></span><span class="code-frame-lang">rust</span><button type="button" class="code-copy" data-copy-code aria-label="Copy code"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="8" y="8" width="11" height="11" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/></svg><span data-copy-label aria-live="polite">Copy</span></button></figcaption><pre class="code-plate"><code><span style="color:#cba6f7;">fn </span><span style="font-style:italic;color:#89b4fa;">main</span><span style="color:#9399b2;">() {
</span><span style="color:#cdd6f4;">    </span><span style="font-style:italic;color:#89b4fa;">println!</span><span style="color:#9399b2;">(</span><span style="color:#a6e3a1;">&quot;hi&quot;</span><span style="color:#9399b2;">);
</span><span style="color:#9399b2;">}
</span></code></pre></figure><div class="table-wrap"><table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody>
<tr><td>Rust</td><td>wasm</td></tr>
<tr><td>Thai</td><td>ไทย</td></tr>
</tbody></table></div>
<blockquote>
<p>A blockquote.</p>
</blockquote>
<p>A footnote reference<sup class="footnote-reference"><a href="#1">1</a></sup>.</p>
<ul>
<li><input disabled="" type="checkbox" checked=""/>
done</li>
<li><input disabled="" type="checkbox"/>
todo</li>
</ul>
<div class="footnote-definition" id="1"><sup class="footnote-definition-label">1</sup>
<p>The footnote text.</p>
</div>
"##;

#[test]
fn golden_post_renders_exact_html() {
  let raw = include_str!("fixtures/golden.md");
  let parsed = frontmatter::parse(raw).expect("fixture must be a valid post");
  assert!(parsed.warnings.is_empty(), "fixture must be schema-clean");
  assert_eq!(parsed.meta.title, "Golden fixture");
  assert_eq!(parsed.meta.date, "2026-08-29");

  let html = markdown::render(&parsed.body);
  assert_eq!(html, GOLDEN_HTML);
}

#[test]
fn golden_detects_markdown_changes() {
  let raw = include_str!("fixtures/golden.md");
  let parsed = frontmatter::parse(raw).unwrap();
  let changed = format!(
    "{}\n\nA new paragraph that must break the golden.",
    parsed.body
  );
  assert_ne!(markdown::render(&changed), GOLDEN_HTML);
}

#[test]
fn golden_code_plate_has_no_stray_closing_tag() {
  let raw = include_str!("fixtures/golden.md");
  let parsed = frontmatter::parse(raw).unwrap();
  let html = markdown::render(&parsed.body);
  assert_eq!(html.matches("</pre>").count(), 1);
}
