use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use rust_blog::content::site;

/// 404 / post-not-found fallback.
#[component]
pub fn NotFound() -> impl IntoView {
  view! {
      <div class="container notfound">
          <Title text=format!("Not found · {}", site::TITLE)/>
          <div class="notfound-inner">
              <p class="notfound-code">"404"</p>
              <h1 class="notfound-title">"This page does not exist"</h1>
              <p class="notfound-sub">
                  "The article may have been moved, or the address is wrong."
              </p>
              <A href="/">
                  <span class="btn-primary">"← Back to articles"</span>
              </A>
          </div>
      </div>
  }
}
