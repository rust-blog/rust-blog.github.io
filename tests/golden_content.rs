//! Golden content tests: the exact rendered HTML for a representative post
//! is pinned here, so any markdown or rendering change that alters the
//! output fails the test instead of silently shipping different pages.

use rust_blog::frontmatter;
use rust_blog::markdown;

const GOLDEN_HTML: &str = r##"<h1>Golden fixture</h1>
<p>A paragraph with <em>emphasis</em>, <strong>strong</strong>, and a <a href="https://rust-lang.org">link</a>.</p>
<h2>Heading two</h2>
<pre class="code-plate"><span class="code-lang">rust</span>
<span style="color:#cc99cc;">fn </span><span style="color:#6699cc;">main</span><span style="color:#d3d0c8;">() {
</span><span style="color:#d3d0c8;">    println!(&quot;</span><span style="color:#99cc99;">hi</span><span style="color:#d3d0c8;">&quot;);
</span><span style="color:#d3d0c8;">}
</span></pre><table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody>
<tr><td>Rust</td><td>wasm</td></tr>
<tr><td>Thai</td><td>ไทย</td></tr>
</tbody></table>
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
