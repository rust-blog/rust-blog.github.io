---
title: "Dependency ซ้ำซ้อนใน Cargo ทำไม crate เดียวกันถึงมีหลายเวอร์ชันพร้อมกัน"
date: "2026-10-12"
description: "เจาะลึกกลไก SemVer และ Dependency Resolution ของ Cargo ที่ทำให้ crate เดียวกันถูกดึงมาคอมไพล์หลายเวอร์ชัน พร้อมวิธีตรวจเช็คและแนวทางแก้ไข"
tags: [rust, wasm, security]
author: "suradet-ps"
---

ตอนที่รัน `cargo build` หลายคนน่าจะเคยเห็น log ที่คอมไพล์ crate ชื่อเดียวกันซ้ำสองรอบ หรือบางครั้งอาจเจอ compile error แปลก ๆ อย่าง `expected itoa::Buffer, found a different itoa::Buffer` ทั้งที่ดูยังไงชื่อ type ก็เหมือนกัน

ผมเจอปัญหานี้อีกรอบตอนรัน `cargo deny check` ในโปรเจกต์ [rust-blog](/post/cargo-deny) แล้วเจอ `warning[duplicate]` แจ้งเตือนว่า `syn`, `thiserror`, `convert_case` และ `wit-bindgen` มีเวอร์ชันซ้ำกันใน dependency graph

บทความนี้จะพาไปดูว่า Cargo มีกลไกการเลือกเวอร์ชันอย่างไร ทำไมบางกรณีถึงยุบรวมเวอร์ชันไม่ได้ ผลกระทบต่อ binary และ compile time มีอะไรบ้าง รวมถึงแนวทางจัดการเมื่อเจอ dependency ซ้ำซ้อน

## Cargo Resolve Dependency อย่างไร

หัวใจสำคัญอยู่ที่ **SemVer** และวิธีที่ Cargo ตีความ version requirement ใน `Cargo.toml`

เวลาเราระบุเวอร์ชัน เช่น `itoa = "1"` Cargo จะมองเป็น caret requirement (`^`) เสมอ

```toml
[dependencies]
itoa = "1"       # ^1.0.0   -> >=1.0.0, <2.0.0
itoa = "1.0.10"  # ^1.0.10  -> >=1.0.10, <2.0.0
serde = "0.9"    # ^0.9     -> >=0.9.0, <0.10.0
```

จุดที่คนมักลืมคือกฎของเวอร์ชัน `0.x` ตาม SemVer ตัวเลข minor ของ `0.x` จะถือเป็น breaking change ทันที ดังนั้น `^0.9` จึงไม่นับรวม `0.10.0`

| ข้อกำหนด | ความหมาย | ตัวอย่างเวอร์ชันที่ resolve ได้ |
|---|---|---|
| `"1.2.3"` | `>=1.2.3, <2.0.0` | 1.9.0 |
| `"0.2.3"` | `>=0.2.3, <0.3.0` | 0.2.11 |
| `"0.0.3"` | `>=0.0.3, <0.0.4` | 0.0.3 |
| `"=1.2.3"` | 1.2.3 เท่านั้น | 1.2.3 |

ถ้า dependency สองตัวเรียกใช้ crate เดียวกัน และเวอร์ชันที่ขอยังอยู่ในช่วง SemVer ที่เข้ากันได้ Cargo จะ **ยุบรวม (unify) ให้เหลือเวอร์ชันเดียวเสมอ** 

แต่ถ้าข้าม major version (หรือข้าม minor ในเคส `0.x`) Cargo จะไม่ยุบรวม เพราะอาจมี breaking change มันจึงแยกคอมไพล์เป็นสองเวอร์ชันขนานกันไป

```toml
# Package A
[dependencies]
rand = "0.7" # >=0.7.0, <0.8.0

# Package B
[dependencies]
rand = "0.6" # >=0.6.0, <0.7.0
```

กรณีนี้ Cargo จะดึงทั้ง `rand` 0.6.x และ 0.7.x เข้ามาทั้งคู่

นอกจากนี้ ใน Rust edition 2024 เป็นต้นไป Cargo จะใช้ **resolver v3** เป็นค่าเริ่มต้น ซึ่งจะแยก feature ระหว่าง dependencies ปกติกับ build-dependencies/proc-macros ออกจากกัน และจะคำนึงถึง MSRV (Minimum Supported Rust Version) เพื่อไม่ให้ระบบเลือกเวอร์ชันที่เกินกว่า rustc ของเราจะคอมไพล์ได้

## ทำไมชื่อ Type เดียวกันถึงเข้ากันไม่ได้

ลองสร้าง workspace จำลองขึ้นมา 4 crate เพื่อดูพฤติกรรมนี้

```text
dup-demo/
├── app/     <- binary ที่เรียกทั้งสามตัว
├── old-fmt/ <- ใช้ itoa "0.4"
├── new-fmt/ <- ใช้ itoa "1"
└── mid/     <- ใช้ itoa "1.0.10"
```

กำหนดให้ `old-fmt` และ `new-fmt` expose type ของ `itoa` ออกมาใน public API

```rust
// old-fmt/src/lib.rs
pub fn buffer() -> itoa::Buffer {
    itoa::Buffer::new()
}
```

```rust
// new-fmt/src/lib.rs
pub fn format(buf: &mut itoa::Buffer, value: i32) -> &str {
    buf.format(value)
}
```

เมื่อรัน `cargo tree -d` จะเห็น dependency ซ้ำกันดังนี้

```text
itoa v0.4.8
└── old-fmt v0.1.0 (dup-demo/old-fmt)

itoa v1.0.18
├── mid v0.1.0 (dup-demo/mid)
└── new-fmt v0.1.0 (dup-demo/new-fmt)
```

`mid` (`^1.0.10`) กับ `new-fmt` (`^1`) ใช้เวอร์ชัน `1.0.18` ร่วมกันได้เพราะ SemVer เข้ากันได้ แต่ `old-fmt` ขอ `0.4` จึงต้องแยกออกมาอีกตัวหนึ่ง

ถ้าใน `app` เราพยายามส่ง object ข้ามกัน

```rust
fn main() {
    let mut buf = old_fmt::buffer();
    let text = new_fmt::format(&mut buf, 42);
    println!("{text}");
}
```

จะเจอ compile error ทันที

```text
error[E0308]: mismatched types
 --> app/src/main.rs:3:32
  |
3 |     let text = new_fmt::format(&mut buf, 42);
  |                --------------- ^^^^^^^^ expected `itoa::Buffer`, found a different `itoa::Buffer`
  |
note: there are multiple different versions of crate `itoa` in the dependency graph
  --> itoa-1.0.18/src/lib.rs:72:1
   |
72 | pub struct Buffer {
   | ^^^^^^^^^^^^^^^^^ this is the expected type
   |
  ::: itoa-0.4.8/src/lib.rs:110:1
   |
110 | pub struct Buffer {
    | ----------------- this is the found type
    = help: you can use `cargo tree` to explore your dependency tree
```

ในมุมของ Rust type safety ตัว `itoa::Buffer` ของ 0.4.8 และ 1.0.18 ถือเป็น **คนละ type กันอย่างสิ้นเชิง** เพราะ Rust ระบุ identity ของ type ตาม crate และ version ไม่ได้ดูแค่ชื่อ struct ปัญหานี้มักแก้ที่โค้ดเราเองตรง ๆ ไม่ได้ เพราะเกิดจาก dependency สองตัวที่เราเรียกใช้ดันผูกกับ library คนละเวอร์ชัน

## ความแตกต่างในการแสดงผลของ `cargo tree -d`

คำสั่ง `cargo tree -d` (หรือ `--duplicates`) ไม่ได้แสดงแค่ crate ที่คนละเวอร์ชันกันเท่านั้น แต่มันยังแสดง

1. Crate เดียวกัน แต่ **คนละเวอร์ชัน** (duplicate ตาม `Cargo.lock`)
2. Crate เดียวกัน **เวอร์ชันเดียวกัน แต่ถูกคอมไพล์หลายรอบ** เพราะเปิด feature คนละชุด

แบบที่สองมักเกิดจาก resolver ที่แยก context ระหว่าง host (พวก proc-macro หรือ build.rs) กับ target จริง ถ้าใส่ flag `--format "{p} [{f}]"` จะเห็นว่า feature set ต่างกัน

```text
libc v0.2.177 []
libc v0.2.177 [default,std]

serde v1.0.229 [default,derive,rc,serde_derive,std]
serde v1.0.229 [default,derive,serde_derive,std]

syn v2.0.111 [..., fold, ...]
syn v2.0.111 [..., (ไม่มี fold), ...]
```

กรณีนี้จะไม่ทำให้เกิด type mismatch แต่อาจเพิ่มเวลา compile time และขนาดของ build cache ใน `target/`

คำสั่งที่มักใช้ในการ debug dependency graph

```bash
# ดูรายการ crate ที่ซ้ำทั้งหมด
cargo tree -d

# ดูว่า crate หรือ version นั้น ๆ ถูกดึงเข้ามาจากไหน (invert tree)
cargo tree -i thiserror@1.0.69 --depth 1

# ดู dependency graph ทั้งหมดรวมทุก platform และทุก feature
cargo tree -d --target all --all-features

# กรองดูเฉพาะ dependency ที่ลง target จริง (ตัด host และ proc-macro ออก)
cargo tree --target wasm32-unknown-unknown -e normal
```

## เคสตัวอย่างจาก rust-blog

ใน `Cargo.lock` ของ rust-blog มี packages ทั้งหมด 284 ตัว โดย `cargo deny check` ตรวจเจอ crate ซ้ำกัน 4 คู่

| crate | เวอร์ชันที่ซ้ำ | ที่มา | มีผลต่อ wasm binary ไหม |
|---|---|---|---|
| `convert_case` | 0.6.0 / 0.11.0 | 0.6 มาจาก `config` (ผ่าน `leptos_config`), 0.11 มาจาก `leptos_macro` | 0.6 อยู่ใน wasm, 0.11 อยู่ฝั่ง host |
| `syn` | 2.0.111 / 3.0.2 | 2.x มาจาก proc-macro หลายตัว, 3.x มาจาก `serde_derive` | ไม่มีผล (รันเฉพาะตอน compile ฝั่ง host) |
| `thiserror` | 1.0.69 / 2.0.18 | 1.x มาจาก `gloo-net`, 2.x มาจาก `leptos` | อยู่ใน wasm binary ทั้งคู่ |
| `wit-bindgen` | 0.46.0 / 0.51.0 | `wasip2` / `wasip3` ผ่าน `getrandom` | ไม่มีผล (เกี่ยวข้องกับ WASI dev-dependencies) |

ใช้ `cargo tree -i` แกะดูต้นทาง

```text
$ cargo tree -i thiserror@1.0.69 --depth 1
thiserror v1.0.69
└── gloo-net v0.6.0

$ cargo tree -i convert_case@0.6.0
convert_case v0.6.0
└── config v0.15.22
    └── leptos_config v0.8.10
        └── leptos v0.8.20

$ cargo tree -i syn@3.0.2 --depth 1
syn v3.0.2
└── serde_derive v1.0.229 (proc-macro)
```

จุดที่ต้องโฟกัสคือ **crate นั้นหลุดเข้าไปอยู่ใน production artifact หรือไม่** 
อย่าง `syn` ถึงจะซ้ำสองเวอร์ชันใหญ่ แต่มีบทบาทแค่ตอนรัน proc-macro บนฝั่ง host จึงไม่กระทบขนาดไฟล์ wasm 

ตรงข้ามกับ `thiserror` ที่ถูกดึงเข้าไปใน wasm artifact ทั้ง 1.x (จาก `gloo-net`) และ 2.x (จาก `leptos`) ซึ่งเคสนี้จะส่งผลให้ขนาด binary บวมขึ้นจริง

## ผลกระทบของ Dependency ที่ซ้ำซ้อน

1. **Compile Time และ Build Cache สูงขึ้น** compiler ต้องดาวน์โหลดและคอมไพล์โค้ดซ้ำหลายรอบ CI cache ใหญ่ขึ้น
2. **Binary Size ใหญ่ขึ้น** ถ้า duplicate นั้นอยู่ใน target runtime จริง
3. **Type Mismatch ข้ามเวอร์ชัน** นำ type ข้าม crate version มาทำงานร่วมกันไม่ได้ (Error E0308)
4. **Runtime Bug ที่จับได้ยาก** เช่น การใช้ `Any::downcast_ref` ข้ามเวอร์ชัน จะได้ `None` เสมอเพราะ TypeId ไม่ตรงกัน โค้ดคอมไพล์ผ่านแต่ทำงานผิดพลาดตอน runtime
5. **Security Advisory ตกค้าง** ถ้า `cargo audit` เตือนเรื่องช่องโหว่ใน crate เวอร์ชันเก่า แม้เราจะอัปเดตเวอร์ชันใหม่ใน `Cargo.toml` แล้ว แต่ถ้า dependency ตัวอื่นยังล็อกเวอร์ชันเก่าไว้ ช่องโหว่นั้นก็ยังคงอยู่ในโปรเจกต์

## แนวทางการจัดการ

1. **ลองรัน `cargo update` ดูก่อน** บ่อยครั้งที่ upstream อัปเดตแพตช์ให้ขยับมาใช้ SemVer ช่วงเดียวกันแล้ว การ update อาจช่วยยุบ dependency ให้เหลือตัวเดียวได้ทันที
2. **หา crate ที่ล็อกเวอร์ชันเก่า** ใช้ `cargo tree -i crate@version` เพื่อดูว่า package ไหนเป็นตัวดึงเวอร์ชันเก่าเข้ามา และตรวจสอบใน crates.io หรือ GitHub ว่ามีเวอร์ชันใหม่ให้ขยับหรือไม่
3. **ส่ง Issue หรือ PR ไปยัง Upstream** ถ้า library ปลายทางยังไม่อัปเดต การแจ้งหรือช่วยทำ PR เป็นทางแก้ที่ถูกต้องที่สุด
4. **ตรวจสอบการระบุ Version Requirement ในโปรเจกต์เราเอง** หลีกเลี่ยงการล็อกเวอร์ชันตายตัวแบบ `=1.2.3` ถ้าไม่จำเป็น เพราะจะทำให้ Cargo รวมเวอร์ชันกับ dependency อื่น ๆ ไม่ได้
5. **คอนฟิกข้อยกเว้นใน `cargo-deny`** ถ้ายังจำเป็นต้องรอ upstream อัปเดต ให้ตั้งค่าเป็น `warn` หรือเพิ่มในรายการ `skip` ของ `cargo-deny` พร้อมใส่เหตุผลกำกับไว้

```toml
[bans]
multiple-versions = "deny"

skip = [
  { crate = "thiserror@1", reason = "gloo-net ยังใช้ 1.x รอ upstream อัปเดตตาม leptos-rs/leptos#xxxx" },
]
```

6. **ใช้ `[patch.crates-io]` เมื่อจำเป็นจริง ๆ** หากจำเป็นต้องแก้ด่วน สามารถ fork repository มาแก้ dependency ชั่วคราว แล้ว patch ผ่าน `Cargo.toml` ได้

*(สำหรับคนทำ library มีเทคนิค [semver trick](https://github.com/dtolnay/semver-trick) ที่เวลาออก major version ใหม่ ให้ทำ patch เวอร์ชันเก่าตัวล่าสุดแล้ว re-export type จากตัวใหม่มา เพื่อให้ downstream อัปเกรดตามได้ง่ายขึ้นโดยไม่เกิดปัญหา type mismatch)*

## สรุป

1. **Cargo รวมเวอร์ชันให้อัตโนมัติเฉพาะเมื่อ SemVer เข้ากันได้** ถ้าเป็นคนละ major version (หรือคนละ minor ในเคส `0.x`) มันจะแยกคอมไพล์คนละชุดเพื่อป้องกัน breaking change
2. **`cargo tree -d` แสดงทั้ง crate คนละเวอร์ชัน และ crate เวอร์ชันเดียวกันที่ compile คนละ feature** เช็คความแตกต่างได้ด้วย flag `--format "{p} [{f}]"`
3. **ดูบริบทว่า dependency นั้นถูกคอมไพล์ลง target จริงหรือไม่** ถ้าซ้ำเฉพาะ build-dependencies หรือ proc-macro ฝั่ง host ก็จะมีผลแค่เรื่อง compile time แต่ไม่กระทบขนาด production binary
4. **ตั้ง policy ให้เหมาะสม** การเปิด `cargo deny check` ใน CI ช่วยให้เรารู้ว่ามี duplicate ตัวไหนหลุดเข้ามาบ้าง และสามารถตัดสินใจได้ว่าจะแก้ไข ยอมรับชั่วคราว หรือรอ upstream อัปเดตต่อไป

---

**Credit & Reference:**
- [Cargo Book: Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- [Cargo Book: Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- [Cargo Book: cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html)
- [cargo-deny: bans check](https://embarkstudios.github.io/cargo-deny/checks/bans/index.html)
- [The semver trick (dtolnay)](https://github.com/dtolnay/semver-trick)
- [How Rust Solved Dependency Hell (Stephen Coakley)](https://stephencoakley.com/2019/04/24/how-rust-solved-dependency-hell)
- [ซอร์สโค้ด rust-blog](https://github.com/rust-blog/rust-blog.github.io)
