---
title: "Deploy เว็บ Leptos ขึ้น GitHub Pages"
date: "2026-09-18"
description: "คู่มือฉบับ Playbook สำหรับการ deploy เว็บ Leptos (CSR + Trunk + Wasm) ขึ้น GitHub Pages"
tags: [rust, leptos, wasm]
author: "suradet-ps"
---

บทความนี้เป็น **Playbook** สำหรับเปิดอ่านย้อนหลังเมื่อ CI พังหรือต้องตั้งค่า Deploy ใหม่ อ้างอิงจาก `.github/workflows/deploy.yml` ของ rust-blog นี้เอง (Leptos 0.8 CSR + Trunk + Wasm) ใช้ได้กับโปรเจกต์ Leptos CSR ที่ build ด้วย Trunk แต่ใช้กับ Leptos SSR ไม่ได้ เพราะ GitHub Pages ไม่มี server runtime ในตัว

## 1. เกิดอะไรขึ้นเมื่อ push

เมื่อ push เข้า branch `main` workflow จะทำงานแบ่งเป็นสอง Job ต่อกัน โดยส่งต่อไฟล์ผ่าน Artifact ของโฟลเดอร์ `dist/`

Build Job (`ubuntu-latest`) มีลำดับการทำงานดังนี้

1. **Checkout** - ดึงโค้ดลงเครื่อง CI พร้อมตั้งค่า `persist-credentials: false`
2. **ติดตั้ง Rust** - อ่านค่าตาม `rust-toolchain.toml` และเพิ่ม target `wasm32-unknown-unknown`
3. **Cache cargo** - ช่วยลดเวลาในการคอมไพล์ซ้ำ
4. **ด่านตรวจสอบคุณภาพ** - รัน `fmt` → `clippy` (โฟกัส target wasm32) → `test`
5. **ด่านตรวจความปลอดภัย Supply Chain** - รัน `cargo-audit` → `cargo-deny`
6. **`trunk build --release --public-url /`** - ประมวลผลออกมาได้โฟลเดอร์ `dist/`
7. **post_build** - สร้างหน้า Static แยกรายบทความเพื่อรองรับ OG tags
8. **SPA Fallback** - คัดลอก `index.html` ไปเป็น `404.html`
9. **Upload Artifact** - ส่งโฟลเดอร์ `dist/` ไปให้ Deploy Job ถัดไป

Deploy Job จะเริ่มทำงานเมื่อ Build ผ่านเรียบร้อยแล้วเท่านั้น (`needs: build`) ถือสิทธิ์ `pages: write` + `id-token: write` ไว้กับตัวเอง ไม่มีการ Checkout โค้ดและไม่รันสคริปต์โปรเจกต์ซ้ำ เรียกเพียง `deploy-pages` เพื่อดันเว็บขึ้น `[https://rust-blog.github.io](https://rust-blog.github.io)`

หลักการสำคัญของไฟล์นี้ คือ ให้ Job ที่อ่านได้อย่างเดียวควบคุมโค้ดที่ประเมินความเสี่ยงได้ไม่หมด, ใช้ Artifact รับส่งแทน branch `gh-pages` และไม่ใช้ Secret ใดๆ เพราะยืนยันตัวตนด้วย OIDC เพื่อขอ token อายุสั้น

## 2. ตั้งค่าครั้งเดียว

* **Pages Source** ต้องเลือกเป็น **GitHub Actions** (Settings → Pages) ไม่งั้นแม้จะอัปโหลด Artifact ได้แต่ตัว Deploy จะไม่ทำงาน
* **`--public-url`** ต้องตั้งให้ตรงกับประเภท Repo: User/Org Site (`<user>.github.io`) ให้ใช้ `/`, ส่วน Project Site ให้ใช้ `/<repo>/` ถ้าตั้งผิด เบราว์เซอร์จะหาไฟล์ `.wasm` กับ CSS ไม่พบ จนได้หน้าเว็บว่างเปล่า
* **`rust-toolchain.toml`** ต้องระบุ target `wasm32-unknown-unknown` และ component `rustfmt`, `clippy` ไว้เสมอ
* **Commit `Cargo.lock`** เข้า Git และระบุ ignore โฟลเดอร์ `dist/` ใน `.gitignore`

ก่อน push ควรตั้งค่า Branch Protection ให้ Job `build` เป็น Required Check และหากมีการปรับเปลี่ยน Step ให้เรียงลำดับเป็น `trunk build` → `post_build` → `cp 404.html` → upload เสมอ

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

`workflow_dispatch` มีไว้สั่งรัน Deploy ใหม่เมื่อปรับค่าภายนอก โดยไม่ต้องสร้าง commit หลอก ส่วน `permissions: contents: read` เป็นการจำกัด Token เริ่มต้นให้อ่านได้อย่างเดียว แล้วค่อยให้ Deploy Job ยกระดับสิทธิ์ขึ้นเอง หากลืมเปิดสิทธิ์ให้ Step ใหม่ Job จะฟ้อง Error ทันที ซึ่งช่วยให้รู้ตัวและแก้ไขได้เร็วกว่าการเปิดสิทธิ์กว้างทิ้งไว้

ข้อควรรู้คือไฟล์นี้ยังไม่มี trigger สำหรับ PR ดังนั้นด่านคุณภาพจะทำงานหลัง Merge เข้า `main` แล้วเท่านั้น หากทีมใหญ่ขึ้นควรแยก Workflow สำหรับ PR ออกมาต่างหาก

### `concurrency` จุดที่พลาดง่ายที่สุด

```yaml
concurrency:
  group: pages
  cancel-in-progress: true
```

การตั้งกลุ่ม `pages` ช่วยจัดคิว Deploy ทีละรอบ ป้องกัน Run เก่าประมวลผลแซง Run ใหม่ แต่ `cancel-in-progress: true` มีจุดที่ต้องแลก: Starter Workflow ทางการมักใช้ `false` เพราะ Pages ยอมให้มี deployment ค้างได้ทีละหนึ่งต่อ ref หาก Run A กำลัง Deploy แล้วถูกสั่งยกเลิกกลางคัน สถานะอาจค้างจน Run B เริ่มไม่ได้ (actions/deploy-pages#118) repo นี้เลือก `true` เพื่อเน้นให้หน้าเว็บสดใหม่อยู่ตลอด หากเจอ error ทำนอง "cannot start a new deployment while one is in progress" ให้ปรับกลับเป็น `false` หรือสั่ง Re-run ใหม่

### Checkout, Toolchain, Cache

```yaml
- name: Checkout
  uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
  with:
    persist-credentials: false
```

Action ทุกตัวได้รับการ Pin ด้วย Commit SHA แทนการใช้ Tag เพราะ Tag อาจถูกย้ายให้ชี้ไปที่โค้ดอันตรายได้ สิ่งที่ต้องแลกคือต้องพึ่งพา Bot อย่าง Renovate หรือ Dependabot คอยอัปเดต SHA ให้ ส่วน `persist-credentials: false` ช่วยตัดการฝัง Token ลงใน Git Config ซึ่ง Build Job ไม่จำเป็นต้องใช้อยู่แล้ว

Rust Toolchain ถูกล็อกไว้ใน `rust-toolchain.toml` โดยไม่ระบุ `stable` ลงใน YAML เพื่อให้สภาพแวดล้อมบน Local กับ CI ตรงกันเป๊ะ อาการ "เครื่องผมผ่าน แต่ CI พัง" จึงไม่เกิดขึ้น และ Lint เวอร์ชันใหม่จะไม่ทำให้ CI แดงแบบไม่ทันตั้งตัว ทำให้การอัปเกรด Rust เป็นกระบวนการที่ต้องตั้งใจทำอย่างเป็นระบบ

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

`Swatinem/rust-cache` ช่วยย่นเวลาคอมไพล์ซ้ำ หาก Cache รวนจนเกิดอาการ Local ผ่านแต่ CI ไม่ผ่าน สามารถลบ Cache ทิ้งในหน้า Actions หรือสั่ง Bump Key ใหม่ได้

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

Clippy ต้องเจาะจงไปที่ target wasm32 เพราะโค้ด Leptos ส่วนใหญ่ซ่อนอยู่ใน `#[cfg(target_arch = "wasm32")]` หากตรวจแค่ Host จะมองไม่เห็นโค้ดที่ใช้งานจริง ส่วน `cargo audit` และ `cargo deny` มีไว้ตรวจ Advisory, License และ Source ซึ่งหากมีแจ้งเตือนช่องโหว่ใหม่ CI อาจติดสีแดงได้แม้ไม่ได้แก้โค้ด ทางแก้คือระบุ `ignore` พร้อมเหตุผลไว้ใน `deny.toml` และ `.cargo/audit.toml` โดยทั้งสองไฟล์นี้ต้องอัปเดตให้ตรงกันเสมอ

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

Trunk ต้องล็อกเวอร์ชันไว้ (`v0.21.14`) เพราะพฤติกรรมการ build และ `wasm-opt` อาจเปลี่ยนได้ในระดับ minor และใน `Trunk.toml` จะมี pre_build hook `cargo build -q` ที่ห้ามลบเด็ดขาด: เนื่องจากฟีเจอร์ `copy-file` ทำงานขนานไปกับ Rust pipeline บน clean checkout หากไม่มี hook นี้บังคับให้ `build.rs` ทำงานเสร็จก่อน ไฟล์อย่าง `rss.xml`, `sitemap.xml`, `robots.txt` จะยังไม่ถูกสร้าง และส่งผลให้ build พังทันที

```toml
[[hooks]]
stage = "pre_build"
command = "cargo"
command_arguments = ["build", "-q"]
```

`post_build` ต้องรันตามหลัง `trunk build` เสมอ มิฉะนั้นจะเกิด panic โดยสคริปต์นี้จะสร้าง `dist/post/<slug>/index.html` ออกมาเป็นไฟล์จริงพร้อม OG tags รายบทความ เพื่อแก้ปัญหาที่ GitHub Pages เสิร์ฟเฉพาะไฟล์ที่มีอยู่จริง และตอบกลับหน้า `404.html` ด้วย HTTP 404 แม้ผู้ใช้จะใช้งานได้ปกติเพราะ SPA บูตขึ้นมาแทน แต่ crawler เช่น Facebook จะไม่รัน JavaScript ทำให้พรีวิวการ์ดแชร์ลิงก์สูญหาย

ส่วน `404.html` จะทำหน้าที่เป็น fallback ให้ระบบ client-side routing เมื่อผู้ใช้สั่ง Refresh บน deep link แต่หน้าลักษณะนี้ (เช่น `/about`) จะไม่ถูกนำไปจัด indexing บน Search Engine เนื่องจากไม่ได้ทำ prerender ไว้

### Upload Artifact และ Deploy Job

Artifact ที่จะอัปโหลดต้องมีไฟล์ `index.html` อยู่ที่ root และขนาดรวมทั้งหมดต้องไม่เกิน 1 GB ตามข้อกำหนดของ Pages

```yaml
- name: Upload artifact
  uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0
  with:
    path: dist
```

Deploy Job (`pages: write`, `id-token: write`, `environment: github-pages`) จะไม่แตะต้องโค้ดในโปรเจกต์เลย ความเสี่ยงจาก dependency จึงจำกัดอยู่แค่ใน Build Job การใช้ `id-token` ช่วยให้ยืนยันตัวตนผ่าน OIDC ได้โดยไม่ต้องคอยสลับหรือ rotate PAT ส่วน `environment` ช่วยให้ติดตาม URL ของ deployment ได้ชัดเจนและรองรับการตั้งค่า approval gate

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

* **Action ถูกเข้าควบคุม** - ป้องกันด้วยการ Pin SHA ทุกตัว และใช้ Bot คอยช่วยอัปเดต
* **Token รั่วไหลจาก build script** - จำกัดสิทธิ์ด้วย `contents: read` + `persist-credentials: false` และแยก Deploy Job ออกมา
* **กำหนด `--public-url` ไม่ถูกต้อง** - หน้าเว็บจะขาวเปล่า ต้องเลือกให้ตรงตามประเภทของ Repo
* **เข้าผ่าน Deep link แล้วเจอ 404** - แก้ด้วยการทำ `404.html` + prerender หน้าบทความ
* **กระบวนการ Deploy ชนกัน** - ควบคุมด้วย `concurrency: pages` แต่ต้องระวังปัญหาค้างจาก `cancel-in-progress: true`
* **Toolchain หรือ Lint มีการเปลี่ยนแปลง** - ล็อกเวอร์ชันใน `rust-toolchain.toml` ห้ามปลดออก
* **ตรวจพบ Advisory ตัวใหม่** - กำหนดค่า `ignore` พร้อมระบุเหตุผล แล้วค่อยกลับมาแก้ไขทีหลัง
* **ข้อจำกัดของ Pages** - ขนาดไซต์ไม่เกิน 1 GB, แบนด์วิดท์ไม่เกิน 100 GB/เดือน, deploy timeout 10 นาที (โควตา 10 builds/ชม. จะไม่นับรวมกับการรัน Actions workflow)
* **ไม่มีระบบ Rollback ในตัว** - ต้องใช้ `git revert` แล้ว push ใหม่ โดยระหว่างนั้น Pages จะยังคงแสดงผลเวอร์ชันเดิมอยู่

## 5. อาการที่เจอบ่อย

* **หน้าเว็บขาว หรือฟ้อง 404 บนไฟล์ `.wasm`** - เกิดจาก `--public-url` ไม่ตรงกับประเภท Repo
* **สั่ง Refresh ที่หน้า `/post/...` แล้วขึ้น 404** - ขาดไฟล์ `404.html` หรือกระบวนการ Deploy ยังไม่เสร็จสิ้น
* **แชร์ลิงก์แล้วภาพการ์ด OG ไม่ขึ้น** - `post_build` ไม่ทำงาน หรือถูกสั่งรันก่อน `trunk build`
* **Deploy ล้มเหลวพร้อมข้อความ "deployment in progress"** - ปรับเป็น `cancel-in-progress: false` แล้วสั่ง Re-run ใหม่
* **ติดปัญหา Permission Denied ตอน Deploy** - ตั้งค่า Pages Source ยังไม่เป็น Actions หรือระบุ `permissions` ไม่ครบถ้วน
* **Clippy แจ้งเตือนสีแดงทั้งที่ไม่ได้แก้ไขโค้ด** - เคยถอดการ Pin toolchain ออก
* **ทดสอบบน Local ผ่าน แต่บน CI ไม่ผ่าน** - เกิดจาก Cache รวน หรือเวอร์ชันของ rustc/trunk ไม่ตรงกัน

## 6. โครง workflow ขั้นต่ำสำหรับโปรเจกต์อื่น

หากต้องการเริ่มโปรเจกต์ใหม่ สามารถนำโครงสร้างนี้ไปวางไว้ที่ `.github/workflows/deploy.yml` แล้วเปลี่ยน `<SHA>` เป็น Commit SHA จริงของแต่ละ Action (ตรวจสอบเวอร์ชันล่าสุดได้จากหน้า Releases ของ Action นั้นๆ)

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
  cancel-in-progress: false   # ให้ deploy ที่กำลังทำอยู่ทำงานจนเสร็จสิ้นก่อน

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

`deploy.yml` ออกแบบมาเพื่อตอบโจทย์สามเรื่องหลัก: **ความเสถียรแน่นอน** (Pin ทุกอย่างที่อาจเปลี่ยนแปลงได้), **ความปลอดภัย** (แยก Job ตามระดับสิทธิ์ ไม่ใช้ Secret) และ **การก้าวข้ามข้อจำกัดของ GitHub Pages** ( static hosting, crawler ไม่รัน JS, base path ต้องตรง)

หากระบบ CI มีปัญหา แนะนำให้เริ่มไล่เช็กจาก `concurrency`, `public-url`, toolchain และ artifact ก่อน เพราะสี่จุดนี้ครอบคลุมสาเหตุส่วนใหญ่ที่พบได้บ่อยที่สุด โดยสามารถดูไฟล์ต้นแบบฉบับเต็มได้ที่ `.github/workflows/deploy.yml` ของ repository นี้
