use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::icons::{MoonIcon, SunIcon};
use crate::util::{Theme, apply_theme, load_theme, save_theme};
use rust_blog::content::Post;
use rust_blog::site;

/// Shared theme state, provided near the root of the app.
#[derive(Clone, Copy)]
pub struct ThemeContext {
  pub theme: ReadSignal<Theme>,
  pub set_theme: WriteSignal<Theme>,
}

/// Wraps the app, initialises the theme and keeps `<html data-theme>` in sync.
#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
  let initial = load_theme();
  let (theme, set_theme) = signal(initial);
  apply_theme(initial);
  provide_context(ThemeContext { theme, set_theme });

  Effect::new(move |_| {
    let t = theme.get();
    apply_theme(t);
    save_theme(t);
  });

  view! { {children()} }
}

/// Access the current theme context.
pub fn use_theme() -> ThemeContext {
  expect_context::<ThemeContext>()
}

/// Light/dark mode toggle button.
#[component]
pub fn ThemeToggle() -> impl IntoView {
  let ThemeContext { theme, set_theme } = use_theme();
  view! {
      <button
          class="theme-toggle"
          type="button"
          aria-label="Toggle light / dark mode"
          title="Toggle light / dark mode"
          on:click=move |_| set_theme.set(theme.get().toggle())
      >
          <span class="theme-toggle-icon">
              {move || match theme.get() {
                  Theme::Light => Either::Left(view! { <MoonIcon/> }),
                  Theme::Dark => Either::Right(view! { <SunIcon/> }),
              }}
          </span>
      </button>
  }
}

/// Site header with branding and primary navigation.
#[component]
pub fn Nav() -> impl IntoView {
  view! {
      <header class="site-header">
          <a class="skip-link" href="#main">"Skip to content"</a>
          <div class="container nav-inner">
              <A href="/">
                  <span class="brand">
                      <span class="brand-mark">"r"</span>
                      <span class="brand-name">{site::TITLE}</span>
                  </span>
              </A>
              <nav class="nav-links" aria-label="primary">
                  <A href="/">
                      <span class="nav-link">"Posts"</span>
                  </A>
                  <A href="/about">
                      <span class="nav-link">"About"</span>
                  </A>
                  <a href="/rss.xml" target="_blank" rel="noreferrer">
                      <span class="nav-link">"RSS"</span>
                  </a>
                  <ThemeToggle/>
              </nav>
          </div>
      </header>
  }
}

/// Site footer.
#[component]
pub fn Footer() -> impl IntoView {
  let year = js_sys::Date::new_0().get_full_year();
  view! {
        <footer class="site-footer">
            <div class="container footer-inner">
  <p class="footer-copy">
                      {site::AUTHOR}
                      " · © "
                      {year}
                  </p>
                <div class="footer-links">
                    <a href=site::GITHUB_URL target="_blank" rel="noreferrer">
                        "GitHub"
                    </a>
                    <A href="/about">
                        <span class="footer-link">"About"</span>
                    </A>
                    <a href="/rss.xml" target="_blank" rel="noreferrer">
                        "RSS"
                    </a>
                </div>
            </div>
        </footer>
    }
}

/// A clickable tag used for filtering on the home page.
#[component]
pub fn TagChip(
  tag: String,
  active: bool,
  #[prop(optional)] on_click: Option<Callback<String>>,
) -> impl IntoView {
  let tag_for_click = tag.clone();
  view! {
      <button
          class=move || if active { "tag-chip active" } else { "tag-chip" }
          type="button"
          on:click=move |_| {
              if let Some(cb) = &on_click {
                  cb.run(tag_for_click.clone());
              }
          }
      >
          {tag}
      </button>
  }
}

/// A single post preview row in the editorial index.
#[component]
pub fn PostCard(post: Post) -> impl IntoView {
  let date = crate::util::format_date(&post.meta.date);
  view! {
    <A href=format!("/post/{}", post.slug)>
        <article class="post-row">
            <div class="post-row-meta">
                <span>{date}</span>
                <span class="dot">"·"</span>
                <span>{format!("{} min read", post.reading_time)}</span>
            </div>
            <h2 class="post-row-title">{post.meta.title.clone()}</h2>
            <p class="post-row-desc">{post.meta.description.clone()}</p>
              <div class="post-row-tags">
                  {post
                      .meta
                      .tags
                      .iter()
                      .cloned()
                      .map(|t| view! { <span class="tag-chip static">{t}</span> })
                      .collect::<Vec<_>>()}
              </div>
          </article>
      </A>
  }
}
