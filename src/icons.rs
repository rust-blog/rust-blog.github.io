use leptos::prelude::*;

#[component]
pub fn RustLogo() -> impl IntoView {
  view! {
      <svg viewBox="0 0 100 100" width="1.2em" height="1.2em" aria-hidden="true">
          <path d="M50 5 L60 35 L85 35 L65 55 L75 95 L50 70 L25 95 L35 55 L15 35 L40 35 Z" fill="currentColor"/>
      </svg>
  }
}

#[component]
pub fn SearchIcon() -> impl IntoView {
  view! {
      <svg viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
  }
}

#[component]
pub fn SunIcon() -> impl IntoView {
  view! {
      <svg viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="5"/>
          <line x1="12" y1="1" x2="12" y2="3"/>
          <line x1="12" y1="21" x2="12" y2="23"/>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
          <line x1="1" y1="12" x2="3" y2="12"/>
          <line x1="21" y1="12" x2="23" y2="12"/>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
      </svg>
  }
}

#[component]
pub fn MoonIcon() -> impl IntoView {
  view! {
      <svg viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.79 A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>
  }
}

#[component]
pub fn HeartIcon() -> impl IntoView {
  view! {
      <svg viewBox="0 0 24 24" width="1em" height="1em" fill="currentColor" stroke="none" aria-hidden="true">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
      </svg>
  }
}
