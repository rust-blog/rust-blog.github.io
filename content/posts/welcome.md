---
title: "Welcome to rust-blog"
date: "2026-08-29"
description: "A blog written in Rust and Leptos, running in your browser as WebAssembly"
tags: [announcement, rust, leptos]
author: "suradet-ps"
---

Hello, and welcome to the first post on **rust-blog**. This blog isn't just about the Rust language - it's built with, and runs on, Rust itself.

## Powered by Rust 100%

What sets this blog apart from most websites is that the frontend relies on almost no JavaScript at all. Everything runs on:

- **Rust** (Edition 2024)
- **Leptos 0.8** in Client-Side Rendering (CSR) mode
- **Trunk** as the build tool
- Compiled to **WebAssembly (Wasm)** and running directly in the browser

Even the syntax highlighting in posts is processed by `syntect` at render time - no external CDN libraries, and not a single JavaScript file embedded anywhere on this site.

## Adding a new post is easy

The system is designed so content is easy to manage: just write a plain Markdown file.

```bash
# Create a new post in content/posts/
echo '---\ntitle: "A new post"\ndate: "2026-08-30"\ntags: [rust]\n---\n\nPost body...' > content/posts/my-post.md
```

That's it - the blog picks the post up automatically and shows it on the site, no other code changes needed.

## What rust-blog has to offer

- **Dark Mode / Light Mode** - switch themes to your liking, with your choice remembered
- **Tag Filtering** - search and filter posts by the tags you care about
- **Syntax Highlighting** - crisp, beautiful code blocks powered by `syntect`
- **RSS Feed** - `/rss.xml` is generated automatically at build time
- **Responsive & Editorial Design** - a clean, easy-on-the-eyes layout that works on every screen size

This blog started from a simple question: *"If a Rust developer wanted to build a personal blog and write the whole thing in Rust, from start to finish - what would it look like?"*

This website is the answer.

See you in the next post!