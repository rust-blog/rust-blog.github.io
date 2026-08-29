# rust-blog

```
██████╗ ██╗   ██╗ ██████╗████████╗██████╗ ██╗      ██████╗  ██████╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔══██╗██║     ██╔═══██╗██╔════╝
██████╔╝██║   ██║███████╗   ██║   ██████╔╝██║     ██║   ██║██║  ███╗
██╔══██╗██║   ██║╚════██║   ██║   ██╔══██╗██║     ██║   ██║██║   ██║
██║  ██║╚██████╔╝██████╔╝   ██║   ██████╔╝███████╗╚██████╔╝╚██████╔╝
╚═╝  ╚═╝ ╚═════╝ ╚═════╝   ╚═╝╚═════╝╚══════╝ ╚═════╝  ╚═════╝
```

---

## ◆ PULSE

A blog engine should be as quiet as a good essay. rust-blog is a
production-grade blog written in Rust and compiled to WebAssembly -
Leptos 0.8 CSR, Trunk-built, GitHub Pages-served - where a post is
nothing but a markdown file dropped into `content/posts/`. It appears,
indexed, searchable, tag-filtered, and syndicated through an
RSS feed generated at build time. No database, no server, no
JavaScript authored by hand - and a site that speaks primarily Thai,
for the readers it serves.

| P0-P2 ▣ | P3-P4 ▣ | P5 ▢ | P6-P10 ☐ |
|---|---|---|---|

*Scaffold, content model, shell, RSS, and the authoring experience are
sealed; accessibility is half-forged; the asset pipeline, safety
suite, hardening, and v1.0 stand open.*

> Built with Rust 2024 + Leptos 0.8, rendered by `pulldown-cmark`,
> deployed by one workflow to GitHub Pages.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One target, one tool, one command.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk
⟫ trunk serve
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

The release artifact: `⟫ trunk build --release --public-url /`

<details>
<summary>Prerequisites</summary>

- [Rust](https://rustup.rs/) (edition 2024)
- The `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) - installed above

</details>

---

## ◆ ANATOMY

One folder, one binary, a quiet set of honest services.

- **Publishes** - content is embedded at compile time via
  `include_dir!`, never fetched at runtime: drop a markdown file into
  `content/posts/` and the build collects it, recursively.
- **Renders** - `pulldown-cmark` with `Options::all()` turns the post
  into HTML; `highlight.js` lights the code; the hand-crafted CSS
  design system (light and dark, no framework) carries the look.
- **Searches** - the home page filters in memory by substring and tag
  chips - no index to maintain, no backend to ask.
- **Syndicates** - `build.rs` generates `rss.xml` at build time and
  the post-build hook copies it into `dist/` - the feed is born with
  the site.
- **Ships** - pushing to `main` triggers the deploy workflow: install,
  build, copy `index.html` to `404.html` for SPA routing, publish
  `dist/` to GitHub Pages.

---

## ◆ RITUALS

**The core ceremony** - publishing a post:

1. Write the markdown with its frontmatter: title, date, description,
   tags - `draft: true` hides it until it is ready.
2. Drop the file into `content/posts/`. That is the whole deployment.
3. The build picks it up: home page, filtering, tags, and the RSS
   feed update together.
4. Push to `main`; the workflow builds and ships the site to GitHub
   Pages.

**The ceremony of the markdown file** - a post is a file, not a form:
no admin panel, no database, no per-post code. The content model is
the folder structure, and the folder is the CMS.

**The ceremony of the compiled site** - everything the reader sees
was embedded when the binary was built: posts, search terms, RSS. A
static file cannot be taken down by a failing backend, because there
is no backend.

---

## ◆ ECHOES

**Where this artifact is heading**

```
P0-P2 ▸ scaffold, content model, shell, theme ───────────────────────── ▸ sealed
P3-P4 ▸ reproducible RSS, authoring experience ──────────────────────── ▸ sealed
P5    ▸ accessibility & SEO ─────────────────────────────────────────── ▸ forging
P6-P8 ▸ asset pipeline, safety suite, hardening ─────────────────────── ▸ open
P9-P10 ▸ CI hardening, v1.0.0 ────────────────────────────────────────── ▸ open
```

**Raising the artifact** - the honest path lives in `ROADMAP.md`;
content in `content/posts/`; the design system in `styles/main.css`.
Gates: `cargo fmt --check`, clippy with `-D warnings`, tests, and the
Trunk build. Open an issue first to discuss a change.

**Status** - CI checks every push and deploys to GitHub Pages
automatically. [Watch the workflow](.github/workflows).

---

```
  ─────────────────────────────────────────
   A blog that ships as a binary
   is a blog that cannot be taken down.
  ─────────────────────────────────────────
```

Distributed under the [MIT License](LICENSE).