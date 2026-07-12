use pulldown_cmark::{html, Options, Parser};

/// Render markdown source to an HTML string.
///
/// Enables GitHub-flavoured extensions (tables, strikethrough, task lists,
/// footnotes, and heading attributes) so authors can write rich posts.
pub fn render(md: &str) -> String {
    let parser = Parser::new_ext(md, Options::all());
    let mut out = String::with_capacity(md.len() + md.len() / 2);
    html::push_html(&mut out, parser);
    out
}
