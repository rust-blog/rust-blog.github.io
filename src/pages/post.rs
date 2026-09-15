use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

use crate::components::PostCard;
use crate::pages::not_found::NotFound;
use crate::util::format_date;
use rust_blog::content::Post;
use rust_blog::site;

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
                      let has_related = !related.is_empty();

                      view! {
                          <Title text=title.clone()/>
                          <Meta name="description" content=desc.clone()/>
                          <Meta property="og:title" content=title.clone()/>
                          <Meta property="og:description" content=desc.clone()/>
                          <Meta property="og:type" content="article"/>

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

                              <div
                                  class="prose"
                                  inner_html=Signal::derive(move || html.get())
                                  on:click=on_prose_click
                              ></div>
                          </article>

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

/// Delegate clicks inside the rendered post body to the framed code blocks'
/// copy buttons. The body is injected as raw HTML, so listening on the stable
/// `.prose` container is safer than wiring each button after injection.
fn on_prose_click(ev: web_sys::MouseEvent) {
  let Some(target) = ev
    .target()
    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
  else {
    return;
  };
  let Ok(Some(button)) = target.closest("[data-copy-code]") else {
    return;
  };
  copy_code(&button);
}

/// Copy the framed block's code to the clipboard and flash the button label.
fn copy_code(button: &web_sys::Element) {
  let Some(frame) = button.closest(".code-frame").ok().flatten() else {
    return;
  };
  let Some(code) = frame.query_selector("code").ok().flatten() else {
    return;
  };
  let label = button.query_selector("[data-copy-label]").ok().flatten();
  let text = code.text_content().unwrap_or_default();

  let Some(window) = web_sys::window() else {
    return;
  };
  // `navigator.clipboard` is absent outside secure contexts; read it
  // reflectively so a missing API degrades to a message instead of a panic.
  let clipboard =
    js_sys::Reflect::get(window.navigator().as_ref(), &JsValue::from_str("clipboard"))
      .ok()
      .filter(|value| !value.is_undefined() && !value.is_null());
  let Some(clipboard) = clipboard.map(|value| value.unchecked_into::<web_sys::Clipboard>()) else {
    flash_label(&window, &label, "Copy failed");
    return;
  };
  let promise = clipboard.write_text(&text);

  let window_for_task = window.clone();
  spawn_local(async move {
    let message = match JsFuture::from(promise).await {
      Ok(_) => "Copied",
      Err(_) => "Copy failed",
    };
    flash_label(&window_for_task, &label, message);
  });
}

/// Show a transient message on a copy button, restoring "Copy" after a beat.
fn flash_label(window: &web_sys::Window, label: &Option<web_sys::Element>, message: &str) {
  let Some(label) = label else {
    return;
  };
  label.set_text_content(Some(message));
  let label = label.clone();
  let restore = Closure::once_into_js(move || label.set_text_content(Some("Copy")));
  let _ =
    window.set_timeout_with_callback_and_timeout_and_arguments_0(restore.unchecked_ref(), 1600);
}
