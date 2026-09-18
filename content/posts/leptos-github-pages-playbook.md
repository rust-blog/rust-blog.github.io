---
title: "Deploy เว็บ Leptos ขึ้น GitHub Pages"
date: "2026-09-18"
description: "คู่มือฉบับ Playbook สำหรับการ deploy เว็บ Leptos (CSR + Trunk + Wasm) ขึ้น GitHub Pages"
tags: [rust, leptos, wasm]
author: "suradet-ps"
---

บทความนี้เป็น **Playbook** สำหรับกลับมาอ่านเมื่อ CI พังหรือต้องตั้งค่า Deploy ใหม่ โดยอ้างอิงจาก `.github/workflows/deploy.yml` ของ rust-blog นี้เอง (Leptos 0.8 CSR + Trunk + Wasm) ใช้ได้กับโปรเจกต์ Leptos CSR ที่ build ด้วย Trunk แต่ใช้กับ Leptos SSR ไม่ได้ เพราะ GitHub Pages ไม่มี server runtime

## 1. เกิดอะไรขึ้นเมื่อ push

push เข้า `main` แล้ว workflow จะทำงานเป็นสอง Job ต่อกัน โดยมี Artifact ของ `dist/` เป็นตัวส่งต่อ

Build Job (`ubuntu-latest`) มีลำดับดังนี้

1. **Checkout** - ดึงโค้ด พร้อมตั้ง `persist-credentials: false`
2. **ติดตั้ง Rust** - ตาม `rust-toolchain.toml` รวม target `wasm32-unknown-unknown`
3. **Cache cargo** - ลดเวลาคอมไพล์ซ้ำ
4. **ด่านคุณภาพ** - `fmt` → `clippy` (ยิง target wasm32) → `test`
5. **ด่าน Supply Chain** - `cargo-audit` → `cargo-deny`
6. **`trunk build --release --public-url /`** - ได้โฟลเดอร์ `dist/`
7. **post_build** - สร้างหน้า Static รายบทความพร้อม OG tags
8. **SPA Fallback** - คัดลอก `index.html` เป็น `404.html`
9. **Upload Artifact** - ส่ง `dist/` ต่อให้ Deploy Job

Deploy Job เริ่มเมื่อ Build สำเร็จเท่านั้น (`needs: build`) ถือสิทธิ์ `pages: write` + `id-token: write` เฉพาะตัวเอง ไม่ Checkout และไม่รันโค้ดโปรเจกต์เลย เรียกแค่ `deploy-pages` แล้วเว็บขึ้นที่ `https://rust-blog.github.io`

หลักที่ทั้งไฟล์ยึดถือคือ โค้ดที่ควบคุมไม่หมดอยู่ใน Job ที่อ่านได้อย่างเดียว, ใช้ Artifact แทน branch `gh-pages` และไม่มี Secret เลยเพราะใช้ OIDC ขอ token อายุสั้น

## 2. ตั้งค่าครั้งเดียว

- **Pages Source** ต้องเป็น **GitHub Actions** (Settings → Pages) ไม่งั้น Artifact อัปโหลดได้แต่ Deploy ไม่ทำงาน
- **`--public-url`** ต้องตรงกับประเภท Repo: User/Org Site (`<user>.github.io`) ใช้ `/`, Project Site ใช้ `/<repo>/` ถ้าผิด เบราว์เซอร์จะหา `.wasm` กับ CSS ไม่เจอ ได้หน้าเปล่า
- **`rust-toolchain.toml`** ต้องมี target `wasm32-unknown-unknown` และ component `rustfmt`, `clippy`
- **Commit `Cargo.lock`** และ ignore โฟลเดอร์ `dist/`

ก่อน push ควรเปิด Branch Protection ให้ Job `build` เป็น Required Check และถ้าเพิ่มหรือสลับ Step ให้ลำดับเป็น `trunk build` → `post_build` → `cp 404.html` → upload เสมอ

## 3. จุดที่ควรรู้ใน deploy.yml

### `on` และ `permissions`

```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
permissions:
  contents: read
```

`workflow_dispatch` ไว้สั่ง Deploy ใหม่หลังแก้ค่ารอบนอก โดยไม่ต้องสร้าง commit เปล่า ส่วน `permissions: contents: read` ล็อก Token เริ่มต้นให้อ่านอย่างเดียว แล้ว Deploy Job ค่อยยกระดับเอง ถ้าลืมเปิดสิทธิ์ให้ Step ใหม่ Job จะล้มเหลวทันที ซึ่งรู้ตัวเร็วกว่าเปิดกว้างเผื่อไว้

ข้อควรรู้คือไฟล์นี้ไม่มี trigger แบบ PR ด่านคุณภาพจึงทำงานหลัง Merge เข้า `main` แล้ว ถ้าทีมโตขึ้นควรมี Workflow สำหรับ PR แยก

### `concurrency` จุดที่พลาดง่ายที่สุด

```yaml
concurrency:
  group: pages
  cancel-in-progress: true
```

กลุ่ม `pages` ทำให้ Deploy ทีละรอบ กัน Run เก่าแซง Run ใหม่ แต่ `cancel-in-progress: true` มีราคาที่ต้องรู้: Starter Workflow ทางการใช้ `false` เพราะ Pages มี deploy ค้างได้ทีละหนึ่งต่อ ref ถ้า Run A กำลัง Deploy แล้วถูกยกเลิกกลางคัน สถานะอาจค้างและ Run B เริ่มไม่ได้ (actions/deploy-pages#118) repo นี้เลือก `true` เพื่อให้เว็บสดเสมอ ถ้าเจอ error ทำนอง "cannot start a new deployment while one is in progress" ให้เปลี่ยนเป็น `false` หรือ Re-run

### Checkout, Toolchain, Cache

```yaml
- name: Checkout
  uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
  with:
    persist-credentials: false
```

Action ทุกตัว Pin ด้วย Commit SHA ไม่ใช้ Tag เพราะ Tag ถูกย้ายให้ชี้โค้ดอันตรายได้ ราคาที่จ่ายคือต้องมี Renovate หรือ Dependabot คอยอัปเดต SHA ให้ ส่วน `persist-credentials: false` ตัดการฝัง Token ลง Git Config ซึ่ง Build Job ไม่ต้องใช้อยู่แล้ว

Rust Toolchain ล็อกใน `rust-toolchain.toml` ไม่เขียน `stable` ใน YAML ทำให้ Local กับ CI เหมือนกันเป๊ะ อาการ "เครื่องผมผ่าน แต่ CI พัง" จึงหายไป และ Lint ใหม่จะไม่ทำให้ CI แดงข้ามคืน การอัปเกรด Rust จึงกลายเป็นงานที่ต้องตั้งใจทำ

```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@f8be11a05b1d4f3fcebe6410cc16743212b999b0 # 1.98.0
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.98.0"
targets = ["wasm32-unknown-unknown"]
components = ["rustfmt", "clippy"]
```

`Swatinem/rust-cache` ลดเวลาคอมไพล์ซ้ำ ถ้า Cache เสียจนเกิดอาการ Local ผ่านแต่ CI ไม่ผ่าน ให้ลบ Cache ในหน้า Actions หรือ Bump Key ใหม่

```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2.9.2
```

### ด่านคุณภาพและ Supply Chain

```yaml
- name: Check formatting
  run: cargo fmt --all -- --check
- name: Clippy
  run: cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
- name: Test
  run: cargo test --workspace
- name: Install cargo-audit + cargo-deny
  uses: taiki-e/install-action@742a3317eac7bd62f91cd888b4eead5e784ba833 # v2.87.1
  with:
    tool: cargo-audit, cargo-deny
- name: Audit dependencies (RustSec advisory DB)
  run: cargo audit
- name: Deny check (advisories, licenses, bans)
  run: cargo deny check
```

Clippy ยิง target wasm32 เพราะโค้ด Leptos ส่วนมากอยู่ใน `#[cfg(target_arch = "wasm32")]` ถ้าตรวจแค่ Host จะไม่เห็นโค้ดที่จะ ship จริง ส่วน `cargo audit` และ `cargo deny` ตรวจ Advisory, License และ Source เมื่อมีช่องโหว่ใหม่ CI อาจแดงทั้งที่ไม่ได้แก้โค้ด ทางแก้คือใส่ `ignore` พร้อมเหตุผลใน `deny.toml` และ `.cargo/audit.toml` โดยสองไฟล์นี้ต้องซิงก์กันเสมอ

### Trunk build และ 404

```yaml
- name: Install Trunk
  uses: jetli/trunk-action@1346cc09eace4beb84e403e199a471346d4684c9 # v0.5.1
  with:
    version: "v0.21.14"
- name: Build (release)
  run: trunk build --release --public-url /
- name: Generate static per-post pages
  run: cargo run --quiet --bin post_build
- name: Prepare SPA fallback (404.html)
  run: cp dist/index.html dist/404.html
```

Trunk ปักเวอร์ชันไว้ (`v0.21.14`) เพราะพฤติกรรม build และ `wasm-opt` เปลี่ยนได้ข้าม minor และใน `Trunk.toml` มี pre_build hook `cargo build -q` ที่ห้ามลบ: ฟีเจอร์ `copy-file` ทำงานคู่ขนานกับ Rust pipeline บน clean checkout ไฟล์ `rss.xml`, `sitemap.xml`, `robots.txt` จึงยังไม่มี hook นี้บังคับให้ `build.rs` สร้างเสร็จก่อน ไม่งั้น build พังทันที

```toml
[[hooks]]
stage = "pre_build"
command = "cargo"
command_arguments = ["build", "-q"]
```

`post_build` ต้องรันหลัง `trunk build` เสมอ ถ้าสลับลำดับจะ panic มันสร้าง `dist/post/<slug>/index.html` เป็นไฟล์จริงพร้อม OG tags รายบทความ เพราะ GitHub Pages เสิร์ฟแต่ไฟล์ที่มีอยู่ และ `404.html` ถูกตอบด้วย HTTP 404 จริง เบราว์เซอร์รอดเพราะ SPA boot ขึ้นมา แต่ crawler อย่าง Facebook ไม่รัน JavaScript แชร์ลิงก์แล้วการ์ดพรีวิวจะหาย

`404.html` เองเป็น fallback ให้ client-side routing ทำงานเวลากด Refresh ที่ deep link แต่หน้าที่ได้ 404 จะไม่ถูก index (หน้าอย่าง `/about` จึงยังไม่ได้ prerender)

### Upload Artifact และ Deploy Job

Artifact ต้องมี `index.html` ที่ root และขนาดรวมไม่เกิน 1 GB ตามข้อจำกัดของ Pages

```yaml
- name: Upload artifact
  uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0
  with:
    path: dist
```

Deploy Job (`pages: write`, `id-token: write`, `environment: github-pages`) ไม่แตะโค้ดโปรเจกต์เลย ความเสียหายจาก dependency จึงจบที่ Build Job `id-token` ทำให้ยืนยันตัวตนผ่าน OIDC ไม่ต้องมี PAT ให้ rotate ส่วน `environment` ทำให้เห็น URL ของ deployment และตั้ง approval gate ได้

```yaml
deploy:
  needs: build
  runs-on: ubuntu-latest
  permissions:
    pages: write
    id-token: write
  environment:
    name: github-pages
    url: ${{ steps.deployment.outputs.page_url }}
  steps:
    - name: Deploy to GitHub Pages
      id: deployment
      uses: actions/deploy-pages@cd2ce8fcbc39b97be8ca5fce6e763baed58fa128 # v5.0.0
```

## 4. ความเสี่ยงที่ต้องรู้

- **Action ถูกยึด** - Pin SHA ทุกตัว และมี Bot คอยอัปเดตให้
- **Token รั่วจาก build script** - `contents: read` + `persist-credentials: false` + แยก Deploy Job
- **`--public-url` ผิด** - หน้าเปล่า ต้องเลือกตามชนิด Repo
- **Deep link 404** - `404.html` + prerender บทความ
- **Deploy ชนกัน** - `concurrency: pages` แต่ระวัง deployment ค้างจาก `cancel-in-progress: true`
- **Toolchain หรือ Lint เปลี่ยน** - Pin `rust-toolchain.toml` อย่าถอด
- **Advisory ใหม่** - ใส่ `ignore` พร้อมเหตุผล และกลับมาทบทวน
- **เพดาน Pages** - ไซต์ไม่เกิน 1 GB, แบนด์วิดท์ 100 GB/เดือน, deploy timeout 10 นาที (โควตา 10 builds/ชม. ไม่ใช้กับ Actions workflow)
- **ไม่มี Rollback ในตัว** - `git revert` แล้ว push ใหม่ ระหว่างนั้น Pages แสดงเวอร์ชันเดิมอยู่

## 5. อาการที่เจอบ่อย

- **หน้าเว็บขาว หรือ 404 `.wasm`** - `--public-url` ไม่ตรงกับชนิด Repo
- **Refresh `/post/...` แล้ว 404** - ไม่มี `404.html` หรือ Deploy ยังไม่เสร็จ
- **แชร์ลิงก์แล้วการ์ด OG ไม่ขึ้น** - `post_build` ไม่ทำงาน หรือถูกรันก่อน `trunk build`
- **Deploy ล้มด้วย "deployment in progress"** - เปลี่ยน `cancel-in-progress: false` แล้ว Re-run
- **Permission Denied ตอน Deploy** - Pages Source ยังไม่ใช่ Actions หรือ `permissions` ไม่ครบ
- **Clippy แดงทั้งที่ไม่ได้แก้โค้ด** - เคยถอด Pin toolchain ออก
- **Local ผ่าน แต่ CI ไม่ผ่าน** - Cache เสีย หรือเวอร์ชัน rustc/trunk ต่างกัน

## 6. โครง workflow ขั้นต่ำสำหรับโปรเจกต์อื่น

ถ้าเริ่มโปรเจกต์ใหม่ เอาโครงนี้ไปวางที่ `.github/workflows/deploy.yml` แล้วแทน `<SHA>` ด้วย Commit SHA จริงของแต่ละ Action (ดูเวอร์ชันล่าสุดได้จากหน้า Releases ของ Action นั้น)

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: pages
  cancel-in-progress: false   # ให้ deploy ที่กำลังทำอยู่ทำงานจนจบก่อน

jobs:
  build:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@<SHA>
        with:
          persist-credentials: false

      - uses: dtolnay/rust-toolchain@<SHA>
        with:
          targets: wasm32-unknown-unknown

      - uses: Swatinem/rust-cache@<SHA>

      - run: cargo fmt --all -- --check
      - run: cargo clippy --target wasm32-unknown-unknown -- -D warnings
      - run: cargo test --workspace

      - uses: jetli/trunk-action@<SHA>
        with:
          version: "v0.21.14"

      # หากเป็น Project Site ให้เปลี่ยน / เป็น /<repo>/
      - run: trunk build --release --public-url /

      - run: cp dist/index.html dist/404.html

      - uses: actions/upload-pages-artifact@<SHA>
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-24.04
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/deploy-pages@<SHA>
        id: deployment
```

## 7. สรุป

`deploy.yml` ตอบโจทย์สามเรื่อง: **ความแน่นอน** (Pin ทุกอย่างที่เลื่อนได้), **ความปลอดภัย** (แยก Job ตามสิทธิ์ ไม่มี Secret) และ **ข้อจำกัดของ GitHub Pages** (static hosting, crawler ไม่รัน JS, base path ต้องตรง)

ถ้า CI พัง ให้เริ่มจาก `concurrency`, `public-url`, toolchain และ artifact ก่อน สี่จุดนี้ครอบคลุมปัญหาที่พบบ่อยที่สุด ไฟล์ต้นแบบเต็มดูได้ที่ `.github/workflows/deploy.yml` ของ repo นี้
