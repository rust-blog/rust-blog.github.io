use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::components::PostCard;
use crate::content::{site, Post};
use crate::pages::not_found::NotFound;
use crate::util::{format_date, highlight_code_blocks};

/// Individual post page, rendered from embedded markdown.
#[component]
pub fn Post() -> impl IntoView {
    let params = use_params_map();
    let posts = expect_context::<Vec<Post>>();

    let slug = {
        let params = params.clone();
        move || params.get().get("slug").unwrap_or_default()
    };

    // (current post, related posts) — recomputed when the slug changes.
    let data = Memo::new({
        let posts = posts.clone();
        let slug = slug.clone();
        move |_| {
            let s = slug();
            posts
                .iter()
                .position(|p| p.slug == s)
                .map(|idx| {
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

    // Re-run syntax highlighting whenever the rendered post changes.
    let slug_effect = slug.clone();
    Effect::new(move |_| {
        let _ = slug_effect();
        highlight_code_blocks();
    });

    let data_for_view = data.clone();

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
                        let html = p.html.clone();

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
                                        <span class="back-link">"← กลับไปหน้าแรก"</span>
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
                                        <span>{format!("{reading} นาทีในการอ่าน")}</span>
                                        <span class="dot">"·"</span>
                                        <span>{author}</span>
                                    </div>
                                </header>

                                <div class="prose" inner_html=html></div>
                            </article>

                            <Show when=move || has_related>
                                <section class="related">
                                    <h2 class="related-title">"บทความอื่นๆ ที่น่าสนใจ"</h2>
                                    <div class="post-grid">
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
