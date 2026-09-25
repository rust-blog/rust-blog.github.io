---
title: "rust-guide: คู่มือเขียน Rust ให้ปลอดภัยฉบับ ANSSI หน่วยงานความมั่นคงไซเบอร์ฝรั่งเศส"
date: "2026-09-25"
description: "แนะนำ Secure Rust Guidelines หนังสือ 60 ข้อแนะนำด้านความปลอดภัยจาก ANSSI"
tags: [rust, security, opensource]
author: "suradet-ps"
---

Rust ขึ้นชื่อว่าเป็นภาษาที่ปลอดภัยเรื่อง memory safety ตั้งแต่ระดับ type system แต่คำว่า "ปลอดภัยเพราะตัวภาษา" กับคำว่า "ปลอดภัยพอสำหรับงานที่มีข้อกำหนดความมั่นคงสูง" เป็นคนละเรื่องกันครับ โค้ด Rust ยัง panic ได้, overflow แบบเงียบ ๆ ได้, leak หน่วยความจำได้, เปิดช่องด้วย `unsafe` ได้ และพังเพราะ FFI ได้ทั้งนั้นครับ

วันนี้เลยจะมาแนะนำหนังสือที่ตอบโจทย์นี้โดยตรง นั่นคือ **rust-guide** หรือชื่อเต็มว่า *Secure Rust Guidelines* จาก **ANSSI** หน่วยงานความมั่นคงไซเบอร์แห่งชาติของฝรั่งเศส (Agence nationale de la sécurité des systèmes d'information) ครับ

## rust-guide คืออะไร

- **ชื่อเต็ม**: Secure Rust Guidelines (สถานะบนหน้าปกขึ้นว่า unstable)
- **เจ้าของ**: องค์กร [ANSSI-FR](https://github.com/ANSSI-FR) บน GitHub โดยเป็นเอกสารระดับ doctrinal ของหน่วยงาน ไม่ใช่โปรเจกต์ส่วนตัวครับ
- **เป้าหมาย**: รวมคำแนะนำสำหรับพัฒนาแอปพลิเคชันด้วย Rust ที่มีข้อกำหนดด้านความปลอดภัยสูง
- **ไม่ใช่หนังสือสอนเขียน Rust**: ตัวบทระบุชัดว่าไม่ได้ตั้งใจเป็นคอร์สเรียนภาษา แต่เป็นคู่มือชี้จุดที่คนเขียนมักพลาด โดยคนที่เขียน Rust คล่องแล้วสามารถอ่านเฉพาะกล่อง *Rule* / *Recommendation* ที่ไฮไลท์ไว้ก็ได้ครับ
- **มีสองภาษา**: อังกฤษกับฝรั่งเศส โดยโครงสร้างและรหัสคำแนะนำตรงกันทุกข้อ
- **สัญญาอนุญาต**: Open Licence 2.0 (Etalab) แบบเดียวกับข้อมูลเปิดภาครัฐของฝรั่งเศส ใช้เชิงพาณิชย์ได้ แต่ต้องให้เครดิตและระบุวันอัปเดตล่าสุดของแหล่งที่มา

ถึงแม้แท็กเวอร์ชันล่าสุดจะหยุดอยู่ที่ v1.0 เมื่อปี 2020 แต่ตัว repo ยังขยับต่อเนื่อง อัปเดตล่าสุดที่ผม clone มาคือเมื่อ พ.ค. 2026 และในปี 2025 มีคอมมิตเข้าไปกว่า 120 ครั้งด้วยกัน ถือเป็นเอกสารที่ยังมีชีวิตและประกาศตัวเองว่าเป็น living document ครับ 

## โครงสร้างของหนังสือเล่มนี้

หนังสือแบ่งเนื้อหาตามวงจรการพัฒนาซอฟต์แวร์ (development lifecycle) ไม่ได้เรียงตามหัวข้อภาษาแบบหนังสือทั่วไป

- **Lifecycle**
  - Development environment - เครื่องมือและสภาพแวดล้อมการคอมไพล์
  - Libraries - การเลือกและตรวจสอบ dependency จากภายนอก
- **Language**
  - Naming - ธรรมเนียมการตั้งชื่อตาม Rust API Guidelines
  - Integer operations - พฤติกรรม overflow และวิธีจัดการ
  - Error handling - `Result`, การ panic และแหล่งที่มาของ panic
  - Language guarantees - UB คืออะไร และ Rust รับประกันอะไรให้เราบ้าง
  - Unsafe Rust
    - Generalities - ความหมายของ `unsafe` ทั้งสองบทบาท
    - Memory management - leak, `ManuallyDrop`, raw pointer, `MaybeUninit`
    - Foreign Function Interface - บทที่ใหญ่ที่สุดของเล่ม
- **Ecosystem**
  - Standard library - `Send`/`Sync`, comparison traits, `Drop`

## Rule กับ Recommendation ต่างกันยังไง

จุดที่ทำให้หนังสือเล่มนี้ต่างจากหนังสือทั่วไปคือระบบคำแนะนำครับ ทุกคำแนะนำถูกเขียนเป็นกล่อง HTML ที่มี ID เฉพาะ และแบ่งเป็นสองระดับ

- **Rule** ใช้คำว่า *must* เป็นข้อบังคับสำหรับงานที่ต้องการความมั่นคงสูง
- **Recommendation** ใช้คำว่า *should* เป็นข้อที่ควรทำเป็นพิเศษเมื่อต้องการความมั่นคงระดับสูงขึ้นไป

ทั้งเล่มมีคำแนะนำที่แสดงผลจริง **60 ข้อ** แบ่งเป็น Rule 46 ข้อ และ Recommendation 14 ข้อ กระจายตามบทต่าง ๆ ดังนี้

- FFI 21 ข้อ
- Standard library 9 ข้อ
- Memory management 9 ข้อ
- Development environment 8 ข้อ
- Libraries 4 ข้อ
- Error handling 4 ข้อ
- Unsafe generalities 3 ข้อ
- Naming 1 ข้อ
- Integer operations 1 ข้อ

ไฮไลท์อีกอย่างคือ **checklist ท้ายเล่ม** ถูก generate อัตโนมัติจากตัวบท ไม่ใช่เขียนมือ ทำให้ไม่มีทางที่ checklist จะตกหล่นหรือไม่ตรงกับคำแนะนำจริง ๆ ในเล่ม

## ไฮไลท์ที่คนเขียน Rust ควรรู้

### สภาพแวดล้อมการพัฒนา

หนังสือเริ่มตั้งแต่ rustup ว่าดาวน์โหลดผ่าน HTTPS แต่ยังไม่มีลายเซ็นกำกับ จึงมีข้อจำกัดเรื่องการป้องกัน downgrade หรือ MitM พร้อมเตือนเรื่อง Cargo ที่ใช้โมเดล TOFU (trust on first use) กับการดาวน์โหลดจาก crates.io ครั้งแรก รวมถึงเรื่องที่หลายคนมองข้าม คือนาฬิกา `debug-assertions` กับ `overflow-checks` ใน profile ต่าง ๆ ที่ไม่ควรไป override ทิ้ง

นอกจากนี้ยังมีข้อควรระวังเวลาใช้เครื่องมือช่วยแบบอัตโนมัติ ทั้ง `rustfmt`, `clippy` และ `cargo fix` โดยเฉพาะ `cargo fix --edition-idioms` ที่อาจเปลี่ยน semantics ของโค้ดได้ จึงต้องตรวจทุกครั้งก่อน commit

### Libraries

ไลบรารีบุคคลที่สามต้องผ่านการตรวจสอบ (vetting) ทั้ง dependency ทางตรงและทางอ้อม โดยใช้ `cargo-outdated` เช็กของเก่า และ `cargo-audit` เทียบกับฐานข้อมูลช่องโหว่ RustSec ครับ

### Integer overflow

Rust ยัง overflow ได้ และพฤติกรรมขึ้นกับ profile ด้วย คือโหมด `dev` จะ panic แต่โหมด `release` จะ wrap แบบเงียบ ๆ หนังสือจึงออก Rule ให้เลี่ยงการใช้ operator ตรง ๆ แล้วหันไปใช้เมธอดที่มีชื่อบอกเจตนาชัดเจนแทน

```rust
let x: u8 = 255;

x.checked_add(1)     // None
x.overflowing_add(1) // (0, true)
x.wrapping_add(1)    // 0
x.saturating_add(1)  // 255
```

พร้อมตัวห่ออย่าง `Wrapping<T>` และ `Saturating<T>` สำหรับกรณีที่อยากให้ type บอกพฤติกรรมไปในตัว

### Error handling และ panic

`Result` คือกลไกหลักในการจัดการข้อผิดพลาด และต้องไม่ถูกเมินทิ้ง ส่วน error type ของ crate เองควร implement `Error + Send + Sync + 'static + Display` และต้อง exception-safe ด้วย

ฝั่ง panic หนังสือกำหนด Rule ตรง ๆ ว่า ฟังก์ชันต้องไม่ panic ยกเว้นกรณีที่ผู้ใช้ละเมิดเงื่อนไขการใช้งาน และ `unwrap` / `expect` / `assert!` ให้ใช้เฉพาะกรณีที่เป็นการละเมิดสัญญาเท่านั้น แหล่ง panic ที่มักถูกมองข้ามได้แก่ การ index array แบบไม่ตรวจ, overflow ในโหมด debug, การหารด้วยศูนย์, การจัดสรรหน่วยความจำก้อนใหญ่ และ `format!`

มีเกร็ดน่าสนใจด้วยว่าในงาน safety-critical การตั้ง `panic = 'abort'` ให้หยุดทันทีถือเป็นตัวเลือกที่ชอบธรรม เช่นในระบบควบคุมการบิน หยุดก่อนดีกว่าปล่อย state ที่เสียแล้วแพร่ไปยังระบบสำรอง

### Unsafe Rust

บทนี้แยกบทบาทของ `unsafe` ได้ดีมาก คือใช้ *marking* เพื่อบอกว่าผู้เรียกต้องรับผิดชอบเอง (ใน `unsafe fn` และ `unsafe trait`) กับใช้ *unlocking* เพื่อรับผิดชอบแทนคอมไพเลอร์ (`unsafe` block, `unsafe impl`, และใน Rust 2024 เพิ่ม `extern` block กับ attribute อย่าง `no_mangle`)

หลักคิดคือ "โค้ดที่ไม่มี unsafe เรียกผิดไม่ได้" จึงมี Rule ให้เลี่ยง unsafe หรือถ้าจำเป็นต้องใช้จริง (FFI, เข้าถึง register ของ embedded, หรือชนกำแพงประสิทธิภาพที่วัดแล้ว) ต้องอธิบายเหตุผลให้ได้ ไม่งั้นใส่ `#![forbid(unsafe_code)]` ไปเลย และทุก unsafe ต้องถูกห่อ (encapsulate) ให้ข้างนอกเห็นเป็น API ที่ปลอดภัยหรือมี precondition ที่เขียนไว้ครบ

หนังสือยกตัวอย่างคลาสสิกของ `Vec` แบบบ้าน ๆ ที่ invariant เรื่อง allocation ถูกทำลายได้ด้วยโค้ด safe ล้วน ๆ ถ้าเผลอเปิด method อย่าง `make_room` ออกมาให้ภายนอกใช้ นี่คือตัวอย่างที่อธิบายได้ดีมากว่าทำไม encapsulation ถึงสำคัญ

### Memory และ leak

- Rule แรกของบทคือ **ห้าม leak หน่วยความจำ**
- `mem::forget` ห้ามใช้ เพราะถึงจะ memory-safe แต่ก็ไม่ secure (destructor ไม่ทำงาน, resource ค้าง, ความลับอาจค้างอยู่ใน RAM)
- `Box::leak` ก็เข้าข่ายห้ามเช่นกัน
- ค่าที่ห่อด้วย `ManuallyDrop` ต้องปล่อยคืนให้ได้
- `into_raw` ต้องมี `from_raw` มารับคู่กันเสมอ และ `from_raw` ห้ามเรียกกับค่าที่ไม่ได้มาจาก `into_raw`
- `mem::uninitialized` เลิกใช้แล้ว ส่วน `MaybeUninit` ทุกครั้งต้องมีเหตุผลกำกับ

ปิดท้ายด้วยตัวอย่างที่หนังสือใช้เตือนสติ นั่นคือ `Rc<RefCell<T>>` แบบวนกลับมาถึงตัวเอง ซึ่งเป็น memory leak ที่เกิดในโค้ด safe Rust ล้วน ๆ โดยไม่ต้องมี `unsafe` สักบรรทัด หนังสือรันโปรแกรมตัวอย่างผ่าน valgrind แล้วได้ผลลัพธ์ออกมาดังนี้

```text
$ valgrind --leak-check=full target/release/safe-rust-leak

==153637== HEAP SUMMARY:
==153637==     in use at exit: 48 bytes in 2 blocks
==153637==   total heap usage: 10 allocs, 8 frees, 3,144 bytes allocated
==153637==
==153637== 48 (24 direct, 24 indirect) bytes in 1 blocks are definitely lost
==153637==
==153637== LEAK SUMMARY:
==153637==    definitely lost: 24 bytes in 1 blocks
==153637==    indirectly lost: 24 bytes in 1 blocks
==153637== ERROR SUMMARY: 1 errors from 1 contexts
```

จากผลนี้ หนังสือจึงตั้ง Rule ไว้ชัดเจนว่า type แบบ recursive ที่ใช้ reference counted pointer **ห้ามใช้ร่วมกับ interior mutability** เพราะนอกจากหน่วยความจำจะรั่วแล้ว ยังเปิดทางไปสู่การโจมตีแบบ DDoS หรือทำให้ข้อมูลลับค้างอยู่ในหน่วยความจำได้อีกด้วย

### FFI บทที่ใหญ่ที่สุดของหนังสือเล่มนี้

ด้วยคำแนะนำ 21 ข้อ บท FFI ครอบคลุมตั้งแต่

- ใช้เฉพาะ type ที่เข้ากันได้กับ C และใช้ alias `c_*` เพื่อความ portable
- ระวัง **non-robust type** โดยเฉพาะ `bool` ที่มี 256 รูปแบบบิตแต่ถูกต้องแค่ 2 แบบ ถ้าโดนยัดค่าจากฝั่ง C ตรง ๆ อาจกลายเป็น trap representation
- ตรวจ pointer จากภายนอกก่อน dereference ทุกครั้ง (null, ช่วงที่ถูกต้อง, alignment)
- ห้ามรับค่า `enum` ของ Rust จากฝั่ง foreign ตรง ๆ เพราะเสี่ยง type confusion ให้ใช้ integer แล้วแปลงแบบตรวจสอบแทน
- ตัวอย่าง UB จาก memory model ของ Rust ที่อ่านสนุกมาก เช่น C ฟังก์ชัน `swap` ที่รับ `&mut` สองตัวมาสลับกันจนเกิด aliasing หรือ `inc_wrap` ที่ mutate ผ่าน `&` ฝั่ง Rust
- กฎเหล็กเรื่อง ownership: **ให้ภาษาเดียวเป็นเจ้าของทั้ง allocation และ deallocation**
- ห้ามส่ง type ที่ implement `Drop` ข้าม FFI แบบ by value
- ฝั่ง Rust ห้าม panic ข้ามขอบ FFI ถ้าหลีกเลี่ยงไม่ได้ให้ใช้ `catch_unwind` (และจำว่ามันไม่จับ `panic = 'abort'`)

พร้อมแนะนำเครื่องมือ `bindgen` / `cbindgen` และแพตเทิร์นการแยก crate เป็น `-sys` ที่เป็น binding ดิบ กับ crate ห่อที่ปลอดภัย

### Standard library

- `Send` / `Sync` เป็น unsafe marker trait การ implement เองผิดคือ UB ตรง ๆ และโชว์เทคนิค `PhantomData<*const ()>` สำหรับ opt out
- Comparison trait อย่าง `PartialEq` / `Ord` มี invariant ที่คอมไพเลอร์ไม่ตรวจให้ทั้งชุด ที่ต้องระวังเป็นพิเศษคือ `#[derive]` เรียงตาม **ลำดับการประกาศฟิลด์** ไม่ใช่ชื่อ ถ้าสลับฟิลด์เมื่อไหร่ผลการเปรียบเทียบก็พลิกทันที
- `Drop`: ห้าม panic, ระวัง cycle ของ `Rc`/`Arc`, ไม่ควรใช้ `Drop` เป็นด่านสุดท้ายของงานด้านความปลอดภัยอย่างการลบคีย์ เพราะ drop ถูกข้ามได้หลายทาง (cycle, `mem::forget`, panic ตอน drop, abort)

## เบื้องหลังงานวิศวกรรมที่น่าสนใจ

หนังสือเล่มนี้ build ด้วย **mdBook fork ของ ANSSI เอง** และ preprocessor ที่เขียนเพิ่มอีกหลายตัว

- `mdbook-checklist` - ไล่เก็บกล่องคำแนะนำแล้ว generate บท checklist ให้อัตโนมัติ
- `mdbook-extensions` - รวม cite processor, ตัวสะกด (aspell) และตัวตรวจไวยากรณ์ (Grammalecte) ไว้ในตัวเดียว
- `mdbook-code-align` - จัด indent ของ code block ที่ตัดมาจากไฟล์จริง

แปลว่าเล่มนี้ต้องผ่านด่านตรวจคำผิดและไวยากรณ์ใน CI ด้วย ซึ่งอธิบายคุณภาพการเรียบเรียงที่อ่านลื่นมากได้เป็นอย่างดี

## ข้อสังเกตและช่องว่างของเล่ม

- **async Rust ยังไม่ถูกพูดถึง** ตัวบทระบุว่าไม่อยู่ในขอบเขตของเวอร์ชันนี้
- บท **Macros**, **Type system** และ **Test & Fuzzing** ยังเป็นโครงร่าง ถูกคอมเมนต์ไม่ให้ render ในสารบัญ
- เรื่อง **`as` cast และ `transmute`** แทบไม่ถูกพูดถึงเลย ซึ่งน่าเสียดายเพราะเป็นจุดที่คนพลาดบ่อย
- ชื่อเรื่องที่ขึ้นว่า *unstable* ต้องตีความว่ายังเพิ่ม/ปรับคำแนะนำได้ แต่ตัว Rule ที่มีอยู่ก็อ้างอิงหลักการที่มั่นคงดี

## ใครควรอ่าน

ถ้าเขียน Rust ใช้งานจริงอยู่แล้ว หนังสือเล่มนี้จะช่วยปิดช่องที่ compiler จับให้ไม่ได้ และถ้าทำงานในโดเมนที่ต้องการความมั่นคงสูง เช่น security tooling, embedded, ระบบการเงิน หรือระบบที่คุยกับ C/C++ ผ่าน FFI ยิ่งควรอ่านไล่ทีละบทครับ

วิธีอ่านที่ผมแนะนำคือ เริ่มจากบท Language guarantees กับ Unsafe Rust เพื่อปรับ mental model ให้ตรงก่อน แล้วค่อยไล่บทอื่น ๆ ตามงานที่ทำอยู่ สุดท้ายเปิด checklist ท้ายเล่มติ๊กเป็นรายการตรวจก่อนปล่อยงาน

rust-guide เป็นหนังสือคุณภาพที่หาได้ยาก เพราะเขียนจากมุมของหน่วยงานรัฐด้านความมั่นคงที่ต้องใช้งาน Rust จริง ไม่ใช่จากมุมคนสอนภาษา เนื้อหาจึงเน้น "อะไรพังได้ แม้ใน safe Rust" มากกว่า "ภาษา Rust ดีอย่างไร" และด้วยระบบ Rule/Recommendation พร้อม ID เฉพาะ ทำให้ใช้อ้างอิงใน review หรือ checklist ของทีมได้ทันที

ใครสนใจสามารถอ่านออนไลน์หรือ clone ไปศึกษาได้เลยครับ

- [อ่านออนไลน์ (ภาษาอังกฤษ)](https://anssi-fr.github.io/rust-guide)
- [อ่านออนไลน์ (ภาษาฝรั่งเศส)](https://anssi-fr.github.io/rust-guide/fr/)
- [ซอร์สบน GitHub](https://github.com/ANSSI-FR/rust-guide)

ถ้าใครอ่านจบแล้วอยากลองเอา Rule สักข้อไปใช้ในโปรเจกต์ตัวเอง ก็ลองเริ่มจากเรื่อง integer overflow กับ panic ดูครับ เพราะเป็นสองจุดที่เจอบ่อยที่สุดในโค้ดจริง และแก้ไม่ยากด้วย
