# rust-blog 🦀

> A production-grade blog built with **Rust + Leptos 0.8 (CSR)**, compiled to **WebAssembly** and deployed on **GitHub Pages**.

[![Deploy to GitHub Pages](https://github.com/rust-blog/rust-blog.github.io/actions/workflows/deploy.yml/badge.svg)](https://github.com/rust-blog/rust-blog.github.io/actions/workflows/deploy.yml)
[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/framework-Leptos-orange)](https://leptos.dev/)

Live site: **https://rust-blog.github.io/**

## Features

- ⚡ **Client-Side Rendering** with Leptos 0.8 compiled to WebAssembly
- 📝 **Content-as-Markdown** — drop a file in `content/posts/` and it appears automatically
- 🔍 **Search & tag filtering** on the home page
- 🌗 **Light / Dark mode** with OS preference detection and persistence
- 🎨 Hand-crafted design system using a custom color palette (no CSS framework)
- 📜 **Syntax highlighting** via highlight.js
- 📰 **RSS feed** (`/rss.xml`) generated at build time
- 📱 Fully responsive, with SEO `<title>`/`<meta>` per page
- 🚀 Automated CI/CD to GitHub Pages

## Tech Stack

| Layer        | Choice                                |
| ------------ | ------------------------------------- |
| Language     | Rust (Edition 2024)                   |
| Framework    | [Leptos](https://leptos.dev/) 0.8     |
| Rendering    | Client-Side Rendering (CSR)           |
| Bundler      | [Trunk](https://trunkrs.dev/)         |
| Styling      | Custom CSS design system              |
| Markdown     | pulldown-cmark                        |
| Hosting      | GitHub Pages                          |

## Getting Started

### Prerequisites

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

### Local development

```sh
trunk serve
# → http://127.0.0.1:8080
```

### Production build

```sh
trunk build --release --public-url /
```

## Writing a new post

Create a Markdown file in `content/posts/`. Frontmatter is required:

```markdown
---
title: "ชื่อบทความ"
date: "2024-07-01"
description: "คำอธิบายสั้นๆ สำหรับตัวอย่างและ RSS"
tags: [rust, wasm]
author: "rust-blog"   # optional
draft: false            # set true to hide
slug: "my-post"         # optional, defaults to the filename
---

เนื้อหาบทความเขียนด้วย Markdown ปกติ...
```

That's it — the home page, filtering, and RSS feed update automatically.

## Project Structure

```text
rust-blog.github.io/
├── Cargo.toml          # Dependencies & release profile
├── Trunk.toml          # Trunk config + post-build hook (RSS copy)
├── build.rs            # Generates rss.xml from content at build time
├── index.html          # App entry point
├── styles/
│   └── main.css        # Design system
├── content/
│   └── posts/          # ← add your .md posts here
├── src/
│   ├── main.rs         # App root, router, providers
│   ├── content.rs      # Post model + embedded content loader
│   ├── markdown.rs     # Markdown → HTML rendering
│   ├── util.rs         # Theme, highlighting, date formatting, base path
│   ├── components.rs   # Nav, Footer, ThemeToggle, PostCard, TagChip
│   └── pages/          # Home, Post, About, NotFound
└── .github/workflows/  # CI/CD to GitHub Pages
```

## Deployment

Pushes to `main` trigger `.github/workflows/deploy.yml`, which:

1. Installs Rust + the `wasm32-unknown-unknown` target and Trunk
2. Runs `build.rs` to generate the RSS feed
3. Builds the app with `trunk build --release --public-url /`
4. Copies `dist/index.html` → `dist/404.html` for SPA routing
5. Publishes the `dist/` artifact to GitHub Pages

## License

Distributed under the MIT License.
