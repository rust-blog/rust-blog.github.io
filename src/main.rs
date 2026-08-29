mod components;
mod content;
mod frontmatter;
mod icons;
mod markdown;
mod pages;
mod util;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use std::borrow::Cow;

use components::{Footer, Nav, ThemeProvider};
use content::load_posts;
use pages::{about::About, home::Home, not_found::NotFound, post::Post};
use util::detect_base;

pub fn main() {
  console_error_panic_hook::set_once();
  mount_to_body(App);
}

/// Root component: meta context, theme provider, router and layout.
#[component]
fn App() -> impl IntoView {
  provide_meta_context();

  let posts = load_posts();
  provide_context(posts);

  let base = detect_base();

  view! {
      <ThemeProvider>
          <Router base=Cow::Owned(base)>
              <Nav/>
              <main class="main">
                  <Routes fallback=|| view! { <NotFound/> }>
                      <Route path=path!("/") view=Home/>
                      <Route path=path!("/about") view=About/>
                      <Route path=path!("/post/:slug") view=Post/>
                  </Routes>
              </main>
              <Footer/>
          </Router>
      </ThemeProvider>
  }
}
