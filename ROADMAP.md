# rust-blog Roadmap

This roadmap describes what rust-blog is, honestly, from reading its own code -
and where it should end up. It follows the conventions in
[README.md](README.md) and the documentation discipline of recording not just
*what* will be built but *why*, *what the tradeoffs were*, and *where each known
gap gets closed*: verify directly, record the decision, name the phase that
closes it. Nothing here is called "done" on intent alone - the repo already has
a real CI (`.github/workflows/deploy.yml`: checkout → Rust + `wasm32` target →
pinned Trunk → `trunk build --release` → GitHub Pages), and every phase's
acceptance is checked against it.

> **What rust-blog is.** A *quiet, personal* blog engine written *in* Rust and
> compiled *to* WebAssembly. The app is a [Leptos](https://leptos.dev/) 0.8
> client-side-rendered (CSR) SPA, built with [Trunk](https://trunkrs.dev/) and
> published to GitHub Pages (`https://rust-blog.github.io/`). Posts
> are plain Markdown files in `content/posts/` - no database, no server, no
> JavaScript authored by hand. The site is primarily Thai-language. Drop a
> `.md` file in and it appears, indexed, searchable, and syndicated, with zero
> per-post code changes.
>
> **What rust-blog is not.** Not a CMS, not a multi-user platform, not a
> hosted SaaS. There is no admin panel, no comments backend, no accounts, and
> nothing in the data model points that way. The single-binary, static,
> zero-server, reader-respecting shape is the product, not a stepping stone to
> something larger. Features that break that shape are listed under
> "Out of Scope" so the line is drawn on purpose.

---

## Current State (verified against the repo, not assumed)

| Aspect            | Today                                                            |
| ----------------- | --------------------------------------------------------------- |
| Language / stack  | Rust edition 2024, Leptos 0.8 (CSR), Trunk, `wasm32-unknown-unknown` |
| Rendering         | Client-side in the browser (WASM), no SSR                       |
| Content           | 2 Markdown posts in `content/posts/` (`welcome`, `rust-variables`) |
| Interactive demos | markdown `demo` directive mounts live Rust/WASM components (e.g. counter) |
| Styling           | Hand-written CSS design system, light/dark, no framework        |
| Syndication       | `rss.xml` built by `build.rs`, copied to `dist/`                |
| Search / filter   | In-memory substring scan + tag chips on the home page           |
| Hosting / CI      | GitHub Pages via `deploy.yml` (fmt + clippy + test + trunk gates); Renovate on deps |
| Tests             | 23 unit + 12 integration (golden HTML, RSS round-trip, sitemap, render fuzzing) |
| Third-party JS    | None - zero runtime network requests (highlight.js and Google Fonts removed, syntect at build, system Thai font stack) |
| License           | MIT                                                             |

### Gaps found while reading the repo (these shape the phases below)

1. ~~CI is build-only.~~ **Resolved:** `deploy.yml` now runs `cargo fmt
   --check`, `cargo clippy -D warnings` (wasm target), and `cargo test` as
   required build steps before `trunk build --release`. (Phase 9.)
2. ~~Frontmatter is parsed in two places~~ (`build.rs::parse` and
   `content.rs::parse_post`) with slightly different body handling. They agree
   today but can drift - the RSS feed and the in-app render could disagree
   about a post's title, date, or slug.
   **Resolved:** one shared parser (`src/frontmatter.rs`) is used by both
   `build.rs` and `content.rs`; typed errors, strict `YYYY-MM-DD` calendar
   validation, and a malformed post now fails the build loudly. (Phase 8.)
3. ~~`<html lang="en">` hardcoded.~~ **Resolved:** root `lang="th"` set in
   `index.html`; matches the primarily-Thai content. (Phase 5.)
4. ~~The only runtime network dependency is highlight.js from cdnjs.~~
   **Resolved:** highlight.js was removed entirely; `syntect` now highlights
   code at render time. The Google Fonts `<link>` (Bai Jamjuree/Sarabun) was
   also removed - the design system now uses the system Thai font stack
   (`styles/main.css`), so the app makes **zero runtime network requests** and
   ships no third-party JavaScript. (Closed - no Phase 8 work needed.)
5. **Search is an O(n) substring scan.** Correct at the current scale (~2
   posts) but unmeasured and unbounded as content grows. (Phase 8.)
6. **No content safety net.** A malformed post, a broken markdown edge case,
   or an invalid date is neither tested nor guaranteed to fail the build
   loudly - it could ship a silent blank page or a malformed RSS feed. (Phase 7.)
7. **No release provenance.** The deployed `dist/` has no checksum or
   verifiable artifact tying it back to the tagged source. (Phase 10.)

---

## Foundation

### Phase 0 - Scaffold (done)

- [x] Cargo package (edition 2024), single WASM binary via Leptos 0.8 CSR
- [x] Trunk 0.21.14 pinned in CI; `wasm32-unknown-unknown` target
- [x] Custom CSS design system (light/dark tokens, no framework)
- [x] `build.rs` generates `rss.xml` at build time
- [x] CI deploys to GitHub Pages with SPA `404.html` fallback
- [x] `renovate.json`, committed `Cargo.lock`, `README.md`, `rustfmt.toml`

**Acceptance (met):** `trunk build --release` deploys a working SPA to GitHub
Pages from a clean checkout.

### Phase 1 - Content model & markdown (done)

- [x] `PostMeta` / `Post` models; embedded `include_dir!` content (no runtime fetch)
- [x] Recursive post collection; `pulldown-cmark` with `Options::all()`
- [x] BOM-tolerant parsing; `all_tags()`; reading-time estimate
- [x] Unit test `loads_embedded_posts` (≥2 posts, newest-first)

**Acceptance (met):** at least two posts render and sort correctly.

### Phase 2 - App shell, routing, theme (done)

- [x] `main.rs`: meta + theme provider + router (base-path detection) + post context
- [x] Routes `/`, `/about`, `/post/:slug`, fallback `NotFound`
- [x] Theme: OS detection + `localStorage` persistence; `util.rs` (base, Thai date)
- [x] Home search (title/description/tags substring) + tag filtering; Post page with OG meta + related
- [x] Thai typography (system font stack, no webfonts), hairline monogram logo, equal home/article content widths
- [x] Live in-post demos: markdown `demo` directive mounts interactive Rust/WASM components (e.g. Leptos-signal counter)

**Acceptance (met):** routing, theme toggle, and basic search work in a real
browser.

---

## Content

### Phase 3 - Build reproducibility & RSS (done, one open)

- [x] `rss.xml` written by `build.rs`, copied to `dist/` as a Trunk
  `copy-file` asset (see below; the original post-build hook was wiped by
  Trunk's final dist assembly and has been removed).
- [x] Items carry title/link/GUID/description/pubDate/categories; drafts excluded

- [x] **Closed (closes gap 6):** RSS `pubDate` relies on implicit `chrono` date
  validation with no explicit test. A bad date must fail the build loudly, not
  emit a malformed feed. — Done: `src/frontmatter.rs` validates
  `YYYY-MM-DD` calendar dates with typed errors (11 tests); `build.rs` panics
  with the file path on any malformed post (verified: `2024-02-30` → exit 101).

**Acceptance:** feed item count == published post count; all dates valid
calendar dates; build fails on any invalid date.

### Phase 4 - Authoring experience (done, growth open)

- [x] "Drop a `.md` → appears" model; documented frontmatter contract; `draft`, `slug`, `author`

- [x] **Frontmatter linter in CI** (advisory warnings, not build failures):
  `lint_post` in `frontmatter.rs` flags missing `description`, future
  `date` (with a "use `draft: true`" hint), and single-use tags (possible
  typos); drafts are exempt by contract. `build.rs` prints the warnings
  with the file path; malformed posts still fail the build hard.
- [ ] **Open:** no post template / `cargo xtask new` - authors copy by hand.
- [ ] **Open:** no `content/assets/` handling (images un-fingerprinted).

**Acceptance:** `cargo xtask new "<title>"` scaffolds a valid post; assets
fingerprint into `dist/`; linter rejects malformed frontmatter in CI.

### Phase 5 - Accessibility & SEO (partial)

- [x] Per-page `<title>`, post description + OG tags, `color-scheme`, inline SVG favicon, semantic landmarks, `aria-label`s

- [x] **(closes gap 3):** `<html lang="en">` → `lang="th"` on the document root,
  matching the primarily-Thai content. Dynamic per-post `lang` remains a future
  enhancement.
- [x] `robots.txt` + `sitemap.xml` generated by `build.rs` (home, about, every
  published post with `lastmod`), copied to `dist/` by the Trunk post-build
  hook; covered by `tests/sitemap_robots`.
- [x] **Share cards:** static `og:title` / `og:description` / `og:type` /
  `og:image` / `og:url` + `twitter:card` / `twitter:image` in `index.html`, plus
  a generated 1200x630 `og-image.png` (brand banner) copied to `dist/` - so
  Facebook and other crawlers (which don't run JS) get a preview card.
- [x] **Focus styling + skip-link:** `:focus-visible` rings in the design
  system; a "Skip to content" link (hidden until focused) jumps to
  `main#main`. `prefers-reduced-motion` was verified present (`main.css`)
  and needs no further work.

**Acceptance:** valid `lang`; visible focus rings; `robots.txt` + `sitemap.xml`
generated; no a11y lint failures.

### Phase 6 - Asset pipeline & multi-format (open)

- [ ] Fingerprint `content/assets/**` into `dist/` (Trunk already fingerprints CSS/JS).
- [ ] **Deliberately skipped:** Code-fence language label - implemented and
  then removed on design review: the plate speaks for itself and the label
  added furniture. (Highlighting for known languages stays, as always.)
- [ ] **Deliberately skipped:** TOC from `##`/`###` headings - implemented
  once (heading ids, `Rendered { html, toc }`, sticky sidebar + mobile
  `<details>`) and then removed on design review: the single quiet column
  is the product, and a sidebar/repeated headings box adds navigation
  furniture the readers did not ask for. Revisit on demand.
- [ ] **Deliberately skipped:** KaTeX - heavy runtime dep for content that does
  not need it yet. Revisit on demand.
- [ ] Local draft preview (`?preview=1` includes drafts).

**Acceptance:** images load with cache-busting hashes; TOC renders; preview
shows drafts without publishing.

---

## Quality

### Phase 7 - Content safety test suite (done, closes gap 6)

- [x] `tests/golden_content`: exact rendered-HTML assertions for a sample post
  (headings, code fence, footnote, table). A markdown change that alters
  output breaks the test. — Caught and fixed a real bug: syntect's trailing
  `</pre>` survived the trim, so every code block shipped a stray closing
  tag (`markdown.rs` now trims and strips it; pinned by the golden).
- [x] Date-validation tests: `2024-02-30`, `2024-13-01`, empty, non-`YYYY-MM-DD`
  all rejected at parse time with a typed error. — covered by
  `frontmatter.rs` tests (`rejects_invalid_calendar_dates`,
  `missing_date_is_an_error`).
- [x] Frontmatter schema tests: missing `title`/`date`, unknown keys (warn),
  duplicate tags (dedupe) behave deterministically. — `parse` returns
  sorted `warnings` for unknown keys (printed by `build.rs`), tags are
  deduplicated, missing `title`/`date` are typed errors.
- [x] Proptest: any string pulldown-cmark accepts renders without panicking.
  — `tests/render_never_panics` (512 cases × 3 strategies: arbitrary
  markdown, arbitrary frontmatter, head+body).
- [x] RSS round-trip: parse `rss.xml` back, assert count == published posts.
  — `tests/rss_round_trip`: item count == published posts, every item
  matches a post by slug/title with a pubDate, channel fields sane.

**Acceptance (met):** every item above has a passing test; a bad post fails
`cargo test`, not production.

### Phase 8 - Correctness & performance hardening (open, closes gaps 2, 4, 5)

- [x] **(gap 2)** Extract a **single shared parser** used by `build.rs` and
  `content.rs`; RSS and in-app render become impossible to disagree. — Done:
  `src/frontmatter.rs` with typed errors + strict date validation; both sides
  call the same `parse()`.
- [ ] **(gap 3-adjacent)** Set `lang="th"` (or dynamic per-post).
- [ ] **(gap 5)** Search scaling: prebuilt embed-time inverted index when
  justified; naive path stays correct for small N. No premature `tantivy`/
  `fuzzy` dependency.
- [x] **(gap 4)** highlight.js removed; `syntect` highlights at render time;
  Google Fonts dropped for the system Thai font stack → **zero runtime network
  requests, no third-party JS**. (Closed.)
- [ ] First-paint budget: document TTI on a throttled link; WASM gzip ceiling
  (e.g. `< 300KB`) verified in CI, not claimed.
- [x] a11y audit: focus rings (`:focus-visible`), `prefers-reduced-motion`
  (verified present), skip-to-content (added to Nav).

**Acceptance:** one parser; `lang` correct; WASM under budget in CI; zero
runtime network requests; a11y lint clean.

### Phase 9 - CI hardening & supply chain (partial, closes gap 1)

- [x] **(gap 1)** Required checks before deploy: `cargo fmt --check`,
  `cargo clippy --workspace --target wasm32-unknown-unknown -D warnings`,
  `cargo test --workspace`, then `trunk build --release` (all in `deploy.yml`).
  Branch protection to *require* these as merge gates is a GitHub setting (see
  open item below).
- [x] Pin the Rust toolchain via `rust-toolchain.toml` (channel + wasm32 target
  + rustfmt/clippy components) so CI == local.
- [x] Confirmed **zero hand-written `unsafe`** in `src/` (only `wasm-bindgen`/
  `js-sys` FFI); keep it that way.
- [x] All workflow actions pinned to full commit SHAs (zizmor-clean);
  `persist-credentials: false` set; `pages`/`id-token` write scopes limited to
  the deploy job only.

- [x] **`cargo audit` + `cargo deny` (license + advisory) as CI steps.**
  Both gates run in `deploy.yml` via pinned `taiki-e/install-action`.
  `deny.toml` allows only permissive licenses (MIT/Apache/BSD/BSL/ISC/
  Unlicense/Unicode-3.0/Zlib/CC0); four unavoidable "unmaintained"
  advisories (paste, proc-macro-error2, bincode, yaml-rust - all at their
  latest versions, all build-time or syntect-pinned) are recorded with
  reasons in `deny.toml` + `.cargo/audit.toml`. Security advisories are
  fixed, not ignored: `bytes` 1.11→1.12 (RUSTSEC-2026-0007), `anyhow`
  →1.0.104, `event-listener` →5.4.2 were upgraded.
- [ ] **Open:** Branch protection on `main`: strict required checks, no
  force-push, no deletion (GitHub repo setting, not a file change).
- [ ] **Open:** Preview deploys from PRs to a non-prod environment.

**Acceptance:** a lint/test regression cannot merge; audit/deny green; branch
protection enforced; preview URL per PR.

---

## Release

### Phase 10 - First public v1.0.0 (open, closes gap 7)

- [ ] `v1.0.0` tag + documented release process.
- [x] `robots.txt` + `sitemap.xml` generated at build time (closes Phase 5).
  — done early with the Phase 5 work; only `og:image`/Twitter cards remain
  open there.
- [ ] **(gap 7)** Deploy provenance: publish `dist/` as a release artifact with
  `SHA256SUMS.txt` so readers can verify the served site matches the tag.
- [ ] `CONTRIBUTING.md`: frontmatter contract, draft model, asset rules, how
  to run full CI locally before pushing.
- [ ] `cargo xtask new` scaffolding (closes Phase 4 growth item).

**Acceptance:** tagged release with checksums; contributor can scaffold and
validate a post locally; site verifiable against the tag.

---

## Future / Ecosystem

- [ ] **Multi-locale** (Thai + English) via a `lang` frontmatter field + locale
  switcher - the Phase 5 `lang` bug becomes the seed of this feature.
- [ ] **Archive pages** `/tag/:tag`, `/author/:name` reusing Home filtering.
- [x] **Lightweight SSG for posts** - `build.rs` emits `posts-manifest.json`
  and a `post_build` Trunk hook (`scripts/post_build.py`) writes one static
  `dist/post/<slug>/index.html` per published post (the built SPA, with
  per-post OG/Twitter/`<title>` swapped in). GitHub Pages now serves
  `/post/<slug>` as a real 200 (with a correct share card) instead of the
  generic `404.html` fallback - fixes Facebook 404 + gives direct links that
  survive a refresh. Full no-JS/SSR snapshot for every route stays a future
  option; CSR remains the default.
- [ ] **Offline / PWA** - service worker caching WASM + assets; natural fit for
   a zero-server static site (hljs already removed in Phase 8).
- [ ] **Series/collection** - a `series` field grouping posts into an ordered
  reading list ("Part N of M").
- [ ] **Privacy-respecting comments** (Webmention or external service, never a
  tracking script).
- [ ] **Visual regression tests** (per-route screenshots in CI) once the
  design system is stable.

---

## How the phases relate

```
Phase 0-2 (scaffold, content, shell)  ─┐
Phase 3-6 (content correctness,         │ foundation - the blog
           authoring, a11y, assets)     │ must be correct
                                        ┘ before it can be trusted
        │
        ▼
Phase 7 (content safety tests)  ──┐
Phase 8 (hardening: parser,        ├─► Quality - verify, then prove
            lang, search, highlighting) │
Phase 9 (CI gates, audit/deny)  ──┘
        │
        ▼
Phase 10 (v1.0.0: checksums, docs, tag)
        │
        ▼
Future (multi-locale, SSG, PWA, series)
```

Phase 0-2 are the running scaffold; Phase 3-6 make content correct and
comfortable; Phase 7-9 earn trust with tests, a single parser, and CI gates
that can't be bypassed; Phase 10 is the first verifiable release. The Future
section only extends the one engine Tome-style calm tools keep - it never adds
a second product.

---

## Out of Scope (drawn on purpose, to stay a quiet personal blog)

Each of these is valuable *for a different product*. rust-blog stays small,
static, and single-reader on purpose:

- **CMS / admin panel / accounts** - rust-blog is a static site; there is no
  server, no auth, and no notion of a user. It stays that way.
- **Server-side comments / community** - deferred indefinitely; adds a backend
  and a moderation surface a static, reader-respecting blog shouldn't carry.
  (Privacy-respecting Webmention in Future is the only tolerated exception, and
  is explicitly non-tracking.)
- **Multi-author / team publishing** - out of scope; conflicts with the
  single-content-owner, no-accounts shape.
- **A full web framework / SSR** - rust-blog is CSR-by-design to demonstrate
  Rust→WASM; an SSG *option* (Future) is the most it will ever bend here.
- **Telemetry / analytics on reader behavior** - explicitly never; the project
  ships no tracker and no third-party analytics.
- **A hosted SaaS version** - out of scope; the engine is MIT so anyone can
  fork and self-host, but there is no hosted service.

---

### How to read this roadmap

- `[x]` done and verified in the current tree; `[ ]` open.
- The numbered **gaps** at the top are the debt driving the phases; each open
  item cites the gap it closes (e.g. "(closes gap 2)") so nothing is silently
  dropped.
- **Acceptance** states the measurable bar each phase must clear before it is
  marked done.
- The destination (v1.0.0) is a content engine that is **safe by construction,
  verified by tests, reproducible by build, and maintainable by CI** - not
  merely a Leptos SPA that renders two Thai Markdown posts today.
