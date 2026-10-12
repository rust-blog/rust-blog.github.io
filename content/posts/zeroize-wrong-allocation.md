---
title: "ล้างรหัสผ่านผิดก้อน: เมื่อ zeroize ไม่เคยแตะข้อมูลตัวจริง"
date: "2026-09-28"
description: "ทำไม get_untracked() ถึงทำให้ zeroize ผิดก้อน"
tags: [rust, security, wasm]
author: "suradet-ps"
---

เรื่องนี้เกิดขึ้นจริงในโปรเจกต์ `drugitems` (Issue #19) ของผมเอง ซึ่งเป็นแอปเดสก์ท็อปสำหรับตรวจสอบข้อมูลยาแบบอ่านอย่างเดียว พัฒนาด้วย Tauri + Leptos ตัวแอปมีฟอร์มกรอกรหัสผ่าน MySQL และมีฟังก์ชันเล็ก ๆ ชื่อ `wipe_field` ที่ตั้งใจจะล้างรหัสผ่านออกจากหน่วยความจำเมื่อผู้ใช้ปิดฟอร์ม แน่นอนว่าโค้ดส่วนนี้ผู้ช่วย AI เป็นคนเขียน แถมยังใช้ `unsafe` อีกด้วย ผลที่ได้คือสิ่งที่มันไปล้างกลับเป็นแค่ **"สำเนา"** ของรหัสผ่าน ไม่ใช่ตัวจริงที่อยู่ใน RAM

ในบทความนี้ผมจะพาไล่ดู Memory ของ Rust ทีละไบต์ ตั้งแต่พื้นฐานว่า `String` เก็บข้อมูลอย่างไร ไปจนถึงการทดลองบน allocator จริงทั้ง macOS และ WASM ให้เห็นชัด ๆ ว่ารหัสผ่านรอดหรือร่วง แล้วปิดท้ายด้วยวิธีแก้ไขที่ถูกต้องตามมาตรฐานครับ

> 💡 **ถ้าไม่มีเวลาอ่านทั้งหมด นี่คือสรุปสามบรรทัด**
> 1. `signal.get_untracked()` คืนค่าเป็น **clone** ไม่ใช่ตัวจริง ดังนั้น `fill(0)` จึงไปล้างแค่ก้อนสำเนา
> 2. ก้อนหน่วยความจำตัวจริงถูก `free` คืนระบบไปโดยไม่เคยถูกเขียนทับด้วย `0` ส่วนรหัสผ่านจะยังค้างอยู่ใน RAM หรือไม่ ขึ้นอยู่กับ allocator ปลายทาง (ซึ่งกรณีนี้ WASM ของ Leptos คือผู้รอดชีวิต)
> 3. วิธีแก้ที่ถูกต้องคือใช้ crate `zeroize` / `Zeroizing` แทนการเขียน `unsafe` จัดการเอง

---

## 1. โค้ดเจ้าปัญหาอยู่ตรงนี้

นี่คือโค้ดจริงจากไฟล์ `connection_form.rs` (ตัดมาเฉพาะส่วนที่เกี่ยวข้อง)

```rust
/// Zeroize an operator-typed field before dropping it.
fn wipe_field(signal: &RwSignal<String>) {
    let mut value = signal.get_untracked();
    if !value.is_empty() {
        unsafe { value.as_mut_vec().fill(0) };
    }
    signal.set(String::new());
}

```

และนี่คือจุดที่มันถูกเรียกใช้ตอน component ถูกทำลาย (cleanup)

```rust
let password = RwSignal::new(String::new());
on_cleanup(move || wipe_field(&password));

```

**เจตนาของผมคือ:** เมื่อผู้ใช้กรอกรหัสผ่านแล้วปิดหน้าต่าง ให้เขียนทับรหัสผ่านในหน่วยความจำด้วยเลขศูนย์ (zeroize) ก่อนปล่อยคืนให้ OS เพื่อไม่ให้มีรหัสผ่านหลงเหลืออยู่ใน RAM

**แต่สิ่งที่เกิดขึ้นจริงคือ:** รหัสผ่านตัวจริงไม่เคยถูกเขียนทับเลยแม้แต่ไบต์เดียว

---

## 2. `String` อยู่ใน RAM อย่างไร (และ `RwSignal` เก็บมันไว้แบบไหน)

`String` ใน Rust ไม่ใช่ "ก้อนข้อความ" ที่ลอยอยู่ในหน่วยความจำ แต่เป็น **header ขนาดเล็กที่อยู่บน stack** ซึ่งประกอบด้วยข้อมูลสามส่วน

```text
[ Stack ]                                      [ Heap Memory ]
value (String)
┌───────────┬──────┬──────┐               ┌───┬───┬───┬───┬───┬───┐
│ ptr       │ cap  │ len  │ ────────────►│ P │ a │ s │ s │ w │ d │
└───────────┴──────┴──────┘               └───┴───┴───┴───┴───┴───┘

```

ตัวอักษรจริงจะถูกเก็บไว้ใน Heap ก้อนหนึ่ง ส่วน header บน stack มีหน้าที่บอกว่า "ก้อนข้อมูลนี้อยู่ที่ไหน จุได้เท่าไร และยาวเท่าไร"

เมื่อมาใช้กับ Leptos ตัว `RwSignal<String>` จะ **เก็บ `String` header ทั้งชุดไว้ใน reactive arena** ของมัน นั่นแปลว่า signal เป็นผู้ถือ pointer ที่ชี้ไปยัง Heap Block A ตัวจริง ไม่ใช่สำเนา

จุดที่ต้องแยกให้ออกคือ เมธอดสำหรับดึงข้อมูลจาก signal มีอยู่สองแบบครับ

| เมธอด | ผลลัพธ์ที่ได้ | ข้อสังเกต |
| --- | --- | --- |
| `signal.with(|v| ...)` / `with_untracked` | ยืม `&String` **ตัวจริง** มาใช้งาน | ใช้งานได้เฉพาะภายใน closure เท่านั้น |
| `signal.get()` / `get_untracked()` | **clone** ออกมาเป็นค่าตัวใหม่ | ข้อมูลชนิดนั้นต้องอิมพลีเมนต์ `T: Clone` |

เหตุผลที่ `get()` ต้องทำการ clone ก็เพราะมันต้องคืนค่าออกไปให้ผู้เรียกเป็นเจ้าของ (`String`) หลังจากหลุดออกจาก closure ซึ่งไม่สามารถยืมข้อมูลข้ามขอบเขต (scope) แบบนั้นได้ ราคาที่ต้องจ่ายจึงเป็น **allocation ใหม่ทุกครั้งที่เรียก**

---

## 3. มาดูการทำงานทีละบรรทัด

มาดูกันว่าเกิดอะไรขึ้นในหน่วยความจำ RAM จริง ๆ ระหว่างที่โค้ดทำงาน

### บรรทัดที่ 1: `let mut value = signal.get_untracked();`

เกิดการ `Clone` ข้อมูลขึ้นมาใหม่หนึ่งก้อน:

```text
signal ──► Heap Block A: "SuperSecretPassword123"   ← ตัวจริง (ยังคงค้างอยู่)
value  ──► Heap Block B: "SuperSecretPassword123"   ← สำเนา (allocation ก้อนใหม่)

```

### บรรทัดที่ 2: `unsafe { value.as_mut_vec().fill(0) };`

เกิดการเขียน 0 ทับลงไปที่ **Heap Block B (ก้อนสำเนา)** เท่านั้น

```text
signal ──► Heap Block A: "SuperSecretPassword123"   ← ไม่ถูกแตะต้องเลย
value  ──► Heap Block B: 0x00 0x00 0x00 ...         ← ถูกล้างข้อมูลเรียบร้อย

```

### บรรทัดที่ 3: `signal.set(String::new());` แล้วจบฟังก์ชัน

* คำสั่ง `signal.set(...)` เข้าไปแทนที่ค่าเดิม ทำให้ `String` ตัวจริง (Block A) ถูก **Drop** และคืนพื้นที่ให้ OS **โดยไม่เคยผ่านการเขียน 0 ทับเลย**
* จากนั้น `value` ก็หลุดสโคปตอนจบฟังก์ชัน ทำให้ Block B (ก้อนที่เพิ่งล้างข้อมูลไป) ถูกคืนพื้นที่ตามลำดับปกติ

> ⚠️ **จุดที่มักเข้าใจสลับกัน:** `set()` ทำงาน **ก่อน** ที่ `value` จะ drop เพราะ value หลุดสโคปตอนปิดฟังก์ชัน แต่ไม่ว่าจะสลับลำดับอย่างไร ผลลัพธ์สุดท้ายก็เหมือนกัน นั่นคือ Block A ถูกปล่อยคืนระบบไปทั้งที่ยังมีรหัสผ่านค้างอยู่เต็ม ๆ

คำถามต่อมาคือ การปล่อยหน่วยความจำคืนไปโดยไม่ล้างข้อมูลแบบนี้ร้ายแรงแค่ไหน คำตอบขึ้นอยู่กับพฤติกรรมของ allocator แต่ละตัว ซึ่งคาดเดาได้ยากกว่าที่คิดครับ

---

## 4. เขียน Test อย่างไรให้จับบั๊กนี้ได้ (โดยไม่ต้องพึ่ง UB)

วิธีแรกที่หลายคนนึกถึงคือแอบเข้าไปอ่านหน่วยความจำที่ pointer เดิมหลังจากเรียก `wipe_field` ว่ายังมีรหัสผ่านเหลืออยู่ไหม แต่นั่นถือเป็น **use-after-free** ซึ่งเป็น Undefined Behavior (UB) ครับ ผลลัพธ์เชื่อถือไม่ได้เลย (อาจได้ค่าเก่า ค่าขยะ หรือแอป crash ไปเลย) และที่สำคัญคือ compiler มีสิทธิ์ออปติไมซ์การอ่านนั้นทิ้งไปได้ด้วย

วิธีที่ถูกต้องและปลอดภัยคือการตรวจสอบ **ownership/pointer identity** แทนการพยายามอ่านค่าหลังถูก free

```rust
#[test]
fn test_wipe_field_clears_wrong_allocation() {
    let secret_str = String::from("SuperSecretPassword123");

    // Pointer ของ heap allocation ที่เก็บความลับตัวจริง
    let original_ptr = secret_str.as_ptr();
    let original_len = secret_str.len();

    let signal = RwSignal::new(secret_str);
    // RwSignal::new ย้าย String เข้าไปเลย pointer จึงต้องเท่าเดิม
    assert_eq!(signal.with_untracked(|value| value.as_ptr()), original_ptr);

    wipe_field(&signal);

    // 1) ค่าที่มองเห็นถูกล้าง
    assert!(signal.get_untracked().is_empty());

    // 2) ก้อนจริงต้องยังอยู่ และถูก zero แล้ว
    //    ถ้าไปล้าง clone ก้อนจริงจะถูก free แบบไม่ zero ดังนั้น assert นี้จับได้
    signal.with_untracked(|value| {
        assert_eq!(
            value.as_ptr(),
            original_ptr,
            "wipe_field zeroed a clone; the real allocation was freed un-zeroed"
        );
        assert!(value.capacity() >= original_len);
        let bytes = unsafe { std::slice::from_raw_parts(original_ptr, original_len) };
        assert!(
            bytes.iter().all(|&byte| byte == 0),
            "secret bytes survived in the original allocation"
        );
    });
}

```

เมื่อนำไปรันกับโค้ดเจ้าปัญหา ผลลัพธ์ก็คือ test แดงทันทีตามคาด

```text
test components::connection_form::tests::test_wipe_field_clears_wrong_allocation ... FAILED

assertion `left == right` failed: wipe_field zeroed a clone; the real allocation was freed un-zeroed
  left: 0x1
 right: 0x600000f80000

```

ลองสังเกตค่าทั้งสองฝั่งให้ดีครับ

* `right = 0x600000f80000` คือ pointer ของ Block A (ก้อนข้อมูลตัวจริง)
* `left = 0x1` คือ pointer ของ `String::new()` ที่อยู่ใน signal ณ ปัจจุบัน เพราะ `Vec` ที่ว่างเปล่าจะไม่มีการทำ allocation จริง มันจึงใช้ **dangling pointer** ที่ aligned กับ `u8` (alignment = 1) แทน

นั่นหมายความว่า signal ไม่ได้ถือ Block A ไว้อีกต่อไป มันถูก drop ทิ้งไปโดยไม่เคยผ่านกระบวนการ zeroize เลยแม้แต่น้อย

ผมตั้งใจปล่อยให้ test นี้แดงทิ้งไว้ก่อน เพราะมันคือหลักฐานยืนยันว่าบั๊กมีอยู่จริงครับ

---

## 5. หลักฐานจาก Allocator จริง: macOS vs WASM

คำว่า "ไม่ถูก zero ก่อน free" ไม่ได้แปลว่าจะ "อ่านรหัสผ่านกลับมาได้เสมอ" เพราะพื้นที่ที่ถูก free ไปแล้วจะตกอยู่ภายใต้การดูแลของ allocator ซึ่งแต่ละตัวก็มีพฤติกรรมไม่เหมือนกัน ผมจึงลองเขียนโปรแกรมทดสอบขึ้นมาสองชุด

* ฝั่ง Native: เขียนด้วย C (ตัดประเด็นเรื่อง Rust ออกไปก่อน) เพื่อจอง block, เขียนรหัสผ่านลงไป, สั่ง free, แล้ว **คัดลอกไบต์บริเวณนั้นลง stack ทันที** (ก่อนที่จะมี allocation ตัวอื่นเข้ามาแทรก)
* ฝั่ง WASM: คอมไพล์ Rust เป็น `wasm32-unknown-unknown` แล้วรันด้วย Node.js เพื่อส่องดู linear memory จริง ๆ (เนื่องจากเป็น target จริงของระบบ frontend)

**ผลลัพธ์ฝั่ง macOS 15.7.7 (Apple M1 รอซื้อ M6)** ออกมาน่าสนใจมาก เพราะพฤติกรรมขึ้นอยู่กับ **ขนาดของ block**

| ขนาด block | ไบต์ที่หลงเหลืออยู่หลัง `free` |
| --- | --- |
| 23 B | 12 ไบต์แรกกลายเป็น metadata + ส่วนที่เหลือกลายเป็น `00` ทั้งหมด |
| 512 B | เกือบทั้งหมดกลายเป็น `00` |
| 1008 B | เกือบทั้งหมดกลายเป็น `00` |
| 1009 B | `"SuperSecretPassword123"` **ยังอยู่ครบถ้วน** |
| 1,048,576 B | ยังอยู่ครบถ้วน |

จุดเปลี่ยนอยู่ที่ **1008 ไบต์** ซึ่งเป็นขอบเขตของ tiny allocator บน macOS block ที่เล็กกว่านั้นถูกเขียน 0 ทับให้โดยอัตโนมัติ (คาดว่าเป็นระบบ security hardening ของ allocator ในเวอร์ชันนี้) แต่ถ้า block ใหญ่กว่านั้น รหัสผ่านจะค้างอยู่ครบถ้วน

> ⚠️ **แต่อย่าเพิ่งวางใจกับพฤติกรรมนี้** เพราะมันไม่ใช่สิ่งการันตีอะไรเลย เป็นเพียง implementation detail ของ OS หรือ allocator ในเวอร์ชันนั้น ๆ หากเปลี่ยนเครื่อง เปลี่ยน OS หรือเปลี่ยนความยาวของรหัสผ่าน พฤติกรรมก็เปลี่ยนได้ทันที และที่สำคัญที่สุด **target จริงของแอปนี้ไม่ใช่ native ครับ**

**ผลลัพธ์ฝั่ง WASM (dlmalloc)** สำหรับก้อนขนาด 23 ไบต์ หลังสั่ง free:

```text
78 03 10 00 78 03 10 00 | 72 65 74 50 61 73 73 77 6f 72 64 31 32 33 00
← free-list prev/next    ← "retPassword123" ยังรอดชีวิตอยู่ครบ 15 ตัว

```

dlmalloc เขียน free-list pointer ทับไป 8 ไบต์แรก (ทำให้คำว่า "SuperSec" หายไป) แต่ **รหัสผ่านส่วนที่เหลือยังอ่านได้จาก linear memory อย่างชัดเจน** นี่คือข้อมูลรั่วไหลจริงในระบบ production เพราะ Leptos CSR ทำงานอยู่บน dlmalloc ตัวนี้นั่นเองครับ

> 💡 **บทเรียนสำคัญ:** OS และ allocator เป็นเพียง "การทำงานแฝง" ไม่ใช่ "ข้อตกลงที่การันตีได้" หน้าที่ในการทำ zeroize จึงต้องเป็นสิ่งที่แอปพลิเคชันต้องรับผิดชอบเองเสมอ

---

## 6. หลุมพรางที่สอง: Compiler อาจแอบลบการทำ Zeroize ของคุณทิ้ง

สมมติว่าเราแก้โค้ดจนล้างข้อมูลได้ถูกก้อน (Block A) แล้ว แต่ยังคงเขียนลูปวนล้างค่าด้วยตัวเองแบบนี้

```rust
for byte in value.as_mut_vec() {
    *byte = 0;
}

```

LLVM จะมองว่า *"โค้ดนี้กำลังเขียน 0 ลงในหน่วยความจำที่กำลังจะถูก free ทิ้งในอีกไม่กี่บรรทัดข้างหน้า แถมไม่มีใครอ่านค่านั้นอีกแล้ว งั้นก็ตัดทิ้งไปเลยดีกว่า"* ซึ่งจะกลายเป็นปรากฏการณ์ **Dead Store Elimination** ในโหมด release build ทันที

นี่คือเหตุผลที่ crate **`zeroize`** ถูกสร้างขึ้นมา โดยมันรับประกันผ่านพฤติกรรมของ `write_volatile` ใน LLVM ว่า

> "Zero out this object from memory using Rust intrinsics which ensure the zeroization operation is not 'optimized away' by the compiler."

ในซอร์สโค้ดของ zeroize เองเคยมีประเด็นถกเถียงเรื่องการใช้ volatile ร่วมกับ non-volatile ตามแนวทาง UCG (Unsafe Code Guidelines) แต่ข้อกังวลนั้นได้รับการตรวจสอบและถอนออกไปแล้ว ถือว่าอิมพลีเมนต์ตามรูปแบบ well-defined พร้อมทั้งมี optimization barrier กำกับไว้อย่างถูกต้อง

---

## 7. แนวทางการแก้ไข

### ทางเลือกที่ 1: ใช้ `update_untracked` + `zeroize`

แทนที่เราจะใช้ `get()` ซึ่งจะได้เป็นก้อน clone ให้เปลี่ยนมาใช้ `update_untracked()` เพื่อยืม **`&mut String` ตัวจริง** ที่อยู่ใน signal ออกมาจัดการ:

```toml
# Cargo.toml
zeroize = "1"

```

```rust
use zeroize::Zeroize;

fn wipe_field(signal: &RwSignal<String>) {
    signal.update_untracked(|value| value.zeroize());
}

```

เหตุผลที่วิธีนี้ถูกต้องสมบูรณ์คือ

* `Zeroize` บน `String` จะเข้าไปจัดการกับ buffer ตัวจริงโดยตรง (ผ่าน `as_mut_vec()`) โดยล้างข้อมูลทั้งในส่วน spare capacity และส่วนที่มีข้อมูลอยู่จริง จากนั้นจึงสั่ง `clear()` ให้ความยาวกลายเป็น 0 โดยที่ **buffer ก้อนเดิมยังคงอยู่ แต่ทุกไบต์ถูกเขียนทับด้วยศูนย์ทั้งหมด**
* ทำให้ไม่มีความจำเป็นต้องสั่ง `signal.set(String::new())` ซ้ำอีก ช่วยลดความสับสนในโค้ดลงได้
* ทดสอบกับ test ในหัวข้อที่ 4 แล้วผ่านครบทั้ง 3 เงื่อนไขคือ ค่ากลายเป็นค่าว่าง / pointer ยังคงเป็นก้อนเดิม / ไบต์ทั้งหมดกลายเป็นศูนย์
* การใช้ `update_untracked` เหมาะมากกับงาน cleanup เพราะไม่จำเป็นต้องส่งสัญญาณแจ้งเตือน subscriber อยู่แล้ว (แต่ถ้าต้องการให้ยังคงความสามารถในการ reactive ตามปกติ ก็เปลี่ยนไปใช้ `update()` แทนได้ ซึ่งให้ผลลัพธ์ด้านความปลอดภัยที่เท่ากัน)

> 💡 **ข้อสังเกตสำหรับ Leptos:** เมธอด `update_untracked` เป็นส่วนหนึ่งของ trait `UpdateUntracked` ซึ่งถูกรวมไว้ใน `leptos::prelude::*` อยู่แล้ว ไม่จำเป็นต้อง import อะไรเพิ่ม

### ทางเลือกที่ 2: ปล่อยให้ Type System ดูแลผ่าน `Zeroizing<String>`

วิธีที่สวยงามและเป็นธรรมชาติที่สุดในสไตล์ Rust (idiomatic Rust) คือการไม่จำเป็นต้องเขียนฟังก์ชัน `wipe_field` ขึ้นมาเลย แต่เปลี่ยนไปใช้ชนิดข้อมูลที่สามารถ "ล้างตัวเองอัตโนมัติเมื่อถูก drop":

```rust
use zeroize::Zeroizing;

let password = RwSignal::new(Zeroizing::new(String::new()));

// การผูกเข้ากับ input element: สามารถ deref ไปเป็น String ได้ตามปกติ
// prop:value=move || password.get().to_string()
// on:input=move |ev| password.set(Zeroizing::new(event_target_value(&ev)))

```

`Zeroizing<Z>` เป็นเพียง wrapper ขนาดเล็ก (`#[repr(transparent)]`) ที่อิมพลีเมนต์ `Deref`/`DerefMut` ครอบไปยังค่าจริง และมี `Drop` ที่จะคอยเรียก `zeroize()` ให้โดยอัตโนมัติ สิ่งที่เราจะได้เพิ่มขึ้นมาทันทีคือ

* ตัดการใช้ `on_cleanup` และ `wipe_field` ทิ้งไปได้เลยทั้งคู่
* ทุกครั้งที่มีการสั่ง `set()` ทับค่าเดิม ค่าเก่าจะถูกสั่ง zeroize ตอนถูก drop โดยอัตโนมัติ
* แม้แต่ตอนที่เรียก `signal.get()` แล้วเกิดการ clone ข้อมูลออกมา ข้อมูลก้อนใหม่ก็ยังคงเป็น `Zeroizing<String>` ซึ่งจะล้างตัวเองเมื่อหลุดสโคปเสมอ ช่วยป้องกันปัญหาข้อมูลรั่วไหลจากตัว clone ได้ด้วย

ข้อควรระวัง: การเชื่อมต่อกับ DOM (`prop:value`) อาจจะทำให้เกิด `String` ชั่วคราวขึ้นที่ฝั่งของ webview ซึ่งอยู่นอกเหนือการควบคุมของ Rust ดังนั้นการทำ zeroize ในฝั่งเราจึงถือว่าเป็นแนวทางป้องกันที่ดีที่สุด (best-effort) เท่าที่จะทำได้ครับ

### ตารางเปรียบเทียบแต่ละวิธี

| แนวทาง | ล้างข้อมูลก้อนจริง | ป้องกัน compiler ลบทิ้ง | ต้องเขียน `unsafe` เอง | ป้องกันการลืมเรียกใช้ |
| --- | --- | --- | --- | --- |
| `fill(0)` หลังสั่ง `get()` | ❌ | ❌ | ✅ | ❌ |
| `update_untracked` + `zeroize` | ✅ | ✅ | ❌ | ❌ |
| `Zeroizing<String>` | ✅ | ✅ | ❌ | ✅ (จัดการผ่าน drop อัตโนมัติ) |

หากเป็นข้อมูลความลับในฝั่ง backend หรือ native เช่น API key หรือรหัสผ่านที่ต้องส่งต่อไปยังระบบอื่น แนะนำให้พิจารณาใช้ **`secrecy::SecretString`** ร่วมด้วยอีกชั้นหนึ่ง ซึ่งถูกออกแบบมาสำหรับจัดการ "ข้อมูลความลับที่ห้ามถูกสั่ง log หรือ print ออกมาโดยไม่ตั้งใจ" โดยเฉพาะครับ

---

## 8. สรุปบทเรียนกันเลยดีกว่า

1. **การ `Clone` โดยไม่ตั้งใจคือต้นตอของปัญหา:** การเรียก `get()` หรือ `get_untracked()` บน `RwSignal<String>` จะได้ข้อมูลสำเนาออกมาเสมอ การตามไปล้างข้อมูลในก้อนสำเนาจึงไม่มีประโยชน์ในเชิงความปลอดภัยเลย
2. **การเขียน `unsafe` จัดการเองเสี่ยงสองเท่า:** มีโอกาสสูงที่จะล้างข้อมูลผิดก้อน (ดังเช่นเคสนี้) และยังมีโอกาสโดน compiler ออปติไมซ์ตัดโค้ดทิ้งไปเฉย ๆ ได้อีกด้วย
3. **อย่าฝากความปลอดภัยไว้กับ Allocator:** จากผลการทดลอง พบว่า macOS จะ zeroize ให้เฉพาะ block ขนาดเล็ก ส่วน WASM (dlmalloc) ยังมีรหัสผ่านค้างอยู่ในหน่วยความจำให้แอบอ่านได้บางส่วน
4. **ใช้ Type System เป็นด่านป้องกัน:** เลือกใช้ `zeroize` สำหรับงานล้างข้อมูลเฉพาะจุด หรือใช้ `Zeroizing`/`secrecy` เพื่อให้ระบบจัดการล้างข้อมูลตาม lifecycle ให้อัตโนมัติ
5. **การเขียน Test ที่ถูกต้องต้องไม่พึ่งพา UB:** ควรใช้วิธีการตรวจสอบ pointer identity และ ownership แทนการพยายามแอบอ่านหน่วยความจำหลังถูกสั่ง free

*หมายเหตุ: โค้ดและผลการทดลองทั้งหมดในบทความนี้ ทดสอบบน macOS 15.7.7 (Apple M1), rustc 1.98, Leptos 0.8.21 และ zeroize 1.9.0*

---

## 9. บทส่งท้าย: สรุปแล้วเราแก้ไขปัญหานี้อย่างไร?

หลังจากเขียนบทความนี้จบ ผมได้กลับไปแก้ไขโค้ดในโปรเจกต์จริงและเปิด PR เป็นที่เรียบร้อยแล้ว ([PR #21](https://github.com/suradet-ps/drugitems/pull/21)) มาดูกันครับว่าสุดท้ายแล้วเราจบปัญหานี้ด้วยวิธีไหน

### ตัดสินใจเลือกทางเลือกที่ 2 เพราะครอบคลุมตลอด Lifecycle

ทางเลือกที่ 1 (`update_untracked` + `zeroize`) แม้จะแก้ปัญหาตรงจุด `on_cleanup` ได้ แต่ก็ยังคงทิ้งช่องโหว่เดิมไว้ นั่นคือทุกครั้งที่ผู้ใช้พิมพ์ข้อความเปลี่ยนแปลง แล้วเกิดการเรียก `password.set(...)` ค่ารหัสผ่านเดิมก่อนหน้านี้จะถูก drop ทิ้งไปโดยไม่เคยผ่านการ zeroize

เราจึงตัดสินใจเลือก **ทางเลือกที่ 2** เพราะการล้างข้อมูลจะเกิดขึ้นโดยอัตโนมัติทุกครั้งที่ค่านั้นถูก drop หรือถูกแทนที่ด้วยค่าใหม่ ที่สำคัญคือเราสามารถลบโค้ดที่ต้องคอยสั่งล้างข้อมูลทิ้งไปได้ทั้งหมดครับ

### โค้ดจริงที่ปรับเปลี่ยน

ตัว signal จะเปลี่ยนมาเก็บ `Zeroizing` โดยตรง ทำให้ไม่จำเป็นต้องมีฟังก์ชัน helper อีกต่อไป

```rust
// connection_form.rs
let password = RwSignal::new(Zeroizing::new(String::new()));

```

และในส่วนของ struct ที่ต้องส่งข้าม IPC ก็ปรับมาใช้ `Zeroizing` ด้วยเช่นกัน เพื่อให้สำเนาของข้อมูลที่ส่งไปยัง backend ถูกสั่งล้างทันทีเมื่อจบการทำงาน

```rust
// api.rs
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    /// Wiped in place whenever this input is dropped or replaced.
    pub password: Zeroizing<String>,
}

```

**สรุปส่วนที่ถูกลบออกจากโค้ดเดิม**

* ลบทั้งฟังก์ชัน `wipe_field` และการเรียกใช้งาน `on_cleanup` ออกทั้งหมด
* ลบคำสั่ง `unsafe { value.as_mut_vec().fill(0) }` ทิ้ง พร้อมทั้งเพิ่ม `#![forbid(unsafe_code)]` ไว้ที่หัวไฟล์ `connection_form.rs` (เลือกใช้ `forbid` แทน `deny` เพื่อป้องกันไม่ให้ใครสามารถเขียน override ทับได้ในอนาคต)
* การ bind ข้อมูลกับ input element ยังจำเป็นต้องเรียก `.to_string()` เพื่อสร้างสำเนาชั่วคราวส่งให้ DOM ซึ่งถือเป็นข้อจำกัดฝั่ง webview ที่เรายังต้องยอมรับอยู่

### ชุด Test ใหม่

unit test ตัวเดิมที่เป็นจุดเริ่มต้นของบทความนี้กลายเป็นของล้าสมัย (obsolete) ไปโดยปริยาย เพราะไม่มีฟังก์ชัน `wipe_field` ให้ทดสอบอีกต่อไป เราจึงแทนที่ด้วย test ใหม่ 3 ตัวเพื่อป้องกันปัญหาในระดับ Type System แทน

1. `password_types_zeroize_on_drop` ตรวจสอบตั้งแต่ตอน compile ว่า `Zeroizing<String>` มีการอิมพลีเมนต์ trait `ZeroizeOnDrop` ไว้จริง
2. `zeroizing_wipes_on_drop` ใช้ spy type ในการพิสูจน์ว่า `Zeroizing` มีการเรียกใช้งาน `zeroize()` ตอนที่ถูก drop จริง ๆ
3. `connection_input_keeps_the_password_zeroizing` ป้องกันไม่ให้มีใครเผลอเปลี่ยน type ของฟิลด์ IPC กลับไปเป็น `String` ธรรมดาในอนาคต

**เกร็ดเล็ก ๆ:** test ตัวเดิมที่เคยใช้วิธีอ่าน raw byte ในหน่วยความจำเพื่อยืนยันการล้างข้อมูล ไม่สามารถนำมารันได้อีกต่อไป เพราะการประกาศ `#![forbid(unsafe_code)]` ห้ามใช้ `unsafe` แม้กระทั่งในบล็อกของ test เราจึงเปลี่ยนมาใช้โครงสร้าง spy ที่เป็น safe code ทั้งหมดแทนครับ

---

**Credit & Reference:**

- [`zeroize` crate](https://docs.rs/zeroize) เอกสารอธิบายการใช้งาน `Zeroize`/`Zeroizing` และรายละเอียดเกี่ยวกับ volatile semantics
- [`secrecy` crate](https://docs.rs/secrecy) เอกสารการใช้งาน `SecretString` สำหรับจัดการข้อมูลความลับ
- [`RwSignal` / `UpdateUntracked` (Leptos 0.8)](https://docs.rs/leptos/latest/leptos/prelude/trait.UpdateUntracked.html) เอกสารอธิบายการใช้งาน trait สำหรับอัปเดตค่าใน Signal
- [`core::ptr::write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html) เอกสารฟังก์ชันมาตรฐานของ Rust
- [Cryptocoding: Clean memory of secret data](https://github.com/veorq/cryptocoding#clean-memory-of-secret-data) แนวทางปฏิบัติด้านความปลอดภัยสำหรับการจัดการหน่วยความจำของข้อมูลความลับ
