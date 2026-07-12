use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Storage, window};

/// Detect the router base path at runtime.
///
/// On GitHub Pages the app is served from `/<repo>/`, so the router needs a
/// matching base. Locally (and on any custom domain) the base is empty.
pub fn detect_base() -> String {
    let path = window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    if path.starts_with("/rust-blog") {
        "/rust-blog".to_string()
    } else {
        String::new()
    }
}

/// Trigger highlight.js over every `<pre><code>` block in the document.
///
/// highlight.js is loaded from a CDN and may not be ready on the first paint,
/// so we poll with `requestAnimationFrame` until the global is available.
pub fn highlight_code_blocks() {
    fn try_hljs() -> bool {
        let global = js_sys::global();
        let hljs = match js_sys::Reflect::get(&global, &JsValue::from_str("hljs")) {
            Ok(v) if !v.is_undefined() && !v.is_null() => v,
            _ => return false,
        };
        let f = match js_sys::Reflect::get(&hljs, &JsValue::from_str("highlightAll")) {
            Ok(v) => v,
            _ => return false,
        };
        let f: js_sys::Function = match f.dyn_into() {
            Ok(f) => f,
            Err(_) => return false,
        };
        let _ = f.call0(&hljs);
        true
    }

    if !try_hljs()
        && let Some(win) = window()
    {
        let cb = Closure::once(Box::new(highlight_code_blocks) as Box<dyn FnMut()>);
        let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

fn local_storage() -> Option<Storage> {
    window()?.local_storage().ok().flatten()
}

const THEME_KEY: &str = "rust-blog:theme";

/// Read the persisted theme preference, falling back to the OS setting.
pub fn load_theme() -> Theme {
    if let Some(storage) = local_storage()
        && let Ok(Some(v)) = storage.get_item(THEME_KEY)
    {
        return match v.as_str() {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => system_theme(),
        };
    }
    system_theme()
}

pub fn save_theme(theme: Theme) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(THEME_KEY, theme.as_str());
    }
}

fn system_theme() -> Theme {
    let dark = window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false);
    if dark { Theme::Dark } else { Theme::Light }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn toggle(&self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

/// Format an ISO `YYYY-MM-DD` date into a friendly Thai-locale string.
pub fn format_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 {
        return iso.to_string();
    }
    let months = [
        "ม.ค.",
        "ก.พ.",
        "มี.ค.",
        "เม.ย.",
        "พ.ค.",
        "มิ.ย.",
        "ก.ค.",
        "ส.ค.",
        "ก.ย.",
        "ต.ค.",
        "พ.ย.",
        "ธ.ค.",
    ];
    let year = parts[0];
    let month: usize = parts[1].parse().unwrap_or(0);
    let day = parts[2].trim_start_matches('0');
    if (1..=12).contains(&month) {
        format!("{day} {} {year}", months[month - 1])
    } else {
        iso.to_string()
    }
}

/// Apply the active theme to the document root element.
pub fn apply_theme(theme: Theme) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        let _ = doc.document_element().map(|el| {
            let _ = el.set_attribute("data-theme", theme.as_str());
        });
    }
}

#[allow(dead_code)]
pub fn document() -> Option<Document> {
    window().and_then(|w| w.document())
}
