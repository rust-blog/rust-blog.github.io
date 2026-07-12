use leptos::prelude::*;
use leptos_meta::Title;

use crate::content::site;

/// About page.
#[component]
pub fn About() -> impl IntoView {
    view! {
        <div class="container about-page">
            <Title text=format!("เกี่ยวกับ · {}", site::TITLE)/>
            <article class="article">
                <header class="article-header">
                    <h1 class="article-title">"เกี่ยวกับบล็อกนี้"</h1>
                </header>
                <div class="prose">
                    <p>
                        "ยินดีต้อนรับสู่ "
                        <strong>{site::TITLE}</strong>
                        " — บล็อกที่รวบรวมบันทึกการเรียนรู้เกี่ยวกับ "
                        <strong>"Rust"</strong>
                        ", "
                        <strong>"WebAssembly"</strong>
                        " และการพัฒนาเว็บสมัยใหม่"
                    </p>
                    <p>
                        "สิ่งที่พิเศษคือ บล็อกนี้ไม่ได้เขียนด้วย JavaScript "
                        "แต่วางรากฐานด้วย "
                        <strong>"Rust"</strong>
                        " และเฟรมเวิร์ก "
                        <strong>"Leptos"</strong>
                        " (Client-Side Rendering) จากนั้นถูกคอมไพล์เป็น WebAssembly "
                        "และทำงานอยู่บนเบราว์เซอร์ของคุณโดยตรง 🦀"
                    </p>
                    <h2>"เทคโนโลยีที่ใช้"</h2>
                    <ul>
                        <li>"ภาษา: Rust (Edition 2024)"</li>
                        <li>"เฟรมเวิร์ก: Leptos 0.8 (CSR)"</li>
                        <li>"ตัวจัดการ Build: Trunk"</li>
                        <li>"โฮสติ้ง: GitHub Pages"</li>
                    </ul>
                    <h2>"เพิ่มบทความใหม่อย่างไร?"</h2>
                    <p>
                        "ทุกบทความเป็นไฟล์ Markdown ธรรมดาในโฟลเดอร์ "
                        <code>"content/posts/"</code>
                        " เพียงแค่สร้างไฟล์ใหม่ นำเข้า frontmatter (ชื่อเรื่อง, วันที่, แท็ก) "
                        "และเขียนเนื้อหาตามปกติ บล็อกจะแสดงบทความนั้นให้โดยอัตโนมัติ"
                    </p>
                    <p>
                        "ดูซอร์สโค้ดทั้งหมดได้ที่ "
                        <a href=site::GITHUB_URL target="_blank" rel="noreferrer">
                            {site::GITHUB_URL}
                        </a>
                    </p>
                </div>
            </article>
        </div>
    }
}
