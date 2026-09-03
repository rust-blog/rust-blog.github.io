---
title: "ลด binary 4 เท่าและไขปริศนา target บวม การทดลองวัดผลจริงจากโปรเจคต์ med-recon"
date: "2026-09-03"
description: "บันทึกผลการทดลองปรับแต่ง build configuration เพื่อลดขนาดโฟลเดอร์ target/ และ binary ของโปรเจกต์ Med Recon (Tauri 2 + Leptos 0.8 CSR)"
tags: [rust, tauri, leptos, wasm]
author: "suradet-ps"
---

บทความนี้บันทึกผลการทดลองปรับแต่ง build configuration เพื่อลดขนาดโฟลเดอร์ `target/` และไฟล์ผลลัพธ์ (artifacts) ของโปรเจกต์ **Med Recon** ([github.com/suradet-ps/med-recon](https://github.com/suradet-ps/med-recon)) (พัฒนาด้วย Tauri 2 + Leptos 0.8 CSR workspace) โดยตัวเลขทั้งหมดวัดจากการทำงานจริงบนเครื่องจริง ไม่ใช่การประเมินจากตัวเลขสมมติ (ดูข้อมูลดิบได้ที่ `docs/size-experiments/measurements.md`)

---

## 1. สภาพแวดล้อมที่ใช้ทดสอบ

| รายการ | ค่า |
|---|---|
| เครื่องที่ใช้ | macOS aarch64 (Apple Silicon) |
| rustc / cargo | 1.98.0 stable |
| trunk | 0.21.14 |
| wasm-opt (binaryen) | 132 (ติดตั้งผ่าน Homebrew) |
| เวลาในการ build | native ~1 นาที 40 วินาที, wasm ~50 วินาที (ครั้งแรก) |

โครงสร้างของ Workspace ประกอบด้วย 4 crates (`med-recon-core`, `med-recon-hosxp`, `med-recon-config`, `med-recon-bridge`) และ 2 apps (`med-recon-app` native, `med-recon-app/frontend` คอมไพล์เป็น `wasm32-unknown-unknown` ผ่าน trunk)

---

## 2. วิธีการวัดผล (Methodology)

1. รันคำสั่ง `cargo clean` ก่อนเริ่มการทดลองแต่ละชุด เพื่อให้โฟลเดอร์ `target/` ว่าง 100%
2. ทำการ build ใหม่ทั้งหมดด้วย configuration ที่กำหนดเพียงชุดเดียว
3. วัดขนาดไฟล์จริงด้วยคำสั่ง `du -sh target` และ `ls -lh` บนไฟล์ binary/wasm

> **เหตุผลที่ต้อง clean ทุกครั้ง** คือ Cargo จะเก็บ artifact ของ flags แต่ละชุดที่เคย build เอาไว้ทั้งหมด (เมื่อ hash ต่างกัน ไฟล์ก็จะแยกจากกัน) หากไม่ clean ตัวเลขขนาดไฟล์จะสะสมปะปนกันจนไม่สามารถสรุปผลที่ถูกต้องได้ และนี่คือสาเหตุหลักที่ทำให้โฟลเดอร์ `target/` มีขนาดใหญ่ผิดปกติ (อ่านต่อในหัวข้อที่ 6)

---

## 3. ค่าเริ่มต้น (Baseline จาก Cargo โดยไม่มี Profile ใดๆ)

| ตัวชี้วัด (Metric) | ขนาด |
|---|---|
| `target/` ทั้งหมด | **1.7G** |
| `target/release` (deps 1.3G + build 131M) | 1.4G |
| `target/wasm32-unknown-unknown` (deps 307M) | 312M |
| native binary `med-recon-app` | **15M** |
| wasm ก่อนทำ wasm-opt (ใน target) | 2.8M |
| wasm หลังทำ wasm-opt (ใน dist) | 2.6M |

**ข้อสรุปช่วงเริ่มต้น** คือ โฟลเดอร์ `target/` กว่า **~95% เป็น cache ของ dependencies (`.rlib`)** ไม่ใช่ขนาดของตัว binary แต่อย่างใด

---

## 4. ขั้นตอนการทดลอง (ปรับทีละขั้น วัดผลทีละสเต็ป)

### การทดลอง A: ใช้ Profile ตามมาตรฐานของ Tauri (`opt-level="s"`)

อ้างอิงตาม [คู่มือแนะนำเรื่องขนาดของ Tauri v2](https://v2.tauri.app/concept/size/) ด้วย profile ต่อไปนี้

```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
panic = "abort"
strip = true
```

| ตัวชี้วัด | ก่อนปรับ | หลังปรับ |
|---|---|---|
| native binary | 15M | **5.3M** (ลดลง 65%) |
| wasm (dist) | 2.6M | 622K |

### การทดลอง B: ปรับเป็น `opt-level="z"` (เน้นบีบขนาดไฟล์ให้เล็กที่สุด)

| ตัวชี้วัด | การทดลอง A | การทดลอง B |
|---|---|---|
| native binary | 5.3M | **4.2M** |
| wasm (dist) | 622K | 511K |
| `target/` | 1.7G | 1.6G |

ระดับ `"z"` ให้ผลลัพธ์ที่ดีกว่า `"s"` ทั้งในส่วนของ native และ wasm ซึ่งเหมาะกับแอปพลิเคชันประเภทนี้ที่ไม่ได้เน้นการประมวลผลหนัก (Compute-bound)

### การทดลอง C: ทดสอบปิด wasm features ที่ rustc เปิดมาเป็นค่าเริ่มต้น

ระหว่างทดสอบพบปัญหาว่า binaryen validator **ไม่รองรับ** wasm ที่มี instruction รุ่นใหม่ (เช่น memory.copy/fill, trunc_sat) เนื่องจาก rustc 1.82+ เปิดใช้งาน `bulk-memory` และ `nontrapping-fptoint` มาเป็นค่าเริ่มต้นบน wasm32 แต่ binaryen validator ยังคงยึดตามมาตรฐาน WebAssembly MVP การลองแก้ด้วย `-C target-cpu=mvp` ก็ไม่ช่วยแก้ปัญหา เนื่องจาก **`wasm-bindgen` จะแทรกคำสั่ง `memory.copy` กลับเข้ามาเองในขั้นตอน post-process** (จากเดิม raw wasm มี 0 op แต่หลัง stage wasm เพิ่มเป็น 758 op)

### การทดลอง D (ชุดสุดท้าย) ทำ Post-processing hook + wasm-opt -Oz

แนวทางที่ใช้งานได้จริง มีดังนี้

1. ปิดระบบ wasm-opt อัตโนมัติของ trunk (`data-wasm-opt="0"` ใน `index.html`) เนื่องจาก trunk ไม่สามารถส่ง feature flags เพิ่มเติมไปยัง wasm-opt ได้
2. สั่งรัน binaryen `-Oz` เองผ่านสคริปต์ `script/wasm-opt.sh` พร้อมระบุ flags ที่จำเป็น ได้แก่
   `--enable-bulk-memory-opt --enable-nontrapping-float-to-int --enable-mutable-globals --strip-debug --low-memory-unused`
3. เชื่อมสคริปต์เข้ากับ `beforeBuildCommand` ใน `tauri.conf.json`

| ตัวชี้วัด | การทดลอง B | การทดลอง D (Final) |
|---|---|---|
| wasm (dist) | 511K (wasm-opt ทำงานล้มเหลวแบบเงียบๆ) | **477K** |

> **ข้อค้นพบสำคัญ** คือ ตัวเลข "622K / 511K" ในการทดลอง A และ B แท้จริงแล้วเป็นผลลัพธ์จาก `wasm-bindgen` **ที่ยังไม่ผ่านการ optimize ด้วย wasm-opt เลย** เนื่องจาก trunk รัน wasm-opt ล้มเหลวโดยไม่มีการแจ้งเตือน (Silent failure) ตัวเลขที่เล็กลงในตอนแรกจึงมาจาก `wasm-bindgen` ล้วนๆ การเขียน custom hook ในการทดลองนี้จึงช่วยให้ wasm-opt ได้ทำงานจริงเป็นครั้งแรกใน pipeline

---

## 5. สรุปผลลัพธ์สุดท้าย

| ตัวชี้วัด | Baseline (เริ่มต้น) | Final (สุดท้าย) | ผลต่าง |
|---|---|---|---|
| native binary | 15M | **4.2M** | **ลดลง 72%** |
| wasm (dist) | 2.6M | **477K** | **ลดลง 82%** |
| `Med Recon.app` | - | 4.5M | - |
| `.dmg` | - | 2.8M | - |
| `target/` (หลัง build เสร็จ 1 รอบ) | 1.7G | 1.6G | ลดลง 6% |

**สรุปภาพรวม** ขนาดของตัว binary เล็กลงถึง **~4 เท่า** โดยแก้เพียง build configuration โดยไม่ต้องแตะต้อง source code เลยแม้แต่บรรทัดเดียว ส่วนโฟลเดอร์ `target/` นั้นไม่ได้ลดลงตาม flags เหล่านี้อย่างมีนัยสำคัญ เพราะพื้นที่ส่วนใหญ่คือ cache ของ dependencies (sqlx + tokio + tauri + leptos + pdf libs) ซึ่งจำเป็นต้องมีไว้เพื่อให้การ rebuild ทำงานได้รวดเร็ว

---

## 6. ทำไมโฟลเดอร์ target/ ถึงมีขนาดใหญ่มาก

ปัญหาที่แท้จริงของ "target บวม" ไม่ได้มาจากไฟล์ binary แต่เกิดจากพฤติกรรมของ Cargo โดยผลการวัดจริงเป็นดังนี้

| สถานะการทำงาน | ขนาดของ target |
|---|---|
| Build เสร็จสมบูรณ์ 1 config | 1.6G (ขนาดไฟล์ cache พื้นฐานที่โปรเจกต์นี้ต้องใช้) |
| ทำการ Release + Bundle (`cargo tauri build`) | 1.8G |
| มีการสลับ flags ไปมา 5 ครั้ง (โดยไม่ clean) | **2.2G** (มีไฟล์ขยะตกค้างสะสม +600M) |
| มีการ build แบบ Debug ปะปนอยู่ด้วย (เช่น `cargo test`/`cargo tauri dev`) | **4.2G** (เฉพาะ debug ก็กินพื้นที่ไป 2.5G ซึ่งใหญ่กว่า release เสียอีก) |
| สั่ง `cargo clean` | 0 (แต่การ build ใหม่ทั้งหมดต้องใช้เวลาราว ~2.5 นาที) |

**ขนาด 2.2G คือสิ่งที่นักพัฒนาจะพบเจอจริงในการทำงาน** Cargo จะไม่ลบ artifact ของ configuration ชุดเก่าให้โดยอัตโนมัติ ทุกครั้งที่มีการอัปเดต rustc, toolchain หรือเปลี่ยน flags ตัว Cargo จะสร้างและเก็บไฟล์ชุดใหม่เพิ่มเข้าไปเรื่อยๆ

แนวทางการจัดการโฟลเดอร์ target (เรียงจากระดับเบาไปหนัก)

```sh
# 1. ลบเฉพาะ artifact เก่าตามระยะเวลา (แนะนำวิธีนี้เป็นประจำ)
cargo install cargo-sweep
cargo sweep --time 30          # ลบ artifact ที่ไม่ได้แตะต้องเกิน 30 วัน
cargo sweep --installed        # ลบ artifact ของ toolchain เก่าที่เคยถอนการติดตั้งไปแล้ว

# 2. ล้างข้อมูลทั้งหมดแบบเด็ดขาด (ต้อง build ใหม่ทั้งหมด ใช้เวลา ~2.5 นาที)
cargo clean

# 3. ย้ายโฟลเดอร์ target ไปไว้ที่ไดรฟ์อื่น (ไม่ช่วยลดขนาด แต่ไม่เปลืองพื้นที่ SSD หลัก)
# กำหนดค่า [build] target-dir ลงใน .cargo/config.toml เช่น
# [build]
# target-dir = "/path/to/big/disk/target"
```

ในการทำงานจริงที่มีทั้ง Debug build และ Test build โฟลเดอร์ `target/` อาจขยายใหญ่ได้ถึง 3-5G ซึ่งถือเป็นเรื่องปกติของโปรเจกต์ระดับ Tauri + Leptos

---

## 7. รายการไฟล์ที่มีการแก้ไข (อัปเดตลง repo แล้ว)

| ไฟล์ | รายละเอียดการแก้ไข |
|---|---|
| `Cargo.toml` | เพิ่มการตั้งค่า `[profile.release]` (lto, codegen-units=1, opt-level="z", panic="abort", strip) |
| `apps/med-recon-app/frontend/index.html` | ใส่ `<link data-trunk rel="rust" data-wasm-opt="0">` เพื่อปิด wasm-opt เดิมของ trunk ที่ทำงานล้มเหลวแบบเงียบๆ |
| `apps/med-recon-app/tauri.conf.json` | เพิ่มคำสั่ง `sh ../../script/wasm-opt.sh` ต่อท้ายใน `beforeBuildCommand` |
| `script/wasm-opt.sh` | ไฟล์สคริปต์ใหม่สำหรับรัน binaryen `-Oz` พร้อม feature flags ที่ถูกต้อง |

---

## 8. ข้อควรระวังและผลกระทบ (Trade-offs)

- **`panic = "abort"`** เมื่อเกิด panic โปรแกรมจะหยุดทำงานและปิดตัวลงทันทีโดยไม่มีการ unwinding ซึ่งเหมาะกับ desktop application ทั่วไป แต่หากมีการใช้โค้ดที่ต้องการ catch panic ข้าม thread จะต้องระมัดระวัง (ในโปรเจกต์นี้ไม่ได้มีการใช้ `catch_unwind`)
- **`strip = true`** บนระบบปฏิบัติการ Linux มีบั๊กที่พบใน Tauri bundler เกี่ยวกับ `__TAURI_BUNDLE_TYPE` (issue #14186) แต่บน macOS และ Windows สามารถใช้งานได้ปกติ ไม่มีปัญหา
- **`opt-level = "z"`** ความเร็วในการประมวลผลอาจช้ากว่าการใช้ `"3"` เล็กน้อย แต่เหมาะสมมากสำหรับแอปพลิเคชันที่เน้น I/O (เช่น การ query ฐานข้อมูล, การแสดงผล UI) อย่างแอปนี้
- **`codegen-units = 1` ร่วมกับ Fat LTO** ตามทฤษฎีจะทำให้ build นานขึ้นเล็กน้อย แต่จากการวัดผลจริง native build ยังใช้เวลา ~1m40s เท่าเดิม เนื่องจากเวลาส่วนใหญ่หมดไปกับการคอมไพล์ dependencies แต่ช่วยให้ได้ขนาดไฟล์ที่เล็กที่สุด
- **wasm-opt hook จำเป็นต้องมี Binaryen ติดตั้งอยู่ในเครื่อง** หากเครื่องที่ build ไม่มี binaryen ให้ติดตั้งผ่าน `brew install binaryen` มิฉะนั้นขั้นตอน build จะ fail ทันที (ซึ่งตั้งใจให้แจ้งเตือนอย่างชัดเจน ดีกว่าปล่อยให้ trunk ข้ามการทำงานไปแบบเงียบๆ)

---

## 9. เทคนิคขั้นสูงสำหรับต่อยอดในอนาคต (บันทึกไว้ศึกษาเพิ่มเติม)

| เทคนิค | ผลลัพธ์ที่คาดว่าจะได้ | ข้อจำกัด / ต้นทุนที่ต้องแลก |
|---|---|---|
| `-Zbuild-std` + `panic_immediate_abort` (Nightly) | native ลดลงอีก ~0.5-1M, wasm ลดลงอีก ~50-100K | ต้องใช้ Rust Nightly toolchain และมีคอนฟิกที่ซับซ้อน |
| Prune features ของ dependencies ที่ไม่ได้ใช้ (เช่น ปิด `tracing-subscriber` env-filter) | ลดลงได้หลายร้อย KB | ต้องแก้ไขโค้ดและทดสอบระบบใหม่อย่างละเอียด |
| ใช้ `cargo-bloat` หรือ `cargo-llvm-lines` วิเคราะห์จุดที่กินพื้นที่แล้ว refactor | ขึ้นอยู่กับจุดที่พบในโค้ด | ต้องใช้เวลาในการพัฒนาและปรับแก้โค้ด |
| แยก `[profile.release]` สำหรับ native และ wasm ออกจากกัน (`cargo_profile` ใน trunk) | ปรับแต่งค่าของแต่ละฝั่งได้ละเอียดยิ่งขึ้น | ฟีเจอร์นี้ยังอยู่ในเวอร์ชัน trunk beta |

ข้อมูลอ้างอิงจากโปรเจกต์อื่นระบุว่า การใช้ `-Zbuild-std` ร่วมกับ `panic_immediate_abort` ช่วยลดขนาด wasm ลงได้อีกราว ~10-15% แต่ต้องแลกกับการดูแล nightly toolchain สำหรับ med-recon (wasm 477K) ขนาดในปัจจุบันถือว่าเหมาะสมและเพียงพอต่อการใช้งานจริงแล้ว

---

## 10. สรุปภาพรวม

1. **ขนาด Binary เล็กลงถึง 4 เท่าโดยไม่ต้องแก้โค้ด** โดย native ลดจาก 15M → 4.2M และ wasm ลดจาก 2.6M → 477K ด้วยการเพิ่ม config ไม่กี่บรรทัดใน `Cargo.toml` ร่วมกับ build hook เพียงไฟล์เดียว
2. **`target/` ที่มีขนาดใหญ่เกิดจาก cache ของ dependencies ไม่ใช่ปัญหาของตัว binary** โดยขนาด 1.6G คือพื้นที่ปกติสำหรับการคอมไพล์โปรเจกต์นี้ ปัญหาที่แท้จริงคือไฟล์ขยะที่ตกค้างจากการสลับ config ไปมา ซึ่งสามารถจัดการได้ง่ายๆ ด้วยการใช้ `cargo-sweep`
3. **สิ่งที่ค้นพบระหว่างการทดลอง** คือ trunk 0.21 ทำงานร่วมกับ wasm-opt ล้มเหลวแบบเงียบๆ เมื่อเจอกับ wasm มาตรฐานใหม่ (bulk-memory ที่สร้างโดย `wasm-bindgen`) โปรเจกต์ที่ใช้ Tauri/Leptos บน rustc ≥1.82 ควรนำแนวทางการใช้ custom hook นี้ไปปรับใช้
4. **ยึดการวัดผลจากเครื่องจริงเสมอ** เพราะตัวเลขจากการ "คาดคะเน" เทียบไม่ได้กับการรันคำสั่ง `du -sh` เพื่อดูขนาดไฟล์ที่เกิดขึ้นจริงในระบบ
