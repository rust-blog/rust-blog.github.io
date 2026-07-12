use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::{PostCard, TagChip};
use crate::content::{Post, all_tags, site};

/// Home page: hero, search + tag filtering, and the post grid.
#[component]
pub fn Home() -> impl IntoView {
    let posts = expect_context::<Vec<Post>>();
    let tags = all_tags(&posts);
    let total = posts.len();

    let (query, set_query) = signal(String::new());
    let (active_tag, set_active_tag) = signal(None::<String>);

    let filtered = Memo::new({
        move |_| {
            let q = query.get().to_lowercase();
            let tag = active_tag.get();
            posts
                .iter()
                .filter(|p| {
                    let matches_tag = tag
                        .as_ref()
                        .map(|t| p.meta.tags.iter().any(|x| x == t))
                        .unwrap_or(true);
                    let matches_q = q.is_empty()
                        || p.meta.title.to_lowercase().contains(&q)
                        || p.meta.description.to_lowercase().contains(&q)
                        || p.meta.tags.iter().any(|t| t.to_lowercase().contains(&q));
                    matches_tag && matches_q
                })
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    let filtered_grid = filtered;
    let filtered_count = filtered;
    let filtered_show = filtered;

    view! {
        <div class="container home">
            <Title text=format!("{} · บล็อก Rust ที่สร้างด้วย Leptos", site::TITLE)/>
            <section class="hero">
                <p class="hero-eyebrow">"🦀 สร้างด้วย Rust + WebAssembly"</p>
                <h1 class="hero-title">{site::TAGLINE}</h1>
                <p class="hero-sub">{site::DESCRIPTION}</p>
            </section>

            <div class="post-toolbar">
                <div class="search-box">
                    <span class="search-icon" aria-hidden="true">"🔍"</span>
                    <input
                        type="search"
                        class="search-input"
                        placeholder="ค้นหาบทความ…"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                    />
                </div>

                <div class="tag-filters">
                    {move || {
                        let active_tag = active_tag;
                        let set_active_tag = set_active_tag;
                        let mut chips = vec![
                            view! {
                                <TagChip
                                    tag="ทั้งหมด".to_string()
                                    active=active_tag.get().is_none()
                                    on_click=Callback::new({
                                        let set = set_active_tag;
                                        move |_| set.set(None)
                                    })
                                />
                            },
                        ];
                        for t in &tags {
                            let t2 = t.clone();
                            let set = set_active_tag;
                            chips.push(
                                view! {
                                    <TagChip
                                        tag=t2.clone()
                                        active=active_tag.get().as_ref() == Some(&t2)
                                        on_click=Callback::new(move |_| set.set(Some(t2.clone())))
                                    />
                                },
                            );
                        }
                        chips
                    }}
                </div>
            </div>

            <p class="result-count">
                {move || {
                    let n = filtered_count.get().len();
                    format!("{n} / {total} บทความ")
                }}
            </p>

            <div class="post-grid">
                {move || {
                    filtered_grid
                        .get()
                        .into_iter()
                        .map(|p| view! { <PostCard post=p/> })
                        .collect::<Vec<_>>()
                }}
            </div>

            <Show when=move || filtered_show.get().is_empty()>
                <p class="empty-state">"ไม่พบบทความที่ตรงกับเงื่อนไขที่ค้นหา 🤔"</p>
            </Show>
        </div>
    }
}
