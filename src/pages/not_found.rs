use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::content::site;

/// 404 / post-not-found fallback.
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="container notfound">
            <Title text=format!("ไม่พบหน้านี้ · {}", site::TITLE)/>
            <div class="notfound-inner">
                <p class="notfound-code">"404"</p>
                <h1 class="notfound-title">"ไม่พบหน้าที่คุณมองหา"</h1>
                <p class="notfound-sub">
                    "ดูเหมือนว่าบทความนี้จะไม่มีอยู่ หรือถูกย้ายไปแล้ว"
                </p>
                <A href="/">
                    <span class="btn-primary">"← กลับไปหน้าแรก"</span>
                </A>
            </div>
        </div>
    }
}
