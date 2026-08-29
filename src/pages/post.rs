use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::components::PostCard;
use crate::pages::not_found::NotFound;
use crate::util::format_date;
use rust_blog::content::{Post, site};

/// Individual post page, rendered from embedded markdown.
#[component]
pub fn Post() -> impl IntoView {
  let params = use_params_map();
  let posts = expect_context::<Vec<Post>>();

  let slug = { move || params.get().get("slug").unwrap_or_default() };

  // (current post, related posts) - recomputed when the slug changes.
  let data = Memo::new({
    let posts = posts.clone();
    move |_| {
      let s = slug();
      posts.iter().position(|p| p.slug == s).map(|idx| {
        let current = posts[idx].clone();
        let related = posts
          .iter()
          .skip(idx + 1)
          .take(2)
          .cloned()
          .collect::<Vec<_>>();
        (current, related)
      })
    }
  });

  // Rendered post HTML, reactive so live demos can be mounted when it changes.
  let html = Memo::new(move |_| data.get().map(|(p, _)| p.html).unwrap_or_default());

  // After the post HTML is injected, mount any live demos it references.
  // We defer a frame and query the document so we never race the `inner_html`
  // mutation or the node-ref attachment.
  Effect::new(move |_| {
    let _ = html.get();
    let window = web_sys::window().expect("no window");
    let cb = wasm_bindgen::prelude::Closure::once(Box::new(move |_t: f64| {
      mount_demos();
    }) as Box<dyn FnMut(f64)>);
    let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
    cb.forget();
  });

  let data_for_view = data;

  view! {
      <div class="container post-page">
          <Show
              when=move || data.get().is_some()
              fallback=|| view! { <NotFound/> }
          >
              {move || {
                  data_for_view.get().map(|(p, related)| {
                      let title = p.meta.title.clone();
                      let desc = if p.meta.description.is_empty() {
                          site::DESCRIPTION.to_string()
                      } else {
                          p.meta.description.clone()
                      };
                      let author = p
                          .meta
                          .author
                          .clone()
                          .unwrap_or_else(|| site::AUTHOR.to_string());
                      let tags = p.meta.tags.clone();
                      let date = format_date(&p.meta.date);
                      let reading = p.reading_time;
                      let toc = p.toc.clone();
                      let build_toc = |entries: Vec<rust_blog::markdown::TocEntry>| {
                          entries
                              .into_iter()
                              .map(|entry| {
                                  let class = if entry.level == 2 {
                                      "toc-h2"
                                  } else {
                                      "toc-h3"
                                  };
                                  view! {
                                      <li class=class>
                                          <a href=format!("#{}", entry.id)>
                                              {entry.text}
                                          </a>
                                      </li>
                                  }
                              })
                              .collect::<Vec<_>>()
                      };
                      let has_toc = !toc.is_empty();
                      let toc_mobile = build_toc(toc.clone());
                      let toc_desktop = build_toc(toc);
                      let has_related = !related.is_empty();

                      view! {
                          <Title text=title.clone()/>
                          <Meta name="description" content=desc.clone()/>
                          <Meta property="og:title" content=title.clone()/>
                          <Meta property="og:description" content=desc.clone()/>
                          <Meta property="og:type" content="article"/>

                          <div class="post-layout">
                              <article class="article">
                                  <header class="article-header">
                                      <leptos_router::components::A href="/">
                                          <span class="back-link">"← Back to articles"</span>
                                      </leptos_router::components::A>
                                      <div class="article-tags">
                                          {tags
                                              .iter()
                                              .cloned()
                                              .map(|t| view! { <span class="tag-chip static">{t}</span> })
                                              .collect::<Vec<_>>()}
                                      </div>
                                      <h1 class="article-title">{title}</h1>
                                      <div class="article-meta">
                                          <span>{date}</span>
                                          <span class="dot">"·"</span>
                                          <span>{format!("{reading} min read")}</span>
                                          <span class="dot">"·"</span>
                                          <span>{author}</span>
                                      </div>
                                  </header>

                                  <Show when=move || has_toc>
                                      <details class="toc-mobile">
                                          <summary>"Contents"</summary>
                                          {toc_mobile.clone()}
                                      </details>
                                  </Show>

                                  <div
                                      class="prose"
                                      inner_html=Signal::derive(move || html.get())
                                  ></div>
                              </article>

                              <Show when=move || has_toc>
                                  <aside class="toc" aria-label="Table of contents">
                                      {toc_desktop.clone()}
                                  </aside>
                              </Show>
                          </div>

                          <Show when=move || has_related>
                              <section class="related">
                                  <h2 class="related-title">"Related articles"</h2>
                                  <div class="post-index">
                                      {related
                                          .iter()
                                          .cloned()
                                          .map(|rp| view! { <PostCard post=rp/> })
                                          .collect::<Vec<_>>()}
                                  </div>
                              </section>
                          </Show>
                      }
                  })
              }}
          </Show>
      </div>
  }
}

/// Mount live demo components into any `.demo-slot` elements left by the
/// markdown `demo` directive.
fn mount_demos() {
  let Some(document) = web_sys::window().and_then(|w| w.document()) else {
    return;
  };
  let slots = document.get_elements_by_class_name("demo-slot");
  web_sys::console::log_1(&format!("demo: found {} slot(s)", slots.length()).into());
  for i in 0..slots.length() {
    if let Some(slot) = slots
      .item(i)
      .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
    {
      // Skip slots we have already mounted into.
      if slot.first_child().is_some() {
        continue;
      }
      if let Some(name) = slot.get_attribute("data-demo")
        && name.as_str() == "counter"
      {
        build_counter(&slot);
      }
    }
  }
}

/// Build the interactive counter demo in pure Rust/WASM, backed by a real
/// Leptos `signal` so its state mirrors what the post describes.
fn build_counter(slot: &web_sys::HtmlElement) {
  let owner = Owner::new();
  owner.with(|| {
    let (count, set_count) = signal(0u32);
    let document = web_sys::window().unwrap().document().unwrap();

    let btn = document.create_element("button").unwrap();
    btn.set_attribute("class", "demo-counter").unwrap();
    btn.set_attribute("type", "button").unwrap();

    let num = document.create_element("span").unwrap();
    num.set_text_content(Some("0"));

    let label = document.create_element("span").unwrap();
    label.set_attribute("class", "demo-counter-label").unwrap();
    label.set_text_content(Some("clicks"));

    btn.append_child(&num).unwrap();
    btn.append_child(&label).unwrap();

    // Keep the displayed number in sync with the signal.
    let num_view = num.clone();
    Effect::new(move |_| {
      num_view.set_text_content(Some(&count.get().to_string()));
    });

    let on_click = Closure::wrap(Box::new(move |_e: web_sys::Event| {
      set_count.update(|n| *n += 1);
    }) as Box<dyn FnMut(web_sys::Event)>);
    btn
      .add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())
      .unwrap();
    on_click.forget();

    slot.append_child(&btn).unwrap();
  });

  // Keep the demo's reactive owner alive for the page lifetime so the
  // display-sync effect keeps tracking the signal.
  std::mem::forget(owner);
}
