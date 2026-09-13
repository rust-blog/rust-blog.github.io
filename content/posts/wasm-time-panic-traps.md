---
title: "Time and Panic Traps in WebAssembly: It Compiles, but It Crashes in the Browser"
date: "2026-09-13"
description: "Time and panic traps in WebAssembly that compile and pass tests, yet break only when they run in the browser, with fixes you can actually use."
tags: [rust, wasm, leptos]
author: "suradet-ps"
---

If you have ever put Rust on the web with Leptos, Yew, or Sycamore, you have probably run into this: everything passes on your machine, but the browser gives you a blank page or just `RuntimeError: unreachable executed`.

In this post I want to walk through the traps I hit myself while building a local-first app, and how many rounds it took before I found the root cause. I hope it helps you debug the same symptoms faster.

The root cause is that `wasm32-unknown-unknown` is not an operating system. It has no clock, no threads, and no filesystem. Some `std` functions are therefore just empty shells. They compile fine, but the moment you call them, they panic.

Worse, `cargo test` runs on your machine, not on wasm, so bugs like this slip through CI easily. Let's go through the traps one by one.

---

## 1. Why CI Cannot Catch These Bugs

Think about what the tools we use every day actually check.

- `cargo test` runs code on your machine (macOS/Linux), where std works fully, so it never hits the problem.
- `cargo check --target wasm32-unknown-unknown` only checks that the code compiles, and an empty shell compiles too.
- `cargo clippy --target wasm32-unknown-unknown` checks lints, which have nothing to do with runtime behavior.
- `trunk build --release` only assembles files. It never runs the code.

So everything is green, but the moment you open the page you get `RuntimeError: unreachable executed`, a blank page, or worse, the app keeps running with quietly wrong values. CI says pass, users say broken.

---

## 2. Trap 1: `std::time` Does Not Work on wasm

The first thing I hit was time. If you look at the `std` source, on a platform with no clock, `SystemTime::now()` and `Instant::now()` are written like this:

```rust
impl Instant {
    pub fn now() -> Instant {
        panic!("time not implemented on this platform")
    }
}

impl SystemTime {
    pub fn now() -> SystemTime {
        panic!("time not implemented on this platform")
    }
}
```

Code that calls `SystemTime::now()` compiles fine, but it panics the moment it runs in the browser. The good news is that `std::time::Duration` still works, because it is just a numeric value. It is only reading the clock that fails.

The usual fix is **[`web-time`](https://docs.rs/web-time)**, a drop-in replacement for `std::time`. On wasm it uses `Date.now()` and `performance.now()`, and on your machine it just re-exports `std::time`. The same code runs in both places, and you can still test it natively.

```toml
# Cargo.toml
web-time = { version = "1", default-features = false, features = ["serde"] }
```

I like to keep all clock reads in one place, for example `core/time.rs`, and let the rest of the app go through that file.

```rust
use web_time::{SystemTime, UNIX_EPOCH};

/// Current time as an RFC-3339 / ISO-8601 string (stable, no locale drift).
pub fn now_iso() -> String {
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_secs() as i64);
  chrono::DateTime::from_timestamp(secs, 0)
    .map_or_else(String::new, |d| d.to_rfc3339())
}
```

The nice part is that `chrono` only turns numbers into text and never reads the clock itself, so `web_time` stays the single place that touches time. That makes it easy to audit, and the value it returns is UTC RFC-3339, which compares cleanly and does not depend on the locale.

---

## 3. Trap 2: Turning Off default-features and Forgetting to Turn a Feature Back On

This one hurts because the symptoms are exactly the same as Trap 1, but the cause is in `Cargo.toml`. wasm projects like to disable a dependency's `default-features` to shrink the binary, and `chrono` is usually one of them. The feature people forget is `wasmbind`.

By default, `chrono` 0.4 enables `wasmbind` for you. But if you write this:

```toml
chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }
```

then `wasmbind` disappears too, and `Utc::now()` reaches for `std::time::SystemTime::now()` internally. Same panic as Trap 1. The fix is just to add it back:

```toml
chrono = { version = "0.4", default-features = false, features = [
  "clock", "std", "wasmbind",
] }
```

The lesson: the moment you disable `default-features`, you are signing a contract to turn the necessary features back on yourself. Leave a comment about why you chose that set, or six months later you will be wondering where `wasmbind` came from.

Another interesting case is `uuid`. On wasm you have to pick a source of randomness explicitly. If you forget to enable the `js` feature (or `rng-getrandom` / `rng-rand`), you get a compile error at build time, not a runtime panic. That is actually good, since the compiler catches it early, but you should still know about it: the error points straight at the feature and does not tell you what you forgot.

Client-side ids do not have to use `uuid`, though. I generate mine from a timestamp plus `Math.random()` through `wasm-bindgen` directly. That is plenty for data stored in `localStorage` on a single machine.

```rust
/// Generate a short, collision-resistant id for client-side records.
pub fn generate_id() -> String {
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_millis());
  let rand: u32 = js_sys_random();
  format!("{secs:x}{rand:x}", rand = rand & 0xff_ffff)
}

fn js_sys_random() -> u32 {
  #[wasm_bindgen]
  extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
  }
  (random() * f64::from(u32::MAX)) as u32
}
```

---

## 4. Trap 3: `Instant` and `SystemTime` Are Not the Same

Once the panic is fixed, the next trap is using the wrong type. `web-time` gives you both `Instant` and `SystemTime`, but they do different jobs.

- `SystemTime` sits on `Date.now()`. Use it to record what time it is.
- `Instant` sits on `performance.now()`. Use it to measure elapsed time between two points.

`SystemTime` follows the real clock, which the user or the system can change at any time (NTP sync, or someone setting the clock by hand). If you subtract two `SystemTime` values to measure duration, the result can go negative or drift. `Instant` is a monotonic clock that only moves forward, which makes it the right tool for durations.

Timezones do not affect the stored value at all, since they only matter when you format it as local text.

For local-first data, I always store time as **UTC RFC-3339** and convert to local time only for display. Day handling goes through helpers, so it never depends on the browser timezone.

```rust
/// Number of days from now as an `YYYY-MM-DD` date string.
pub fn date_plus_days(days: i64) -> String {
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_secs() as i64);
  let target = secs + days * 86_400;
  /* ... format as %Y-%m-%d ... */
}

/// Whole days between `a` and `b` (`b - a`), or `None` if either is invalid.
pub fn days_between(a: &str, b: &str) -> Option<i64> {
  use chrono::NaiveDate;
  let da = NaiveDate::parse_from_str(a, "%Y-%m-%d").ok()?;
  let db = NaiveDate::parse_from_str(b, "%Y-%m-%d").ok()?;
  Some((db - da).num_days())
}
```

`days_between` works for both spaced repetition schedules (1/3/7/14/30 days) and a weekly streak counter. Dates are stored as plain `YYYY-MM-DD` strings, which compare directly with no timezone conversion.

> **Watch out** Adding `86_400` seconds is always one UTC day. If you need calendar days in the user's local time (which has DST), that is a separate timezone problem. Do not let a simple formula turn into a silent bug.

---

## 5. Trap 4: Panics That Tell You Nothing

The big problem with panics on wasm is that they tell you nothing. You see `unreachable executed`, or some browsers show only `RuntimeError`. The fix is to install a panic hook that pushes the message and stack trace to the console.

```rust
#[wasm_bindgen(start)]
pub fn start() {
  // Install this first so every panic from here on is readable.
  console_error_panic_hook::set_once();
  console_log::init_with_level(log::Level::Debug).ok();

  mount_to_body(|| {
    view! { <App /> }
  });
}
```

`set_once()` is the best-value line in this whole file. Without it, you waste time hunting for a panic with an empty message. Always call it first inside `#[wasm_bindgen(start)]`.

As for a release profile with `panic = "abort"` (which is already the default for `wasm32-unknown-unknown`), you still get the full panic message, because the hook runs before the abort.

---

## 6. Trap 5: Wait with JS Timers, Not `std::thread`

Code that needs to wait, such as a Pomodoro timer or a debounce, is where several traps stack up.

First, `std::thread` does not work on wasm.

- `std::thread::spawn` is unsupported, because the browser has no OS threads.
- `std::thread::sleep` panics with `can't sleep` (and even with `+atomics` you should not use it, because blocking the main thread freezes the UI).

What works is the browser's `setInterval` / `setTimeout` through `web-sys`. That leads to the second layer of the trap: **`Closure`**.

A `Closure` has to be kept alive as long as the timer is running. If you let it drop, the next interval tick throws `closure invoked recursively or after being dropped`.

The fix is `mem::forget` after the interval is set successfully.

```rust
let closure: Closure<dyn Fn()> = Closure::wrap(Box::new(tick));
let window = web_sys::window().expect("no window");
let h = window.set_interval_with_callback_and_timeout_and_arguments_0(
  closure.as_ref().unchecked_ref(),
  1000,
);
if let Ok(h) = h {
  // Keep the closure alive for the lifetime of the interval.
  std::mem::forget(closure);
  handle.set(Some(h));
}
```

And when the component unmounts, always clear the interval, or the timer keeps running in the background.

```rust
let clear = move || {
  if let Some(h) = handle.get_untracked() {
    web_sys::window().expect("no window").clear_interval_with_handle(h);
    handle.set(None);
  }
  is_running.set(false);
};

on_cleanup(clear);
```

> **Trade-off** `mem::forget` makes the timer work correctly, but the closure's memory is never returned, even after you clear the interval. For an app with a single timer that is acceptable, but if you create and destroy timers often, keep the `Closure` in a signal alongside the handle and drop it when you stop. That is cleaner.

---

## 7. Trap 6: `cargo test` Passes but the Real Thing Breaks

All of the above is why a wasm project should have CI touch `wasm32-unknown-unknown`, not just run `cargo test`. The gate I use looks like this:

```yaml
- name: clippy
  run: cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings
- name: cargo check (wasm)
  run: cargo check --target wasm32-unknown-unknown
- name: cargo test
  run: cargo test
- name: Build
  run: trunk build --release
```

But the trick that helps more is **separating the clock from the logic**. Let one module read `SystemTime::now()`, and have every other function take time in as a parameter or a date string. Then all the date logic becomes pure functions you can test on your machine right away.

```rust
#[test]
fn days_between_is_signed_and_correct() {
  assert_eq!(days_between("2026-07-12", "2026-07-19"), Some(7));
  assert_eq!(days_between("2026-07-19", "2026-07-12"), Some(-7));
  assert_eq!(days_between("nonsense", "2026-07-19"), None);
}
```

If some code truly has to run in the browser, like a timer or a service worker, add `wasm-bindgen-test` and write `#[wasm_bindgen_test]` tests. That is the last line of defense for the things native tests cannot reach.

---

## 8. Summary

If you want a short list to review before writing code, try to remember these:

- `std::time::SystemTime::now()` panics with `time not implemented on this platform`. Fix: use `web-time`.
- `Instant::now()` panics the same way. Fix: use `web_time::Instant`.
- `chrono` with default-features off makes `Utc::now()` panic. Fix: add the `wasmbind` feature.
- `uuid` without the `js` feature gives a compile error at build time. Fix: add the `js` feature (or `rng-getrandom` / `rng-rand`).
- `std::thread::spawn` / `sleep` panic or freeze the UI. Fix: use `setInterval` / `setTimeout`.
- A dropped `Closure` throws `closure invoked recursively or after being dropped`. Fix: keep the closure alive and clear the timer in `on_cleanup`.
- Panics with no message show only `unreachable executed`. Fix: `console_error_panic_hook::set_once()`.
- Native tests passing alone lets bugs reach production. Fix: add `cargo check/clippy --target wasm32` and `wasm-bindgen-test`.

If you only remember one sentence, remember this: **compiling does not mean it runs**. Especially on `wasm32-unknown-unknown`, where much of std is an empty shell waiting to panic.

Three habits help a lot: read the clock in one place, keep time logic pure so it can be tested natively, and make CI check the wasm target every time. Do all three and these traps get caught before you even open the browser, so you never have to guess what caused that `unreachable executed` again.
