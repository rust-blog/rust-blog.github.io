---
title: "CUDA Rust เขียน GPU Kernel ด้วย Rust"
date: "2026-09-22"
description: "NVIDIA เปิดตัว CUDA Rust ให้เขียน GPU kernel ด้วย Rust ได้โดยตรง"
tags: [rust, gpu, cuda]
author: "suradet-ps"
---

เดือนนี้ข่าว Rust เยอะผิดปกติครับ ไล่มาตั้งแต่ Microsoft ประกาศยก Rust เป็น tier-1 language ของตัวเอง, next-generation trait solver ได้เป็นค่าเริ่มต้นบน nightly, mold linker ประกาศ rewrite เป็น Rust ไปจนถึง RustConf 2026 ที่มอนทรีออล

แต่ข่าวที่ผมสะดุดที่สุดคือของ NVIDIA **CUDA Rust** เปิดตัววันแรกของงาน RustConf (8 กันยายน) และวันถัดมา NVIDIA ก็สมัครเป็นสมาชิกระดับ Platinum ของ Rust Foundation

เหตุผลที่ผมสนใจข่าวนี้เป็นพิเศษคือแม้เราจะเขียน Rust ฝั่ง host สวยแค่ไหน สุดท้าย kernel ก็ต้องไปเขียนเป็น C++ อยู่ดี และพอเข้าไปอยู่ใน kernel แล้ว คอมไพเลอร์ก็แทบไม่ช่วยอะไรเราเลย

ต้องบอกก่อนว่าบทความนี้ผมยังไม่ได้ลองรันเอง เพราะการรันต้องการ Linux กับ GPU ของ NVIDIA ซึ่งเครื่องที่ผมใช้เป็น Mac เนื้อหาทั้งหมดมาจากการอ่าน[ประกาศของ NVIDIA](https://developer.nvidia.com/blog/introducing-cuda-rust-two-tracks-for-writing-gpu-kernels/), เอกสารใน repo และบทวิเคราะห์ที่ตามมาอีกที ตรงไหนที่เป็นข้อสังเกตส่วนตัวผมจะเขียนกำกับไว้นะครับ

---

## ทำไม kernel ถึงเป็นชิ้นสุดท้ายที่ยังเป็น C++

NVIDIA ไม่ได้เพิ่งมาสนใจ Rust ครับ ฝั่ง host ของเขาเองมีงานที่เขียนด้วย Rust อยู่แล้ว ทั้งไดรเวอร์ Nova สำหรับ Linux, แกนหลักของ NVIDIA Dynamo และ Rust bindings ของ NVTX แต่ชิ้นที่เป็นข้อยกเว้นมาตลอดคือ kernel ที่รันบน GPU ซึ่งเรามักเขียนเป็น C++ แล้วเรียกผ่าน binding จาก Rust

การเรียกแบบนั้นใช้ได้ แต่ปัญหาคลาสสิกของการเขียน GPU ยังอยู่ครบ ข้อที่เจ็บที่สุดคือ **aliasing** กับ **race condition** เพราะมี thread เป็นพัน ๆ แตะ buffer เดียวกันโดยไม่มีใครรับประกันลำดับการทำงาน ถ้า thread หนึ่งเขียนและอีก thread อ่านตำแหน่งเดียวกัน ผลลัพธ์จะขึ้นกับจังหวะ ซึ่งหมายความว่าโปรแกรมไม่ crash แต่ให้ตัวเลขผิดแบบเงียบ ๆ บางครั้งเจอเฉพาะ launch size บางแบบ ผ่าน test ทั้งชุดแล้วไปพังใน production จนได้

CUDA C++ มี `__restrict__` ให้ใบ้คอมไพเลอร์ว่า pointer ไม่ทับกัน แต่ตอบได้แค่คำถามว่า argument ที่ส่งเข้ามาทับกันหรือเปล่า ไม่ได้ตอบว่า thread ใน block จะ race กันไหม โดยเฉพาะบน shared memory ซึ่งเป็นที่ที่ bug ประเภทนี้ชอบเกิดมากที่สุด

CUDA Rust เข้ามาที่จุดนี้โดยตรง kernel ยังคอมไพล์เป็น PTX เหมือนเดิม แต่บาง bug กลายเป็น error ตอน build

---

## มีสองเส้นทางที่ต้องเลือก

NVIDIA แยก CUDA Rust เป็นสองโมเดลตามสไตล์การเขียนที่ต่างกัน

| เส้นทาง | โมเดล | Crate | Toolchain | สถานะ |
|---|---|---|---|---|
| **cuda-oxide** | SIMT (สั่งเป็นราย thread) | `NVlabs/cuda-oxide` | pinned nightly + LLVM | early alpha |
| **cutile-rs** | Tile (สั่งเป็นราย tile) | `NVlabs/cutile-rs` | stable Rust 1.89+ | ใช้จริงใน Hugging Face Grout และ mistral.rs |

คำแนะนำของ NVIDIA คือเริ่มจาก Tile ก่อน เพราะคอมไพเลอร์เป็นคนตัดสินใจว่า tile จะ map ลง architecture อย่างไร โค้ดเราจึงไม่ผูกกับฮาร์ดแวร์รุ่นใดรุ่นหนึ่ง แล้วค่อยลงมาใช้ SIMT ตอนที่ต้องคุม memory กับ threads เองจริง ๆ

อันนี้ไม่ได้คิดขึ้นใหม่ด้วยครับ NVIDIA ทำแบบเดียวกันกับ Python มาก่อนแล้ว (cuTile Python มากับ CUDA 13.1 เมื่อปลายปีที่แล้ว) คือผลักคนเขียนใหม่ไปที่ Tile แล้วเก็บ SIMT ไว้ให้สายจูน

อีกอย่างที่ควรรู้คือทั้งสองเส้นทางคอมไพล์เป็น PTX เหมือนกัน และ NVIDIA วางแผนทำ interop ระหว่าง CUDA Rust, CUDA C++ และ CUDA Python ไว้ด้วย เลือก frontend แล้วจะไม่ถูกตัดขาดจากกัน

---

## cuda-oxide สาย SIMT ที่ต้องคุม thread เอง

`cuda-oxide` เป็น custom codegen backend ของ rustc ตัวมันดักการคอมไพล์ แล้วส่งฟังก์ชันที่ติด `#[kernel]` ผ่าน Rust MIR → Pliron IR framework → LLVM IR → PTX ส่วนโค้ดที่เหลือปล่อยให้ backend ปกติทำงานต่อ dialect ของ GPU ที่ต่อบน Pliron เป็นของ NVIDIA เอง และทุก transform อยู่ใน Rust จนถึงขั้นตอนสุดท้าย

ฝั่งนี้ต้องการ Linux, GPU ที่ compute capability 8.0 ขึ้นไป, CUDA toolkit 12.x+, clang พร้อม libclang และ nightly toolchain เวอร์ชัน pin ไว้ ตอนนี้คือ `nightly-2026-04-03`

```bash
cargo +nightly-2026-04-03 install --git https://github.com/NVlabs/cuda-oxide.git cargo-oxide

cargo oxide new vecadd_demo
cd vecadd_demo
cargo oxide doctor
cargo oxide run
```

`cargo oxide new` สร้างโปรแกรม vector addition ที่สมบูรณ์มาให้เลย host กับ device อยู่ในไฟล์เดียว ฝั่ง device หน้าตาแบบนี้

```rust
#[cuda_module]
mod kernels {
    use super::*;

    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub fn vecadd(a: &[f32], b: &[f32], mut c: DisjointSlice<f32>) {
        let idx = thread::index_1d();
        let idx_raw = idx.get(); // usize เปล่า ๆ สำหรับอ่าน input
        if let Some(c_elem) = c.get_mut(idx) {
            *c_elem = a[idx_raw] + b[idx_raw];
        }
    }
}
```

signature ของ kernel ตัวนี้น่าสนใจครับ `a` กับ `b` เป็น slice ธรรมดาที่ทุก thread อ่านได้ ส่วน `c` เป็น `DisjointSlice<f32>` ซึ่งแจกสิทธิ์เขียนให้แต่ละ thread เฉพาะ element ของตัวเอง จำเป็นต้องมี type นี้เพราะ `&mut [f32]` ใช้ไม่ได้ ทุก thread อยากได้ `&mut` ก้อนเดียวกัน ซึ่ง Rust ปฏิเสธถูกแล้ว `DisjointSlice` จึงแบ่ง mutable borrow ก้อนนั้นออกเป็นชิ้นต่อ thread

อีกจุดคือ `thread::index_1d()` คืน index type เฉพาะ ไม่ใช่ `usize` เปล่า ๆ และ `c.get_mut(idx)` รับแค่ type นั้น พร้อมคืน `Option` ให้เรา handle out-of-bounds เป็น branch ปกติ ไม่ใช่ memory error ที่ไปเจอเอาทีหลัง

การ launch ก็ตรวจก่อนเชื่อ `#[launch_contract]` ประกาศว่า kernel นี้ index 1 มิติ บล็อกละ 256 threads แล้ว `prepare_vecadd` จะตรวจ `LaunchConfig1D` ของเรากับ contract และข้อจำกัดจริงของ device ก่อนคืน proof ที่ safe launch method บังคับต้องใช้

```rust
let module = unsafe { kernels::load(&ctx)? };
let prepared = module.prepare_vecadd(LaunchConfig1D::new((N as u32).div_ceil(256), 256, 0))?;
module.vecadd(&stream, &prepared, &a_dev, &b_dev, &mut c_dev)?;
```

จะเห็นว่ามี `unsafe` จุดเดียวคือตอน `load` module ที่เราคุมเอง ส่วน kernel ที่ไม่มี contract จะเปิดเฉพาะ unsafe launch method เพราะ `LaunchConfig` เปล่า ๆ ไม่ได้บอกอะไรเกี่ยวกับ kernel ที่กำลัง launch

เห็น nightly ที่ pin ไว้แล้วผมใจแป้วนิดหน่อยครับ หมายความว่าทุกการเปลี่ยนของ rustc upstream มีโอกาสทำให้ backend พัง และ NVIDIA ต้องคอยวิ่งตาม compiler internals ไปเรื่อย ๆ กว่าจะกล้าใส่ใน pipeline จริงคงต้องรอให้ build บน stable ได้ก่อน

---

## cutile-rs สาย Tile ที่คอมไพเลอร์คุมให้

`cutile-rs` ทำงานสูงขึ้นมาหนึ่งระดับ เราคิดเป็น tile ไม่ใช่ scalar แต่ละ tile block รัน body ของ kernel ครั้งเดียวเป็น logical thread ตัวเดียวบน sub-tensor หนึ่งชิ้น คอมไพเลอร์เป็นคนตัดสินว่าจะใช้ GPU thread จริงกี่ตัวมารองรับ

macro `#[cutile::module]` ฝัง AST ของ kernel ไว้ใน host binary แล้ว JIT ผ่าน CUDA Tile IR ครั้งแรกที่ kernel ถูกเรียกใช้จริง ข้อกำหนดเบากว่าฝั่งแรกเยอะ Linux, GPU compute capability 8.0+, CUDA 13.3 และ stable Rust 1.89+ ไม่ต้อง nightly ไม่ต้อง build LLVM เอง

```bash
cargo new vecadd_demo
cd vecadd_demo
cargo add cutile
```

โค้ดฝั่ง device ของงานเดียวกันเขียนแบบนี้

```rust
#[cutile::module]
mod kernel {
    use cutile::core::*;

    #[cutile::entry()]
    fn add<const B: i32>(
        z: &mut Tensor<f32, { [B] }>, // output แบบ exclusive หนึ่ง sub-tensor ขนาด B
        x: &Tensor<f32, { [-1] }>,    // input ที่แชร์กัน โดย -1 คือมิติ dynamic
        y: &Tensor<f32, { [-1] }>,
    ) {
        let tx = load_tile_like(x, z); // โหลด tile ของ x ที่ตรงกับ sub-tensor นี้
        let ty = load_tile_like(y, z);
        z.store(tx + ty);              // บวกกันทั้ง tile
    }
}
```

ฝั่ง host

```rust
let x = api::ones::<f32>(&[1024]);
let y = api::ones::<f32>(&[1024]);

let z = api::zeros::<f32>(&[1024]).partition([128]);

let c: Vec<f32> = kernel::add(z, x, y) // รับ ownership ของทั้งสาม tensor ไป
    .first()                           // แล้วคืนกลับมาเป็น tuple เลือก output ออกมา
    .unpartition()                     // ถอด wrapper ฝั่ง host ไม่มีการย้ายข้อมูล
    .to_host_vec()
    .sync_on(&stream)?;                // จุดนี้เท่านั้นที่ทุกอย่างรันจริง
```

บรรทัดที่ทำงานหนักที่สุดคือ `.partition([128])` มันทำสามอย่างพร้อมกัน

1. ทำให้ exclusivity เป็นจริง แต่ละ tile เป็นเจ้าของ chunk 128 element ของตัวเอง และ tile อื่นแตะไม่ได้
2. กำหนด geometry ของการ launch 1,024 หาร 128 ได้ grid 8 tiles โดยไม่ต้องคำนวณแยกแล้วเอาไปตรวจกับ kernel
3. ส่งค่า `B` ให้ kernel โดยเราไม่ต้องเขียนที่ call site เพราะ launcher อ่านความกว้าง tile จาก partition เอง

ทั้งหมดก่อน `.sync_on()` เป็นแค่คำอธิบายแบบ lazy ที่ยังไม่แตะ GPU เลย แม้แต่การ copy กลับ host ก็ยังไม่เกิด

ถ้าให้เลือกตัวเดียวที่จะลองตอนนี้ ผมเลือก cutile-rs เพราะอยู่บน stable Rust, ไม่ต้อง build LLVM เอง และมีคนใช้จริงนอก NVIDIA แล้วทั้งใน Grout ของ Hugging Face กับ mistral.rs

---

## aliasing ที่กลายเป็น error ตอน build

ส่วนที่ผมชอบที่สุดไม่ใช่ตัวภาษา แต่เป็น error ที่มันจับได้ ถ้าเราส่ง output buffer ของ kernel สาย SIMT กลับเข้าไปเป็น input ของตัวเอง

```rust
module.vecadd(&stream, &prepared, &c_dev, &b_dev, &mut c_dev)?;
```

```
error[E0502]: cannot borrow `c_dev` as mutable because it is also borrowed as immutable
```

ฝั่ง Tile ก็เหมือนกัน

```rust
let z = api::zeros::<f32>(&[1024]);
kernel::add(z.partition([128]), z, y)
```

```
error[E0382]: use of moved value: `z`
```

error ทั้งคู่เป็น error ธรรมดาที่เราเจอบ่อยในโค้ด Rust ฝั่ง host มันเลยดูไม่น่าตื่นเต้น แต่สำหรับคนที่เคยนั่งไล่ bug บน GPU มาทั้งคืน การที่ aliasing กลายเป็น build error คือสิ่งที่อยากได้มานาน

ต่างกันนิดหน่อยตรงที่ cuda-oxide ตรวจตอนแต่ละ launch call ส่วน cutile-rs ให้ ownership ตาม tensor ข้ามขอบเขต launch ไปเลย ซึ่งเป็นสัญญาที่แรงกว่า และเป็นเหตุผลว่าทำไมฝั่ง Tile ต้อง partition output ก่อนส่งเข้า kernel เสมอ

---

## ตัวเลขประสิทธิภาพที่ควรอ่านให้ครบ

ตามที่ NVIDIA รายงานบน B200

| งาน | ผลที่ได้ | เทียบ baseline |
|---|---|---|
| Persistent f16 GEMM (8192) | 2.07 PFlop/s | 96.4% ของ cuBLAS |
| Element-wise bandwidth | 7 TB/s | ~91% ของ 8 TB/s HBM3e peak |

ฝั่งความปลอดภัยแทบไม่แลกประสิทธิภาพเลยครับ เอกสารของ cuda-oxide รายงานว่า safe view kernel ทำได้ 7,159 GFLOPS เทียบกับ 7,161 GFLOPS ของเวอร์ชัน raw pointer ที่เขียน `unsafe` เอง คิดเป็น overhead ราว 0.1%

แต่ต้องอ่านให้ครบด้วยว่า 96.4% มาจาก persistent GEMM ที่ NVIDIA จูนเอง ซึ่งไม่เหมือน tile code ทั่วไป มีผลประเมินอิสระบน RTX PRO 6000 Blackwell ที่ได้ 52-79% ของ cuBLAS สำหรับ GEMM และประมาณ 53% ของ FlashAttention-2 โดยผู้ประเมินระบุว่าประสิทธิภาพขึ้นกับ workload และ architecture มาก ตัวเลขแนวนี้ถ้าไม่ดูว่าวัดจากเคสไหนก็หลงได้ง่าย

---

## สิ่งที่ type system ยังจับไม่ได้

ข่าวนี้ดี แต่ไม่ควรเข้าใจว่าปลอดภัยครบทุกมุมแล้ว

- **Shared memory ในสาย SIMT ยังต้อง `unsafe`** และนี่คือหัวใจของ kernel ที่เร็ว ๆ แปลว่าเส้นทางปลอดภัยยังครอบไม่ถึงงานที่ต้องจูนหนัก
- **Thread-divergent control flow รอบ barrier** ยังไม่ถูกพิสูจน์ ทีมงานแก้ด้วยการปิด optimization pass ของ LLVM ไม่ใช่ด้วย type system
- **Warp convergence ของ `shfl_sync` กับ `ballot_sync`** พลาดแล้วได้ silent hang ไม่ใช่ error
- **TMA, tensor cores และ cluster-level communication** ยัง manual ทั้งหมด
- **ทั้งสองเส้นทางต้อง Linux + GPU compute capability 8.0 ขึ้นไป** เครื่อง Mac อย่างที่ผมใช้อยู่นี้รันไม่ได้เลย
- **ยังไม่ production-ready** cuda-oxide ยัง early alpha และต้อง nightly ที่ pin ไว้ ส่วน cutile-rs ไกลกว่าแล้ว แต่ API ยังขยับ

---

## ตอนนี้ควรทำอะไร

โปรเจกต์ Rust on GPU ไม่ใช่เรื่องใหม่ และ NVIDIA เองก็ให้เครดิตคนที่มาก่อน ทั้ง rust-cuda ที่เริ่มจาก v0.3 เมื่อกุมภาพันธ์ 2022 แล้วเงียบไปสามปี ก่อนกลับมาเริ่มใหม่เมื่อมกราคม 2025 โดย Christian Legnitto กับ Jorge Ortega และ merge PR จาก contributor ใหม่ได้กว่า 20 รายภายในไม่กี่เดือน

ส่วน CubeCL เป็นอีกทางที่เขียน kernel เดียวแล้วคอมไพล์ได้ทั้ง CUDA, ROCm และ WGPU ได้ portability แต่แลกมาด้วยข้อจำกัดของภาษาที่ใช้เขียน จะได้ safety บนฮาร์ดแวร์ NVIDIA หรือได้ portability ด้วยภาษาที่เล็กกว่า ยังไม่มีใครขายทั้งสองอย่างพร้อมประสิทธิภาพเต็ม

ถ้าให้สรุปเป็นแนวปฏิบัติตอนนี้

- โปรเจกต์ inference ที่มี kernel อยู่แล้ว เก็บไว้ตรงนั้นแล้วเรียกผ่าน **cudarc** ต่อไป ยังไม่ต้อง rewrite เพราะ cudarc เป็น binding ฝั่ง host ที่ stable และเข้ากันได้กับ PTX ที่ cuda-oxide ผลิต
- โปรเจกต์ใหม่ที่คุมฮาร์ดแวร์เอง ลอง **cutile-rs** ได้เลย
- **เลี่ยง cuda-oxide ใน production** จนกว่าจะ build บน stable toolchain ได้

---

## ทิ้งท้าย

หลังอ่านจบผมมองว่านี่เป็นก้าวที่หลายคนรออยู่ ฝั่ง host ของระบบ AI ย้ายมา Rust กันเยอะมากในช่วงสองปีที่ผ่านมา kernel เหลือเป็นชิ้นสุดท้ายที่ยังต้องพึ่งภาษาอื่น ข่าวนี้ปิดช่องนั้นได้ในทางเทคนิคแล้ว เหลือแค่รอให้ toolchain นิ่งพอจะเอาไปใช้จริง

ใครมีเครื่อง Linux กับ GPU NVIDIA ลองเริ่มจาก `cargo add cutile` แล้วรัน hello world ดูครับ ส่วนผมคงได้แต่อ่าน [cuda-oxide book](https://nvlabs.github.io/cuda-oxide/) กับ[เอกสาร cutile-rs](https://nvlabs.github.io/cutile-rs/main/) ไปก่อน อย่างน้อยเวลามีคนถามว่า Rust เขียน GPU ได้จริงไหม จะได้ตอบได้ว่าตอนนี้ได้แล้ว พร้อมบอกต่อได้ด้วยว่าตรงไหนที่ยังต้องระวัง
