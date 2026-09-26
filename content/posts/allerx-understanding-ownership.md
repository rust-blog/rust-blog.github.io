---
title: "Understanding Ownership ผ่านโปรเจกต์ AllerX"
date: "2026-09-26"
description: "ปิดบทที่ 4 ของ The Rust Book ด้วยโค้ดจริงจากโปรเจกต์ AllerX"
tags: [rust, tauri]
author: "suradet-ps"
---

ถ้าถามว่าเรื่องไหนใน Rust ที่คนเริ่มเขียนใหม่ ๆ สะดุดมากที่สุด ผมว่าคำตอบส่วนใหญ่คือ **Ownership** ครับ มันเป็นแนวคิดที่ไม่มีในภาษาไหนที่เราเคยชิน และเป็นกำแพงแรกที่ทำให้หลายคนถอยกลับไปเขียนภาษาอื่น แต่วันที่เราเข้าใจมันจริง ๆ เราจะเห็นว่ากฎทุกข้อมีเหตุผลของมัน

บทความนี้ผมจะไม่เล่า Ownership แบบท่องจำตามตำรา แต่จะพาไปดู **โค้ดจริงจากโปรเจกต์ AllerX** ว่าแนวคิดแต่ละส่วนเรานำไปใช้อย่างไร และทำไมเราถึงเขียนแบบนั้น เป้าหมายคืออ่านจบแล้วเข้าใจ Ownership จริง ๆ ไม่ใช่แค่ผ่านบทที่ 4 ของ The Rust Book นะครับ

## AllerX คืออะไร

**AllerX** ([github.com/suradet-ps/allerx](https://github.com/suradet-ps/allerx)) เป็น Desktop App ที่เภสัชกรและแพทย์ใช้ **ตรวจประวัติการได้รับยา** ของผู้ป่วยก่อนประเมินอาการแพ้ยา ตัวโปรแกรมเชื่อมต่อกับฐานข้อมูลโรงพยาบาลแบบ **อ่านอย่างเดียวเท่านั้น** แล้วตอบคำถามเดียวให้เร็วที่สุดว่า *"คนไข้รายนี้เคยได้รับยานี้หรือยัง และครั้งล่าสุดเมื่อไหร่"*

ตัว workspace แบ่งเป็นชั้น ๆ ที่มีทิศทางการพึ่งพาชัดเจน

```
allerx/
├── src-tauri/            # Tauri 2 shell — adapter บาง ๆ ไม่มี business logic
├── crates/
│   ├── models/           # โดเมนไทป์ล้วน ๆ ไม่มี I/O
│   ├── hosxp-connector/  # ชั้นเดียวที่แตะ MySQL + read-only guard
│   └── search-core/      # business logic + repository trait
└── app/                  # Leptos 0.8 CSR/WASM frontend
```

ownership คือสิ่งที่ทำให้การแบ่งชั้นแบบนี้ทำงานได้โดย **ไม่ต้องใช้ garbage collector และไม่มี data race** ไปดูกันที่ละเรื่องครับ

> ทุก path ที่อ้างถึงในบทความนี้เป็น path สัมพัทธ์จาก root ของรีโป เปิดดูโค้ดเต็มได้ที่ [suradet-ps/allerx](https://github.com/suradet-ps/allerx)

---

## 1. กฎ 3 ข้อที่ต้องเข้าใจก่อน

The Rust Book สรุป Ownership ไว้ 3 ข้อ ซึ่งเราจะอ้างถึงตลอดบทความนี้

1. **ทุกค่าใน Rust มีเจ้าของ (owner) หนึ่งคน**
2. **ณ เวลาหนึ่ง มีเจ้าของได้เพียงคนเดียว**
3. **เมื่อเจ้าของหลุดออกจาก scope ค่านั้นจะถูก drop**

คำว่า "ค่า" (value) หมายถึงข้อมูลจริงในหน่วยความจำ ส่วน "เจ้าของ" (owner) คือ binding ที่ถือมันอยู่ ต่างกันตรงที่ binding เป็นแนวคิดระดับโค้ด คอมไพเลอร์เลือกเองว่าจะวางค่าไว้ที่ stack, ที่ register หรือคำนวณล่วงหน้าไปเลย (ถ้าอยากลงลึกกลไกนี้ อ่านต่อได้ที่ [Rust Variables: Deep Dive](/post/rust-variables-deep-dive))

ประเด็นสำคัญที่ต้องเข้าใจคือ **Ownership ไม่ได้ทำงานตอนรัน** ไม่มี runtime check ไม่มี GC คอยกวาดทีหลัง ทุกอย่างถูกตรวจตอนคอมไพล์ ถ้ากฎถูกละเมิดโปรแกรมจะไม่ผ่านการคอมไพล์ตั้งแต่แรก และเมื่อผ่านได้ โค้ดที่ได้จะเร็วเทียบเท่า C 

มาดูกันว่ากฎทั้ง 3 ข้อหน้าตาเป็นยังไง

---

## 2. กฎข้อ 1: "ค่า" กับ "เจ้าของ" ในโดเมนของ AllerX

เริ่มจากไทป์ที่ AllerX ใช้จริง ข้อมูลผู้ป่วยหนึ่งคนถูกนิยามไว้แบบนี้

`crates/models/src/lib.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientSummary {
    pub hn: String,
    pub cid: Option<String>,
    pub full_name_th: String,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<String>,
}
```

สังเกตว่า `hn` และ `full_name_th` เป็น `String` ไม่ใช่ `&str` ตรงนี้เองที่แสดงความต่างระหว่าง **"การเป็นเจ้าของข้อความ"** กับ **"การยืมข้อความ"** ให้เห็นชัด `String` เป็นเจ้าของบัฟเฟอร์บน heap ที่ตัวเองจองมา ฉะนั้นใครก็ตามที่มี `PatientSummary` อยู่ ก็เป็นเจ้าของข้อมูลทั้งหมดในนั้นด้วย

ในทางกลับกัน ค่าคงที่อย่างคำสั่ง SQL เป็น `&'static str` หมายถึงการยืมข้อมูลที่ฝังอยู่ในตัว binary เอง

`crates/hosxp-connector/src/readonly_guard.rs`

```rust
/// คำสั่ง M0 smoke test
pub const PING_SQL: &str = "SELECT 1";
```

ตัว literal `"SELECT 1"` ถูกฝังอยู่ใน binary ตั้งแต่ตอนคอมไพล์ ไม่มีใครต้อง drop และ `PING_SQL` ก็แค่ชี้ไปที่มัน นี่คือเหตุผลที่ string literal ใน Rust มีไทป์ `&str` ไม่ใช่ `String`

### stack กับ heap ในภาพเดียว

เพื่อให้เห็นภาพ จำง่าย ๆ ว่า

| ข้อมูล | อยู่ที่ไหน | ใครเป็นเจ้าของ | ถูกล้างเมื่อไหร่ |
|---|---|---|---|
| `u32`, `bool`, `char` | stack | binding ที่ประกาศ | จบ scope |
| `NaiveDate` (struct เล็ก) | stack | binding | จบ scope |
| `String` | ตัวชี้ 3 ตัวอยู่ stack, ตัวอักษรอยู่ heap | binding ที่ถือ `String` | จบ scope → เรียก `drop` คืน heap |
| `Vec<DrugHistoryRecord>` | ตัวชี้ + len + capacity บน stack, ข้อมูลบน heap | binding ที่ถือ `Vec` | จบ scope → drop ทุก element แล้วคืน heap |

`String` กิน 24 ไบต์บน stack (pointer 8 + length 8 + capacity 8) เท่ากันทุกความยาว ส่วนเนื้อหาไปอยู่บน heap การย้าย `String` จึงเป็นการคัดลอกแค่ 3 ตัวเลขนี้ ไม่ใช่คัดลอกข้อความทั้งก้อน

### scope: binding มีชีวิตอยู่ช่วงไหน ?

binding ทุกตัวเกิดใน **scope** (block `{}`) มีชีวิตตั้งแต่บรรทัดที่ประกาศไปจนจบ block แล้วถูก drop ตอนออกจาก scope ลองดูของจริงใน `merge_drug_history`

```rust
pub fn merge_drug_history(opd: Vec<...>, ipd: Vec<...>) -> Vec<...> {
    let mut all = opd;   // all เกิดตรงนี้ และเป็นเจ้าของ Vec
    all.extend(ipd);
    all
}                        // ถ้าไม่ return all, all จะถูก drop ตรงนี้
```

`all` มีชีวิตอยู่จนจบฟังก์ชัน แต่เพราะถูก return ออกไป ownership จึงย้ายไปให้ caller แทนที่จะถูก drop ทิ้ง ประเด็นเดียวกับที่ The Rust Book อธิบายว่า binding มีผลตั้งแต่ "เข้ามาใน scope" จนถึง "ออกจาก scope" นั่นเอง

---

## 3. กฎข้อ 2: มีเจ้าของได้ทีละคน การย้าย (Move)

นี่คือหัวใจที่มือใหม่สับสนกันมากที่สุด ลองดูตัวอย่างใน Chapter 4

```rust
let s1 = String::from("hello");
let s2 = s1; // ไม่ใช่ copy แต่เป็น move
println!("{s1}"); // ❌ ใช้ไม่ได้แล้ว
```

`s2 = s1` ไม่ได้คัดลอกข้อมูลบน heap Rust แค่คัดลอก "ตัวชี้" ทั้ง 3 ตัวบน stack แล้ว **ประกาศว่า `s1` ใช้งานไม่ได้อีกต่อไป** เพื่อไม่ให้มีสองเจ้าของคอยคืนหน่วยความจำก้อนเดียวกัน (double free)

### Move ใน AllerX

ใน AllerX มีฟังก์ชันที่ออกแบบให้ **รับ ownership ของ Vec เข้ามาเลย** ชื่อว่า `merge_drug_history`

`crates/search-core/src/history.rs`

```rust
pub fn merge_drug_history(
    opd: Vec<DrugHistoryRecord>,
    ipd: Vec<DrugHistoryRecord>,
) -> Vec<DrugHistoryRecord> {
    let mut all = opd;
    all.extend(ipd);
    all.sort_by(|a, b| {
        b.visit_date
            .cmp(&a.visit_date)
            .then_with(|| visit_type_rank(a.visit_type).cmp(&visit_type_rank(b.visit_type)))
    });
    all
}
```

อ่านบรรทัดต่อบรรทัดจะเห็น ownership เคลื่อนที่ไปทั้งฟังก์ชัน

- `opd` และ `ipd` ถูก **ย้ายเข้ามา** เป็นของฟังก์ชันนี้ (caller ไม่มีสิทธิ์ใช้ต่อ)
- `let mut all = opd;` ย้าย ownership จาก `opd` ไป `all`
- `all.extend(ipd);` ย้ายทุก element จาก `ipd` เข้าไปใน `all` แล้วทิ้ง `ipd`
- `all` ถูก **คืนออกไป** เป็น return value (ownership ออกจากฟังก์ชัน)

ฟังก์ชันนี้ไม่ต้อง `clone()` สักตัว เพราะตั้งใจ "กิน" ข้อมูลที่รับเข้ามาแล้วสร้างของใหม่คืนไป ซึ่งเป็นสไตล์ที่ Rust แนะนำ เรียกว่า **ownership transfer** หรือ move semantics

### แล้วถ้าฝืนใช้ค่าที่ถูก move ไปแล้วล่ะ?

ลองรันโค้ดนี้ดู (ผมตั้งใจทำเวอร์ชันที่พัง เพื่อให้เห็น error จริงจาก `rustc 1.98`)

```rust
fn merge_drug_history(opd: Vec<String>, ipd: Vec<String>) -> Vec<String> {
    let mut all = opd;
    all.extend(ipd);
    all
}

fn main() {
    let opd = vec![String::from("a")];
    let ipd = vec![String::from("b")];
    let merged = merge_drug_history(opd, ipd);
    println!("{merged:?}");
    println!("{opd:?}"); // ❌ ใช้ opd หลังถูก move ไปแล้ว
}
```

```console
error[E0382]: borrow of moved value: `opd`
  --> own_move.rs:12:16
   |
 8 |     let opd = vec![String::from("a")];
   |         --- move occurs because `opd` has type `Vec<String>`, which does not implement the `Copy` trait
 9 |     let ipd = vec![String::from("b")];
10 |     let merged = merge_drug_history(opd, ipd);
   |                                     --- value moved here
11 |     println!("{merged:?}");
12 |     println!("{opd:?}");
   |                ^^^ value borrowed here after move
   |
note: consider changing this parameter type in function `merge_drug_history` to borrow instead if owning the value
isn't necessary
  --> own_move.rs:1:28
   |
 1 | fn merge_drug_history(opd: Vec<String>, ipd: Vec<String>) -> Vec<String> {
   |    ------------------      ^^^^^^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
help: consider cloning the value if the performance cost is acceptable
   |
10 |     let merged = merge_drug_history(opd.clone(), ipd);
   |                                        ++++++++
```

คอมไพเลอร์ไม่ได้แค่บอกว่าผิด แต่บอกว่า **ย้ายที่บรรทัดไหน** **ถูกใช้ผิดที่บรรทัดไหน** และเสนอทางออก 2 ทาง คือ เปลี่ยนไปยืม (`&Vec`) หรือ clone ถ้าจำเป็นจริง ๆ นี่คือความต่างระหว่างบั๊กที่ถูกจับได้ตอนคอมไพล์ กับบั๊ก use-after-free ที่รันแล้วพังแบบไม่แน่นอน

> **ทำไม Rust ต้องบังคับ move แทนที่จะ copy ให้เงียบ ๆ?**
> ถ้า copy ตัวชี้โดยไม่ทำให้ตัวเก่าใช้งานไม่ได้ จะกลายเป็นสองเจ้าของชี้ heap ก้อนเดียวกัน พอจบ scope ทั้งคู่จะพยายาม free หน่วยความจำเดียวกัน (double free) ซึ่งเป็นช่องโหว่ความปลอดภัยระดับร้ายแรง Rust จึงเลือกทางที่ปลอดภัยที่สุด และปล่อยให้เราเป็นคนกำหนดเองว่า "ตรงนี้อยากคัดลอกจริง ๆ" ด้วย `.clone()`

---

## 4. กฎข้อ 3: หลุด scope เมื่อไหร่ ถูก drop ทันที (RAII)

กฎข้อสุดท้ายคือหัวใจที่ทำให้ Rust ไม่ต้องมี garbage collector เมื่อเจ้าของหลุดออกจาก scope ระบบจะเรียกฟังก์ชัน `drop` ให้อัตโนมัติ ซึ่งเป็นจุดที่ทรัพยากรถูกคืน ทั้งหน่วยความจำ, ไฟล์, connection

### drop ที่สำคัญที่สุดใน AllerX: รหัสผ่าน

นี่คือตัวอย่างที่ชัดเจนที่สุดในโปรเจกต์ `HosxConfig` เก็บรหัสผ่านฐานข้อมูลไว้ใน `SecretString`

`crates/hosxp-connector/src/config.rs`

```rust
/// HOSxP connection settings. Plaintext in memory **only** — never
/// serialized to disk in this form.
#[derive(Clone)]
pub struct HosxConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: SecretString,
}
```

`SecretString` มาจาก crate `secrecy` ซึ่งออกแบบมาโดยเฉพาะ **หน่วยความจำของมันจะถูก zeroize (เขียนทับด้วยศูนย์) ทันทีที่ตัวมันถูก drop** เพื่อไม่ให้รหัสผ่านค้างอยู่ใน RAM รอให้ใครมาอ่าน

ตรงนี้เชื่อมกับ Ownership ตรง ๆ เพราะ `SecretString` ถูกย้ายไปเป็นเจ้าของของ `HosxConfig` เมื่อ `HosxConfig` หลุด scope `SecretString` ก็ถูก drop ตาม แล้ว zeroize ให้เอง เราไม่ต้องเขียนโค้ดล้างรหัสผ่านเองแม้แต่บรรทัดเดียว

แต่มีจุดที่ต้องระวัง: การอ่านค่าออกจาก `SecretString` ทำไม่ได้ตรง ๆ ต้องเรียก `.expose_secret()` เท่านั้น ทำให้ทุกจุดที่แตะรหัสผ่านจริงเป็นจุดที่เราตั้งใจชัดเจน

`crates/hosxp-connector/src/pool.rs`

```rust
let options = MySqlConnectOptions::new()
    .host(&cfg.host)
    .port(cfg.port)
    .database(&cfg.database)
    .username(&cfg.user)
    .ssl_mode(configured_ssl_mode())
    .password(cfg.password.expose_secret());
```

สังเกตว่า `.host(&cfg.host)` รับ `&str` (ยืม) แต่ `.password(...)` ต้องเปิดเผยค่าออกมาเป็น `&str` ชั่วคราว แล้ว sqlx จะคัดลอกค่าไปเก็บไว้เอง

### Debug ก็ไม่หลุด

อีกจุดที่มักพลาดคือการ `println!("{:?}", config)` แล้วรหัสผ่านหลุดลง log AllerX แก้ด้วยการเขียน `Debug` เอง

`crates/hosxp-connector/src/config.rs`

```rust
impl fmt::Debug for HosxConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HosxConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"***")
            .finish()
    }
}
```

มี Unit Test กันไว้ด้วยว่า debug ต้องไม่มีรหัสผ่านจริง

```rust
#[test]
fn debug_impl_masks_password() {
    let debug = format!("{:?}", sample_config());
    assert!(!debug.contains("s3cret!p@ss"));
    assert!(debug.contains("***"));
}
```

### drop ที่มองไม่เห็นแต่สำคัญ: connection pool

อีกจุดที่ ownership ทำงานอยู่เบื้องหลังคือเมื่อ `MySqlPool` ถูก drop มันจะปิด connection ทั้งหมดที่ถืออยู่ ตอนผู้ใช้บันทึกการตั้งค่าใหม่ โค้ดตั้งใจทิ้ง pool เก่าทันที

`src-tauri/src/commands.rs`

```rust
// Reconfigured — drop the old pool and re-verify in the background
*state.pool.lock().await = None;
```

การ assign `None` ทับ `Option<MySqlPool>` ทำให้ pool เก่าไม่มีเจ้าของและถูก drop ทันที connection ที่ค้างอยู่ถูกปิดอย่างเป็นระเบียบ เราไม่ต้องเรียก `close()` เองเลย นี่คือกรณี "Scope and Assignment" ที่ The Rust Book อธิบายไว้ เมื่อ assign ค่าใหม่ทับ binding เดิม ค่าเก่าจะหลุด scope และถูก drop ทันที ไม่ใช่รอจนจบฟังก์ชัน

> **นี่คือ RAII (Resource Acquisition Is Initialization)**
> ทรัพยากรผูกกับอายุขัยของเจ้าของ เกิดมาพร้อมกัน ตายไปพร้อมกัน ต่างจาก GC ที่ "ไม่รู้เมื่อไหร่จะเก็บ" และต่างจากภาษา manual ที่เราลืม free ได้ Rust เลือกใช้ **deterministic drop** คือรู้แน่นอน 100% ว่าทรัพยากรจะถูกคืนตอนไหน

---

## 5. Copy กับ Clone: การคัดลอกสองแบบที่ต่างกันสิ้นเชิง

หลายคนสับสนระหว่าง `Copy` กับ `Clone` ทั้งที่มันต่างกันมาก

- **`Copy`** คือการคัดลอกระดับบิตที่ราคาถูกมาก (bitwise copy) หลังคัดลอกแล้ว **ตัวเดิมยังใช้ได้** เพราะข้อมูลอยู่บน stack ล้วน ๆ
- **`Clone`** คือการคัดลอกแบบ "ลึก" ที่อาจมีค่าใช้จ่ายสูง (จอง heap ใหม่) เพราะตั้งใจให้ตัวเดิมและตัวใหม่เป็นอิสระต่อกัน

Rust ไม่เคยคัดลอกข้อมูลบน heap เองโดยไม่บอก ดังนั้นทุก `clone()` ที่เห็นในโค้ดคือสัญญาณว่า **"ตรงนี้มีค่าใช้จ่าย และเราตั้งใจจ่าย"**

### AllerX ใช้ Copy ตรงไหน

`VisitType` เป็น enum เล็ก ๆ ที่บอกว่าการจ่ายยาเกิดที่ผู้ป่วยนอกหรือผู้ป่วยใน

`crates/models/src/lib.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VisitType {
    Opd,
    Ipd,
}
```

การที่มี `Copy` ทำให้เวลาส่ง `VisitType` เข้าฟังก์ชัน ตัวเดิมไม่ถูก move เช่นใน `merge_drug_history` เราส่ง `a.visit_type` และ `b.visit_type` เข้า `visit_type_rank` ได้เลย ทั้งที่ `a` กับ `b` ยังเป็น reference อยู่

`crates/search-core/src/query_kind.rs` ก็มี `QueryKind` เป็น `Copy`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    Cid,
    Hn,
    Name,
}
```

และฝั่ง frontend `ConnectionHealth` ก็เป็น `Copy`

`app/src/state.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionHealth {
    Unconfigured,
    Connected,
    Disconnected,
}
```

ทำไม `VisitType` ถึงมี `Copy` แต่ `PatientSummary` ไม่มี? เพราะ `VisitType` เก็บแค่เลข discriminant ไม่กี่ไบต์บน stack ไม่มี heap ให้เป็นเจ้าของ ส่วน `PatientSummary` มี `String` ข้างใน ถ้าให้ `Copy` มันจะกลายเป็นการคัดลอกที่แอบจองหน่วยความจำบน heap โดยที่เราไม่รู้ตัว ซึ่งขัดกับปรัชญาของ Rust

> **หลักจำง่าย ๆ:** ถ้าไทป์ไม่มีอะไรบน heap และทุก field เป็น `Copy` ด้วย ไทป์นั้นก็เป็น `Copy` ได้ ถ้ามี `String`/`Vec`/ทรัพยากรอยู่ข้างใน ให้ใช้ `Clone` แทน

---

## 6. Ownership กับการส่งเข้า-คืนออกจากฟังก์ชัน

การส่งค่าเข้าฟังก์ชันเหมือนการ assign ทุกประการ คือ move หรือ copy ตามไทป์ และการ return ก็ย้าย ownership ออกมา

หัวใจของ AllerX อยู่ที่การแปลง "คำที่เภสัชกรพิมพ์" ให้กลายเป็น verdict สุดท้าย มาดูว่า ownership ไหลยังไง

`crates/search-core/src/resolution.rs`

```rust
pub fn classify_drug_resolution(
    exact: Option<DrugItem>,
    candidates: Vec<DrugItem>,
) -> DrugResolution {
    match exact {
        Some(drug) => DrugResolution::Exact { drug },
        None => DrugResolution::Candidates { items: candidates },
    }
}
```

ฟังก์ชันนี้ **รับ `exact` และ `candidates` มาเป็นเจ้าของ** แล้วหยิบใส่ enum `DrugResolution` ตัวที่ถูกเลือกจะได้ ownership ไป ตัวที่ไม่ถูกเลือก (เช่น `candidates` ตอนที่ `exact` เป็น `Some`) จะถูก drop ทันทีที่จบ `match` โดยที่เราไม่ต้องเขียนโค้ดล้างอะไรเลย

จากนั้น verdict ถูกสร้างที่จุดเดียว

```rust
pub fn verdict_from_resolution(
    resolution: DrugResolution,
    records: Vec<allerx_models::DrugHistoryRecord>,
    truncated: bool,
) -> HistoryVerdict {
    match resolution {
        DrugResolution::Exact { drug } => HistoryVerdict::Resolved {
            drug,
            history: ResolvedHistory { records, truncated },
        },
        DrugResolution::Candidates { items } => HistoryVerdict::Unresolved { candidates: items },
    }
}
```

ownership ไหลต่อเนื่องเป็นสาย `DrugItem` ถูกย้ายเข้า `HistoryVerdict::Resolved` และ `Vec<DrugHistoryRecord>` ถูกย้ายเข้า `ResolvedHistory` ไม่มีการคัดลอกข้อมูลผู้ป่วยเลยแม้แต่ก้อนเดียว

ตรงนี้เองที่ทำให้การออกแบบ API ปลอดภัยมาก การที่ฟังก์ชัน "กิน" ข้อมูลเข้าไปแล้วคืน verdict ออกมา ทำให้ไม่มีทางที่ข้อมูลจะถูกแก้ระหว่างทางจากที่อื่น เพราะมีเจ้าของทีละคนเท่านั้น

> **ข้อสังเกต** comment ใน `resolution.rs` เขียนไว้ว่า *"This is the single place where `HistoryVerdict` is constructed"* นี่คือ invariant ด้านความปลอดภัยของผู้ป่วย ที่โค้ดบังคับผ่านการออกแบบ ownership ให้มีจุดเดียวที่สร้าง verdict ได้ ถ้าไม่มี ownership ที่ชัดเจน การรับประกันแบบนี้จะทำได้ยากกว่ามาก

---

## 7. References และ Borrowing: ยืมโดยไม่เอาตัวจริงไป

การ move ทุกครั้งไม่สะดวกเสมอไป ถ้าเราแค่อยาก "อ่าน" ค่าของคนอื่น เราควรยืม (`&T`) แทน

ลองเทียบ signature สองแบบที่ AllerX ใช้

- `merge_drug_history(opd: Vec<...>, ipd: Vec<...>)` — **move** เพราะตั้งใจจะรวบข้อมูลเป็นก้อนใหม่
- `search_patients(&self, term: &str, ...)` — **borrow** เพราะแค่อ่าน แล้วคืนผลลัพธ์เป็นของใหม่

`crates/search-core/src/repository.rs`

```rust
async fn search_patients(
    &self,
    term: &str,
    kind: QueryKind,
) -> Result<Vec<PatientSummary>, RepositoryError>;
```

`&self` คือการยืม repository เอง ไม่ต้อง move ทั้งก้อนเข้าไปในเมธอด `term: &str` ก็ยืมข้อความ ไม่ต้องโอน `String` ให้ `kind: QueryKind` ถูกคัดลอกแบบ `Copy`

พอ implementation ต้องส่งค่าลง SQL จริง จะเห็นการยืมซ้อนกันอีกชั้น

`crates/hosxp-connector/src/repository.rs`

```rust
async fn guarded_fetch<T>(&self, sql: &str, params: &[&str]) -> Result<Vec<T>, Error>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin,
{
    assert_read_only(sql).map_err(|_| Error::Guard)?;
    self.raw_fetch(sql, params).await.map_err(Error::from)
}
```

`sql: &str` ยืมสตริง SQL (ซึ่งจริง ๆ เป็น `&'static str` จาก `queries.rs`) และ `params: &[&str]` คือ slice ของ reference ไม่มีข้อมูลไหนถูกคัดลอกเลย

และเมื่อเรียกใช้จริง เราแค่ส่ง `&`

```rust
let hits = match kind {
    QueryKind::Cid => self.fetch_patients(PATIENT_SEARCH_BY_CID, &[term]).await?,
    QueryKind::Hn => self.fetch_patients(PATIENT_SEARCH_BY_HN, &[term]).await?,
    QueryKind::Name => {
        let prefix = format!("{term}%");
        let mut hits = self
            .fetch_patients(PATIENT_SEARCH_NAME_PREFIX, &[&prefix, &prefix, &prefix])
            .await?;
        // ...
    }
};
```

ในกรณี name เรา **สร้าง** `String` ใหม่ชื่อ `prefix` (เพราะต้องเติม `%`) แล้วยืมมันด้วย `&prefix` ส่งลงไป 3 ครั้ง จะเห็นว่า Rust ยอมให้ยืมแบบ immutable ได้หลายครั้งพร้อมกัน เพราะไม่มีใครแก้ข้อมูลระหว่างที่คนอื่นอ่าน

### Borrow ไม่ทำให้เจ้าของถูก drop

จุดที่คนเข้าใจผิดบ่อยคือคิดว่า reference ทำให้ ownership ถูกย้ายไป ที่จริงไม่ใช่เลย

```rust
pub fn drug_identity(drug: &DrugItem) -> String {
    match &drug.strength {
        Some(strength) => format!("{} ({strength})", drug.name),
        None => drug.name.clone(),
    }
}
```

`app/src/state.rs` ฟังก์ชันนี้ยืม `&DrugItem` แล้วคืน `String` ก้อนใหม่ ตัว `drug` ยังเป็นของ caller เหมือนเดิม หลังเรียกจบ caller ยังใช้ `drug` ได้ตามปกติ

> **ทำไมต้องยืม?**
> เพราะ "การอ่าน" ควรมีต้นทุนต่ำที่สุด การ move เข้าไปแล้วต้องคืนกลับเป็นเรื่องน่ารำคาญ (ต้องใช้ tuple คืนหลายค่า) ส่วนการ clone ก็แพงเกินจำเป็นสำหรับแค่อ่าน การยืมจึงเป็นการประนีประนอมที่ลงตัวที่สุด และเนื่องจากทุกคนที่ยืมเป็น immutable จึงไม่มีใครแอบแก้ข้อมูลที่คนอื่นกำลังอ่านอยู่

---

## 8. Mutable References และกฎ "ได้อย่างใดอย่างหนึ่งเท่านั้น"

บางครั้งเราจำเป็นต้องแก้ไขค่าที่เรายืมมา นั่นคือ `&mut T` และมันมาพร้อมกฎที่เข้มที่สุดของ Rust

> **ณ เวลาหนึ่ง จะมี `&mut T` ได้หนึ่งตัว หรือ `&T` ได้หลายตัว แต่ห้ามมีทั้งสองแบบพร้อมกัน**

ดูตัวอย่างจริงที่ AllerX ใช้ mutable borrow เพื่อ "เติมข้อมูล"

`src-tauri/src/stats.rs`

```rust
pub fn record(&mut self, command: &str, elapsed_ms: u64, ok: bool) {
    if self.buffer.len() == CAPACITY {
        self.buffer.pop_front();
    }
    self.buffer.push_back(QuerySample {
        command: command.to_string(),
        at_ms: self.started.elapsed().as_millis() as u64,
        elapsed_ms,
        ok,
    });
}
```

`&mut self` คือการยืม struct แบบแก้ไขได้ เพราะต้อง `pop_front` และ `push_back` เข้า `VecDeque` ข้างใน ถ้าเป็น `&self` แบบ immutable จะทำไม่ได้เลย และการยืม `&mut` นี้เป็นแบบเอกสิทธิ์ ระหว่างที่ `record` ทำงาน จะไม่มีใครอ่าน `self` พร้อมกันได้

อีกตัวอย่างฝั่ง frontend ที่ใช้ mutable borrow ผ่าน closure

`app/src/components/drug_search.rs`

```rust
state.drug_chips.update(|chips| {
    chips.push(DrugChip { label, icode });
});
```

`update` จะส่ง `&mut Vec<DrugChip>` เข้าให้ closure เราแก้ในนั้นได้ โดย API ของ Leptos จัดการเรื่อง borrow ให้อย่างปลอดภัย

ในตัวอย่างนั้นมีกฎคลาสสิกที่ทำให้หลายคนติดกับดัก ลองดูโค้ดนี้ที่ผมตั้งใจทำให้พัง

```rust
fn main() {
    let mut chips = vec![String::from("paracetamol")];
    let label = &chips[0];                      // immutable borrow ตรงนี้
    chips.push(String::from("ibuprofen"));       // mutable borrow ตรงนี้
    println!("{label}");
}
```

```console
error[E0502]: cannot borrow `chips` as mutable because it is also borrowed as immutable
 --> own_borrow.rs:4:5
  |
3 |     let label = &chips[0];
  |                  ----- immutable borrow occurs here
4 |     chips.push(String::from("ibuprofen"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{label}");
  |                ----- immutable borrow later used here
```

ทำไม Rust ถึงไม่ยอม? เพราะ `push` อาจต้อง **ขยาย capacity แล้วย้ายข้อมูลทั้งก้อนไปที่ใหม่** ถ้ามันย้ายจริง `label` ที่ชี้ไปที่ที่อยู่เก่าจะกลายเป็น dangling pointer ทันที Rust จึงกันไว้ตั้งแต่ตอนคอมไพล์

### scope ของ reference จบที่ "การใช้งานครั้งสุดท้าย" ไม่ใช่จบที่ `}`

Rust สมัยใหม่ใช้ **non-lexical lifetimes (NLL)**: reference ตัวหนึ่งจะถือว่ามีชีวิตจนถึงครั้งสุดท้ายที่มันถูกใช้เท่านั้น พอเลยจุดนั้นไป borrow checker จะถือว่า borrow จบแล้ว แม้ block `{ }` จะยังไม่ปิด

ตัวอย่างจริงจาก `app/src/components/timeline.rs`

```rust
let key = results_key(state.check_seq.get_untracked(), &results);
if last_results_key.get_untracked().as_deref() != Some(key.as_str()) {
    last_results_key.set(Some(key));
    filter.set(None);
}
```

`key.as_str()` ยืม `key` อยู่ แต่การยืมนั้นถูกใช้แค่ในเงื่อนไข `if` พอเลยบรรทัดนั้นไป borrow ก็จบ จึงย้าย `key` เข้า `Some(key)` ในบล็อกได้ตามปกติ ถ้าเป็น borrow checker แบบเก่าที่ดูตาม block `{ }` เฉย ๆ โค้ดนี้จะคอมไพล์ไม่ผ่านทันที เพราะมันจะคิดว่า borrow ยังมีชีวิตอยู่ตลอดบล็อก

> **ทำไมกฎนี้ถึงสำคัญมากกับ concurrency?**
> เพราะ Data Race เกิดจาก 3 อย่างพร้อมกัน: (1) มีหลายตัวชี้ไปที่ข้อมูลเดียวกัน (2) อย่างน้อยหนึ่งตัวเขียน (3) ไม่มีกลไก sync กฎ borrow ของ Rust ตัดข้อ (1) หรือ (2) ทิ้งไปเลยในเคสที่อันตราย นั่นคือเหตุผลที่ Rust กล้าประกาศว่า "data race เป็นไปไม่ได้ใน safe code" ตั้งแต่ตอนคอมไพล์

---

## 9. Dangling Reference กับ Lifetime: ทำไม Rust ไม่ยอมให้คืน reference ไปหาของที่ตายแล้ว

ในภาษา C เราสามารถคืน pointer ไปหา local variable ได้ แล้วค่อยไปเจอค่าขยะในภายหลัง Rust ป้องกันปัญหานี้ด้วย **lifetime** ซึ่งเป็นสิ่งที่ borrow checker ใช้ตรวจว่าข้อมูลที่ reference อ้างถึงต้องมีอายุอย่างน้อยเท่ากับตัว reference เอง

มีตัวอย่างที่สวยมากใน AllerX คือไฟล์ `readonly_guard.rs` ที่ทำหน้าที่สแกน SQL ว่าเป็นคำสั่งอ่านอย่างเดียวหรือไม่

`crates/hosxp-connector/src/readonly_guard.rs`

```rust
pub fn assert_read_only(sql: &str) -> Result<(), GuardError> {
    let Some(rest) = strip_leading_comments(sql) else {
        return Err(GuardError);
    };
    let rest = rest.trim_start();
    let keyword_end = rest
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(rest.len());
    let keyword = rest[..keyword_end].to_ascii_uppercase();
    // ...
}
```

`strip_leading_comments` รับ `&str` เข้าไป แล้วคืน `&str` ออกมา

```rust
fn strip_leading_comments(mut sql: &str) -> Option<&str> {
    loop {
        sql = sql.trim_start();
        if let Some(rest) = sql.strip_prefix("--") {
            sql = rest.split_once('\n').map(|(_, after)| after).unwrap_or("");
        } else if let Some(rest) = sql.strip_prefix("/*") {
            sql = &rest[rest.find("*/")? + 2..];
        } else {
            return Some(sql);
        }
    }
}
```

`&str` ที่คืนออกมา **ไม่ใช่ข้อมูลชุดใหม่** แต่เป็น "หน้าต่าง" ที่มองเข้าไปใน `sql` ตัวเดิม นี่คือประเด็นของ lifetime คอมไพเลอร์จะมองว่า reference ที่คืนมา "ผูก" กับอายุของ input โดยอัตโนมัติ (เรียกว่า lifetime elision) จึงรับประกันได้ว่าตราบใดที่เรายังใช้ผลลัพธ์ ตัว `sql` ต้นทางจะยังไม่ถูก drop

นี่คือคำตอบว่าทำไมฟังก์ชันแบบนี้ใน Chapter 4 ถึงพัง

```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s // ❌ คืน reference ไปหาค่าที่กำลังจะตาย
}
```

คอมไพเลอร์จะฟ้องว่า *"this function's return type contains a borrowed value, but there is no value for it to be borrowed from"* เพราะไม่มีค่าที่อายุยืนพอจะให้ reference นั้นอ้างถึง หลังฟังก์ชันจบ `s` จะถูก drop ทางแก้คือคืน `String` ตรง ๆ ให้ ownership ออกมา

จุดนี้ทำให้เรากลับไปเข้าใจ `drug_identity` กับ `record_label` ใน AllerX ได้ว่า **ทำไมถึงคืน `String` ไม่ใช่ `&str`**

`app/src/state.rs`

```rust
pub fn record_label(record: &DrugHistoryRecord) -> String {
    match &record.strength {
        Some(strength) => format!("{} ({strength})", record.drug_name),
        None => record.drug_name.clone(),
    }
}
```

เหตุผลหลักคือ label ที่ได้เป็นข้อความที่ **เพิ่งประกอบขึ้นใหม่** ด้วย `format!` (ชื่อยา + ขนาดยา) ไม่มีอยู่ใน struct อยู่แล้ว จึงไม่มี field ไหนให้ยืมได้ ต้องคืนเป็น owned `String` และการคืน owned value แบบนี้ยังช่วยให้ข้อมูลถูกส่งต่อไปยัง signal หรือ UI ได้โดยไม่ต้องผูก lifetime กับ `record` อีกด้วย

> **สรุปกฎของ reference** (1) ณ เวลาหนึ่ง มี `&mut T` ได้หนึ่งตัว หรือ `&T` ได้หลายตัว แต่ห้ามมีทั้งสองแบบพร้อมกัน (2) reference ต้องชี้ไปยังค่าที่ valid เสมอ ตลอดอายุการใช้งานของมัน

---

## 10. Slice: reference ที่มองเห็นเป็นช่วง

Slice คือ reference ชนิดหนึ่งที่ชี้ไปยังลำดับข้อมูลต่อเนื่องกัน โดยไม่มี ownership หน้าตาเป็น `&[T]` สำหรับ array/vector และ `&str` สำหรับข้อความ

กลับไปที่ `readonly_guard.rs` อีกครั้ง นี่คือการใช้ slice ที่ตรงตำรา Chapter 4 ที่สุดในโปรเจกต์

```rust
let keyword_end = rest
    .find(|c: char| !c.is_ascii_alphanumeric())
    .unwrap_or(rest.len());
let keyword = rest[..keyword_end].to_ascii_uppercase();
let followed_by_boundary = rest[keyword_end..]
    .chars()
    .next()
    .is_some_and(|c| c.is_whitespace() || c == '(');
```

- `rest[..keyword_end]` คือ slice ตั้งแต่ต้นข้อความจนถึงตัวอักษรตัวแรกที่ไม่ใช่ alphanumeric - สร้าง `&str` ตัวใหม่ที่มองเข้าไปใน `rest` โดยไม่คัดลอกข้อความ
- `rest[keyword_end..]` คือ slice ตั้งแต่ตำแหน่งนั้นจนจบ
- ทั้งสองอันเป็นแค่ **pointer + length** เท่านั้น ต้นทุนจึงต่ำมาก

นี่คือเหตุผลที่ฟังก์ชัน `first_word` ใน Chapter 4 ควรคืน `&str` แทนที่จะคืน `usize` เพราะ slice "ผูก" กับเจ้าของข้อมูล ทำให้ borrow checker ตรวจจับได้ทันทีถ้าข้อมูลต้นทางถูกแก้

ฝั่ง AllerX ยังใช้ slice เต็มไปหมดใน signature

`crates/hosxp-connector/src/repository.rs`

```rust
async fn raw_fetch<T>(&self, sql: &str, params: &[&str]) -> Result<Vec<T>, sqlx::Error>
```

```rust
async fn fetch_first_working<T>(&self, candidates: &[(&str, &[&str])]) -> Result<Vec<T>, Error>
```

`&[&str]` คือ slice ของ `&str` และ `&[(&str, &[&str])]` คือ slice ของ tuple ที่ข้างในมี slice ซ้อนอีกที ฟังดูซับซ้อนแต่ทั้งหมดนี้คือการยืม ไม่มีข้อมูลไหนถูกคัดลอกเข้าฟังก์ชันเลย

ส่วน `check_drugs` ฝั่ง trait รับ slice ของ `String`

`crates/search-core/src/repository.rs`

```rust
async fn check_drugs(
    &self,
    hn: &str,
    drugs: &[String],
) -> Result<Vec<DrugCheckResult>, RepositoryError> {
    let mut results = Vec::with_capacity(drugs.len());
    for drug in drugs {
        let verdict = self.fetch_drug_history(hn, drug).await?;
        results.push(DrugCheckResult {
            term: drug.clone(),
            verdict,
        });
    }
    Ok(results)
}
```

ตรงนี้จะเห็น borrow และ clone ทำงานร่วมกันอย่างตั้งใจ `drugs: &[String]` ถูกยืม ส่วน `drug.clone()` คือการคัดลอกจริงหนึ่งก้อน เพราะ `DrugCheckResult` ต้อง **เป็นเจ้าของ** term ของตัวเอง (มันจะถูกส่งข้าม IPC กลับไปฝั่ง UI) ถ้าไม่ clone จะเป็นการย้ายค่าออกจาก slice ที่เรายืมอยู่ ซึ่งทำไม่ได้

อีกจุดที่เห็นว่า API ที่รับ `&str` ยืดหยุ่นกว่าคือ **deref coercion**: AllerX ส่ง `&String` เข้า parameter `&str` ได้ตรง ๆ

`src-tauri/src/commands.rs`

```rust
let result = repo.search_patients(&term, detect_query_kind(&term)).await;
```

`term` เป็น `String` แต่ `search_patients` รับ `&str` เราแค่ส่ง `&term` ก็พอ Rust จะลดรูป `&String` ให้เป็น `&str` เองโดยอัตโนมัติ ไม่ต้องเรียก `.as_str()` หรือ `term.as_ref()` ให้ยุ่งยาก นี่คือเหตุผลที่ The Rust Book แนะนำให้ฟังก์ชันรับ `&str` แทน `&String` เพราะรับได้ทั้งสองแบบนั่นเอง

> **`&str` กับ `String` เลือกอันไหน?**
> ถ้าฟังก์ชันแค่อ่านข้อความ ให้รับ `&str` (หรือ `&[String]`/`&[T]` สำหรับ collection) เพราะยืมได้ทั้งจาก `String` และจาก literal ทำให้ API ยืดหยุ่นที่สุด เก็บ `String` ไว้เฉพาะเมื่อต้องเป็นเจ้าของจริง ๆ เช่นเก็บลง struct หรือส่งข้ามเธรด

---

## 11. Ownership กับการทำงานพร้อมกัน: ทำไม `check_drugs` ต้อง `clone`

นี่คือส่วนที่ Ownership ส่งผลกับงานจริงมากที่สุด การที่ AllerX ตรวจยาทีละหลายตัวพร้อมกัน (batch) ทำให้เจอข้อกำหนดที่เข้มขึ้น

`crates/hosxp-connector/src/repository.rs`

```rust
async fn check_drugs(
    &self,
    hn: &str,
    drugs: &[String],
) -> Result<Vec<DrugCheckResult>, RepositoryError> {
    let mut tasks = tokio::task::JoinSet::new();
    for drug in drugs {
        let repo = self.clone();
        let hn = hn.to_string();
        let drug = drug.clone();
        tasks.spawn(async move {
            let verdict = repo.fetch_drug_history(&hn, &drug).await;
            (drug, verdict)
        });
    }
    // ...
}
```

ทำไมต้อง clone ทั้ง `repo`, `hn`, `drug`? เพราะ `tasks.spawn` ต้องการ closure ที่มีอายุ **`'static`** และ **`Send`** นั่นหมายความว่าทุกอย่างที่ async task ใช้ต้องเป็นเจ้าของตัวเองทั้งหมด task อาจรันต่อหลังฟังก์ชัน `check_drugs` จบไปแล้ว จึงห้ามยืม `&self`, `&hn`, `&drug` ที่เป็นของ scope ปัจจุบัน

- `self.clone()` ราคาถูกมาก เพราะ `MySqlPool` ข้างในเป็น `Arc` แค่เพิ่ม reference count
- `hn.to_string()` และ `drug.clone()` เป็นการคัดลอก `String` จริง เพราะทั้งสองต้องอยู่รอดข้าม task

นี่คือเหตุผลที่ Rust ต้องมี ownership ที่เข้มงวด ถ้าปล่อยให้ task ยืมข้อมูลของ scope หลักได้ แล้ว scope หลักจบไปก่อน task ก็จะกลายเป็น dangling pointer ในทันที Ownership + `'static` จึงเป็นการรับประกันที่คอมไพเลอร์ตรวจให้ก่อนรันจริง

ฝั่ง frontend ก็เจอแบบเดียวกันเวลาส่งค่าข้าม async boundary

`app/src/components/drug_search.rs`

```rust
let hn = patient.hn.clone();
leptos::task::spawn_local(async move {
    match api::check_history(&hn, &terms).await {
        // ...
    }
});
```

`patient` ถูกอ่านจาก signal (ได้สำเนามาแล้ว) แต่ `hn` ต้องถูกคัดลอกเป็น `String` ของตัวเองเพื่อ **move เข้า async block** ที่อาจจบทีหลัง

> **ทำไม `terms` ถูกส่งเป็น `&[String]` แต่ตอนเข้าหน้าต่าง async ต้องเป็นเจ้าของ?**
> เพราะ reference ใช้ได้เฉพาะตอนที่เจ้าของยังมีชีวิตอยู่ แต่ async task อาจถูกพักและไปรันต่อตอนที่เจ้าของตายไปแล้ว Rust จึงบังคับให้ข้อมูลที่ต้องข้ามขอบเขตแบบนี้เป็น "เจ้าของตัวเอง" 

---

## 12. ข้อยกเว้นที่ตั้งใจ: Shared Ownership ด้วย `Arc` และ `Rc`

ถึงตรงนี้เราบอกว่า "มีเจ้าของได้ทีละคน" แต่ในโลกจริงมีบางกรณีที่เราต้องการให้หลายที่แชร์ค่าก้อนเดียวกัน นั่นคือที่มาของ smart pointer อย่าง `Arc` และ `Rc`

`src-tauri/src/state.rs`

```rust
/// `Arc<Mutex<…>>` (not `Mutex<…>` directly): `tokio::sync::Mutex` has no
/// `Clone` impl, and both the startup/monitor tasks and command closures
/// need cheap owned handles into the same state.
#[derive(Clone)]
pub struct AppState {
    pub config_dir: PathBuf,
    pub pool: Arc<Mutex<Option<MySqlPool>>>,
    pub stats: Arc<Mutex<QueryStats>>,
    pub health: Arc<Mutex<ConnectionHealth>>,
}
```

`Arc<T>` คือ reference-counted pointer แบบ thread-safe (Atomic Rc) แต่ละ `clone()` ไม่ได้คัดลอกข้อมูล แต่เพิ่มเลข refcount ขึ้นหนึ่ง ข้อมูลจริงจะมีเจ้าของรวมกันหลายที่ และจะถูก drop จริง ๆ เมื่อ refcount กลับเป็นศูนย์

ทำไมยังถือว่าปลอดภัย? เพราะ `Arc` บังคับให้ข้อมูลที่แชร์ต้องแก้ผ่าน `Mutex` เท่านั้น ในโค้ดจะเห็นการล็อกก่อนแตะทุกครั้ง

`src-tauri/src/commands.rs`

```rust
stats.lock().await.record(
    command,
    started.elapsed().as_millis() as u64,
    result.is_ok(),
);
```

การที่ Rust บังคับใช้กฎ ownership ตรงนี้ก็เพื่อให้เราไม่ต้องพึ่ง GC หรือวินัยในการล็อกด้วยมือ คอมไพเลอร์รู้ว่า `Arc<Mutex<T>>` ถูกใช้อย่างไร และยอมให้โค้ดผ่านได้อย่างปลอดภัย

ฝั่ง frontend (single-thread, WASM) ใช้ `Rc` ซึ่งเป็นเวอร์ชันไม่ atomic และใช้ `RefCell` สำหรับ interior mutability

`app/src/lib.rs`

```rust
type PollLoop = Rc<RefCell<Option<Rc<dyn Fn()>>>>;
```

```rust
let poll_loop: PollLoop = Rc::new(RefCell::new(None));
let trigger = {
    let poll_loop = Rc::clone(&poll_loop);
    Rc::new(move || {
        poll_health();
        let next = Rc::clone(
            poll_loop
                .borrow()
                .as_ref()
                .expect("invariant: trigger is set before first poll"),
        );
        let _ = set_timeout_with_handle(move || next(), HEALTH_POLL_INTERVAL);
    })
};
```

จะเห็น `Rc::clone(&poll_loop)` แทน `poll_loop.clone()` ที่มือใหม่มักสับสน ทั้งสองอย่างนี้เรียก method เดียวกัน แต่สำนวน `Rc::clone(&x)` สื่อเจตนาว่า **"เพิ่ม refcount ไม่ได้คัดลอกข้อมูล"** เป็นธรรมเนียมที่ Rust community แนะนำ

ส่วน `RefCell` เปิดทางให้แก้ข้อมูลที่ยืมอยู่ได้ (interior mutability) โดยย้ายการตรวจ borrow จากตอนคอมไพล์มาเป็นตอนรันแทน ถ้าละเมิดจะ panic ทันที ซึ่งยอมรับได้ใน UI ที่ทำงานบนเธรดเดียว

---

## 13. สรุปทั้งหมดในตารางเดียว

| แนวคิด | อยู่ที่ไหนใน AllerX | บทเรียน |
|---|---|---|
| Move | `merge_drug_history`, `classify_drug_resolution` | ส่ง `Vec`/struct เข้าฟังก์ชันคือย้าย ownership ตัวเก่าใช้ต่อไม่ได้ |
| Use after move (E0382) | error จริงจาก `rustc 1.98` | คอมไพเลอร์ชี้จุด move และจุดที่ใช้ผิด พร้อมทางแก้ |
| Drop / RAII | `SecretString` ใน `HosxConfig`, `MySqlPool` ใน `AppState` | รหัสผ่านถูก zeroize, pool ถูกปิด เมื่อเจ้าของหลุด scope |
| Copy | `VisitType`, `QueryKind`, `ConnectionHealth` | ค่าที่อยู่บน stack ล้วน ๆ คัดลอกได้ถูกและเงียบ ไม่มี heap ให้เป็นเจ้าของ |
| Clone | `drug.clone()`, `self.clone()`, `snapshot()` | เป็นสัญญาณว่ามีต้นทุน ต้องจ่ายอย่างตั้งใจ |
| Borrow `&T` | trait `&self`, `term: &str`, `params: &[&str]` | อ่านโดยไม่ย้าย ownership ยืมซ้อนแบบ immutable ได้หลายตัว |
| Mutable borrow `&mut T` | `QueryStats::record`, `chips.update(...)` | ได้อย่างใดอย่างหนึ่ง กัน data race ตั้งแต่ตอนคอมไพล์ |
| Borrow conflict (E0502) | error จริงจาก `rustc 1.98` | `Vec::push` อาจย้ายข้อมูล ทำให้ reference เดิม dangling |
| Lifetime / dangling | `strip_leading_comments`, `dangle()` | reference ต้องมีอายุไม่สั้นกว่าข้อมูลที่มันชี้ไป |
| Slice | `rest[..keyword_end]`, `&[&str]`, `&[String]` | pointer + length ไม่มี ownership ราคาถูก |
| Concurrency | `check_drugs` spawn + `'static` | task ที่รันข้าม scope ต้องเป็นเจ้าของข้อมูลเอง |
| Shared ownership | `Arc<Mutex<...>>`, `Rc<RefCell<...>>` | หลายเจ้าของได้อย่างปลอดภัยด้วย refcount + lock |

## Checklist ตอนเจอ error ของ borrow checker

1. **"moved value"** - ถามตัวเองว่าฟังก์ชันนั้นจำเป็นต้องเป็นเจ้าของจริงไหม ถ้าไม่ เปลี่ยน parameter เป็น `&T` หรือ `&[T]`
2. **"cannot borrow as mutable because it is also borrowed"** - หา reference ตัวที่ยังมีชีวิตอยู่ แล้วปิด scope มันให้จบก่อน หรือ clone ออกมาก่อน
3. **"does not live long enough"** - ข้อมูลที่คืนหรือส่งข้าม scope ต้องมีเจ้าของที่อายุยืนพอ อาจต้องคืน owned value แทน reference
4. **อยากใช้ค่าต่อหลังส่งเข้าไปแล้ว** - อย่าเพิ่ง clone ให้พิจารณายืมก่อน ถ้าจำเป็นจริง ๆ ค่อย clone อย่างตั้งใจ
5. **ส่งข้อมูลเข้า async task** - ต้องเป็นเจ้าของเองทั้งหมด เพราะ task อาจรันหลัง scope เดิมจบ

---

## ปิดท้าย

Ownership ฟังดูเป็นกำแพงตอนแรก แต่พอเห็นมันทำงานในโค้ดจริง จะเข้าใจว่ามันไม่ได้มีไว้ให้เขียนยาก แต่มีไว้ทำให้ **ความปลอดภัยและประสิทธิภาพเป็นสิ่งที่พิสูจน์ได้ตั้งแต่ตอนคอมไพล์** แทนที่จะไปเจอบั๊กเอาตอนผู้ป่วยนอนอยู่บนเตียง

สามกฎของมันเรียบง่ายมาก ทวนอีกครั้ง

1. ทุกค่ามีเจ้าของหนึ่งคน
2. ณ เวลาหนึ่ง มีเจ้าของได้เพียงคนเดียว
3. เจ้าของหลุด scope → ค่านั้นถูก drop

แต่เมื่อกฎเรียบง่ายเหล่านี้บวกกับ borrow checker มันกลายเป็นระบบที่ทำให้ AllerX เก็บรหัสผ่านฐานข้อมูลได้อย่างปลอดภัย ตรวจยาแบบขนานได้โดยไม่มี data race และส่งข้อมูลผู้ป่วยข้ามชั้นไปมาได้โดยไม่มี dangling pointer และทั้งหมดนี้ **ไม่มี garbage collector แม้แต่ตัวเดียว**

ถ้าอยากปูพื้นฐานเรื่องตัวแปรและ memory ให้แน่นกว่านี้ก่อน กลับไปอ่าน [Rust Variables: Deep Dive](/post/rust-variables-deep-dive) ได้ครับ รวมกับบทความนี้จะได้ภาพ Ownership ที่ครบถ้วน

---

**แหล่งอ้างอิงที่ใช้ในบทความนี้**

- [The Rust Book - ch. 4: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [The Rust Book - ch. 4.1: What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [The Rust Book - ch. 4.2: References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [The Rust Book - ch. 4.3: The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html)
- [error E0382 - borrow of moved value](https://doc.rust-lang.org/error_codes/E0382.html)
- [error E0502 - cannot borrow as mutable because also borrowed as immutable](https://doc.rust-lang.org/error_codes/E0502.html)
- [Rust Variables: Deep Dive](/post/rust-variables-deep-dive)
- โค้ดอ้างอิงจากโปรเจกต์ [AllerX](https://github.com/suradet-ps/allerx) (`crates/models`, `crates/search-core`, `crates/hosxp-connector`, `src-tauri`, `app`)
