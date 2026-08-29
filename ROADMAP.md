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
| Styling           | Hand-written CSS design system, light/dark, no framework        |
| Syndication       | `rss.xml` built by `build.rs`, copied to `dist/`                |
| Search / filter   | In-memory substring scan + tag chips on the home page           |
| Hosting / CI      | GitHub Pages via `deploy.yml`; Renovate on deps                 |
| Tests             | 1 unit test (`loads_embedded_posts`)                            |
| Third-party JS    | 1 script: highlight.js 11.11.1 from cdnjs (no SRI)              |
| License           | MIT                                                             |

### Gaps found while reading the repo (these shape the phases below)

1. **CI is build-only.** It runs `trunk build --release` and deploys, but
   never `cargo fmt --check`, `cargo clippy -D warnings`, or `cargo test`.
   Correctness rests on developer discipline, not a required status check.
   (Phase 9.)
2. **Frontmatter is parsed in two places** (`build.rs::parse` and
   `content.rs::parse_post`) with slightly different body handling. They agree
   today but can drift - the RSS feed and the in-app render could disagree
   about a post's title, date, or slug. (Phase 8.)
3. **`<html lang="en">` is hardcoded** but the content is Thai - a real
   a11y/SEO defect for screen readers and search engines. (Phase 5.)
4. **The only runtime network dependency is highlight.js** from cdnjs, loaded
   without `integrity` (SRI) and without `crossorigin`. If cdnjs is down or
   compromised, code blocks silently lose highlighting. (Phase 8.)
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
- [x] Theme: OS detection + `localStorage` persistence; `util.rs` (base, hljs, Thai date)
- [x] Home search (title/description/tags substring) + tag filtering; Post page with OG meta + related

**Acceptance (met):** routing, theme toggle, and basic search work in a real
browser.

---

## Content

### Phase 3 - Build reproducibility & RSS (done, one open)

- [x] `rss.xml` written by `build.rs`, copied to `dist/` via Trunk post-build hook
- [x] Items carry title/link/GUID/description/pubDate/categories; drafts excluded

- [ ] **Open (closes gap 6):** RSS `pubDate` relies on implicit `chrono` date
  validation with no explicit test. A bad date must fail the build loudly, not
  emit a malformed feed.

**Acceptance:** feed item count == published post count; all dates valid
calendar dates; build fails on any invalid date.

### Phase 4 - Authoring experience (done, growth open)

- [x] "Drop a `.md` → appears" model; documented frontmatter contract; `draft`, `slug`, `author`

- [ ] **Open:** no post template / `cargo xtask new` - authors copy by hand.
- [ ] **Open:** no `content/assets/` handling (images un-fingerprinted).
- [ ] **Open:** no frontmatter linter (missing description, unknown tag, future date).

**Acceptance:** `cargo xtask new "<title>"` scaffolds a valid post; assets
fingerprint into `dist/`; linter rejects malformed frontmatter in CI.

### Phase 5 - Accessibility & SEO (partial)

- [x] Per-page `<title>`, post description + OG tags, `color-scheme`, inline SVG favicon, semantic landmarks, `aria-label`s

- [ ] **Open (closes gap 3):** `<html lang="en">` hardcoded but content is
  Thai. Fixed by setting `lang="th"` (or dynamic per-post).
- [ ] **Open:** no `robots.txt` / `sitemap.xml` / `og:image` / Twitter cards.
- [ ] **Open:** focus styling, `prefers-reduced-motion`, skip-link not audited.

**Acceptance:** valid `lang`; visible focus rings; `robots.txt` + `sitemap.xml`
generated; no a11y lint failures.

### Phase 6 - Asset pipeline & multi-format (open)

- [ ] Fingerprint `content/assets/**` into `dist/` (Trunk already fingerprints CSS/JS).
- [ ] Code-fence language label; invoke hljs only for known languages.
- [ ] TOC from `##`/`###` headings (ids already generated) in the Post sidebar.
- [ ] **Deliberately skipped:** KaTeX - heavy runtime dep for content that does
  not need it yet. Revisit on demand.
- [ ] Local draft preview (`?preview=1` includes drafts).

**Acceptance:** images load with cache-busting hashes; TOC renders; preview
shows drafts without publishing.

---

## Quality

### Phase 7 - Content safety test suite (open, closes gap 6)

- [ ] `tests/golden_content`: exact rendered-HTML assertions for a sample post
  (headings, code fence, footnote, table). A markdown change that alters
  output breaks the test.
- [ ] Date-validation tests: `2024-02-30`, `2024-13-01`, empty, non-`YYYY-MM-DD`
  all rejected at parse time with a typed error.
- [ ] Frontmatter schema tests: missing `title`/`date`, unknown keys (warn),
  duplicate tags (dedupe) behave deterministically.
- [ ] Proptest: any string pulldown-cmark accepts renders without panicking.
- [ ] RSS round-trip: parse `rss.xml` back, assert count == published posts.

**Acceptance:** every item above has a passing test; a bad post fails
`cargo test`, not production.

### Phase 8 - Correctness & performance hardening (open, closes gaps 2, 4, 5)

- [ ] **(gap 2)** Extract a **single shared parser** used by `build.rs` and
  `content.rs`; RSS and in-app render become impossible to disagree.
- [ ] **(gap 3-adjacent)** Set `lang="th"` (or dynamic per-post).
- [ ] **(gap 5)** Search scaling: prebuilt embed-time inverted index when
  justified; naive path stays correct for small N. No premature `tantivy`/
  `fuzzy` dependency.
- [ ] **(gap 4)** highlight.js: add SRI `integrity`+`crossorigin`, or **vendor
  as WASM** → zero external runtime requests.
- [ ] First-paint budget: document TTI on a throttled link; WASM gzip ceiling
  (e.g. `< 300KB`) verified in CI, not claimed.
- [ ] a11y audit: focus rings, `prefers-reduced-motion`, skip-to-content.

**Acceptance:** one parser; `lang` correct; WASM under budget in CI; zero
runtime network requests; a11y lint clean.

### Phase 9 - CI hardening & supply chain (open, closes gap 1)

- [ ] **(gap 1)** Required status checks before merge to `main`: `fmt --check`,
  `clippy -D warnings`, `cargo test --workspace`, `trunk build --release`.
- [ ] `cargo audit` + `cargo deny` (license + advisory) as CI steps.
- [ ] Branch protection: strict required checks, no force-push, no deletion.
- [ ] Preview deploys from PRs to a non-prod environment.
- [ ] Pin the Rust toolchain via `rust-toolchain.toml` so CI == local.
- [ ] Document that the app has **zero hand-written `unsafe`** (only
  `wasm-bindgen`/`js-sys` FFI); keep it that way.

**Acceptance:** a lint/test regression cannot merge; audit/deny green; branch
protection enforced; preview URL per PR.

---

## Release

### Phase 10 - First public v1.0.0 (open, closes gap 7)

- [ ] `v1.0.0` tag + documented release process.
- [ ] `robots.txt` + `sitemap.xml` generated at build time (closes Phase 5).
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
- [ ] **SSG option** - keep CSR as default (the project's whole point), but
  offer an optional build-time HTML snapshot for SEO/no-JS fallback, from the
  same `build.rs` content pass.
- [ ] **Offline / PWA** - service worker caching WASM + assets; natural fit for
  a zero-server static site (enabled by the Phase 8 WASM-vendored hljs).
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
           lang, search, hljs)     │
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
