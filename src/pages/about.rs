use leptos::prelude::*;
use leptos_meta::Title;

use crate::icons::RustLogo;
use rust_blog::content::site;

/// About page.
#[component]
pub fn About() -> impl IntoView {
  view! {
      <div class="container about-page">
          <Title text=format!("About · {}", site::TITLE)/>
          <article class="article">
              <header class="article-header">
                  <p class="back-link">"About this blog"</p>
                  <h1 class="article-title">"Written in Rust, about Rust"</h1>
              </header>
              <div class="prose">
                  <p>
                      "Welcome to "
                      <strong>{site::TITLE}</strong>
                      " - a collection of notes and writing about "
                      <strong>"Rust"</strong>
                      ", "
                      <strong>"WebAssembly"</strong>
                      " and modern web development."
                  </p>
                  <p>
                      "The notable part: this blog is not written in JavaScript. "
                      "It is built with "
                      <strong>"Rust"</strong>
                      " and the "
                      <strong>"Leptos"</strong>
                      " framework (client-side rendering), compiled to WebAssembly, "
                      "and runs directly in your browser "
                      <RustLogo/>
                      "."
                  </p>
                  <h2>"The stack"</h2>
                  <ul>
                      <li>"Language: Rust (edition 2024)"</li>
                      <li>"Framework: Leptos 0.8 (CSR)"</li>
                      <li>"Bundler: Trunk"</li>
                      <li>"Hosting: GitHub Pages"</li>
                  </ul>
                  <h2>"How articles are published"</h2>
                  <p>
                      "Every post is a plain Markdown file in "
                      <code>"content/posts/"</code>
                      ". Create a file with frontmatter (title, date, tags), write "
                      "the content, and the blog shows it automatically - no "
                      "database, no build step per post."
                  </p>
                  <p>
                      "The full source lives at "
                      <a href=site::GITHUB_URL target="_blank" rel="noreferrer">
                          {site::GITHUB_URL}
                      </a>
                      "."
                  </p>
              </div>
          </article>
      </div>
  }
}
