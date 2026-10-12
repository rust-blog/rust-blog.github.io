---
title: "เมื่อ build script ทำงานด้วยสิทธิ์ของคุณ"
date: "2026-09-30"
description: "ต่อยอดจากบท Development environment ของ ANSSI rust-guide ว่าด้วย rustup, cargo, rustfmt และ clippy"
tags: [rust, security]
author: "suradet-ps"
---

บทความก่อนหน้านี้ผมแนะนำ [rust-guide](/post/anssi-rust-guide) หรือ *Secure Rust Guidelines* ของ ANSSI ไปแล้วทั้งเล่ม ใครที่ยังไม่ได้อ่าน สามารถกลับไปอ่านก่อนได้ครับ คราวนี้จะซูมเฉพาะบท **Development environment** ซึ่งมีคำแนะนำทั้งหมด 8 ข้อ (Rule 7 + Recommendation 1) ครอบคลุมเครื่องมือสี่ตัวที่เราติดตั้งลงเครื่องและใช้ทุกวันโดยไม่เคยคิดถึงมัน ได้แก่ **rustup**, **cargo**, **rustfmt** และ **clippy**

เป้าหมายของบทความนี้มีอย่างเดียวคือความปลอดภัย และเพราะเรื่องนี้ขยับเร็วมาก ข้อมูลเพิ่มเติมถึงกันยายน 2026 ปรากฏว่าคำเตือนในหนังสือเล่มนี้หลายข้อเลิกเป็นทฤษฎีไปแล้ว หลังเหตุการณ์ [arrayref เมื่อเดือนสิงหาคม 2026](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/) ที่ build script ของ dependency แอบรันมัลแวร์บนเครื่องนักพัฒนาระหว่าง `cargo build` ธรรมดา ๆ

## ทำไมบทนี้ถึงสำคัญกว่าเดิม

เครื่องมือใน toolchain ไม่ใช่เรื่องแยกจากความปลอดภัยของโปรแกรม แต่เป็นส่วนหนึ่งของ trusted computing base

ลองคิดตามจริง ๆ ดูนะครับ ก่อนที่โค้ดของเราจะได้ทำงาน มันต้องผ่าน rustup ที่ดาวน์โหลด toolchain มาให้ ผ่าน cargo ที่ดึงโค้ดคนอื่นมาร้อยกว่า crate ผ่าน build script ของ dependency ที่รันโค้ดบนเครื่องเราก่อนโค้ดของเราเสียอีก ผ่าน rustfmt/clippy ที่บางครั้งก็แก้โค้ดให้อัตโนมัติ ทั้งหมดนี้คือพื้นที่ที่หนังสือบท Development environment ดูแลครับ

และในปี 2026 มันจับต้องได้จริง เพราะ `cargo` เองยังมี CVE ของตัวเอง คือ [CVE-2026-33055 และ CVE-2026-33056](https://blog.rust-lang.org/2026/03/21/cve-2026-33056/) ใน crate `tar` ที่ cargo ใช้ตอนแตกไฟล์ `.crate` ตอน build โดยตัวที่รุนแรงคือการ `chmod` โฟลเดอร์นอก extraction root ผ่าน symlink (แก้ใน Rust 1.94.1 วันที่ 26 มีนาคม 2026) ส่วนฝั่ง crates.io เองก็แจ้งว่าได้ audit crate ทั้งหมดที่เคย publish และบล็อกการอัปโหลดที่ลองใช้ช่องนี้แล้ว

## Rustup

`rustup` ดาวน์โหลดทุกอย่างผ่าน HTTPS แต่**ยังไม่ตรวจลายเซ็นของไฟล์ที่ดาวน์โหลด** การป้องกัน downgrade attack, certificate pinning และการตรวจลายเซ็นยังอยู่ระหว่างดำเนินการ ทำให้ในบางกรณีเราอาจเลือกวิธีติดตั้งแบบอื่นจากหน้า *Install* ของเว็บไซต์ Rust อย่างเป็นทางการแทน

หนังสือยังมี Rule สองข้อที่เกี่ยวข้องตรง ๆ คือ

- **DENV-STABLE** การพัฒนาแอปที่ปลอดภัยต้องใช้ toolchain ที่ stable เต็มรูปแบบ เพื่อจำกัดบั๊กที่อาจเกิดจากตัวคอมไพเลอร์ รันไทม์ หรือเครื่องมือ
- **DENV-TIERS** งานที่ต้องการความมั่นคงสูงต้องใช้ target ระดับ tier 1 และ toolchain ที่ผ่านการ certify แล้วเท่านั้น

พร้อมกับข้อควรระวังที่คนมักมองข้าม คือการมี toolchain หลายชุดในเครื่อง เพราะต่อให้ default เป็น stable แต่ในโฟลเดอร์หนึ่งอาจมี override เป็น nightly ค้างอยู่ก็ได้ (`rustup override list` ดูได้)

### สถานะจริงในปี 2026

ณ กันยายน 2026 [หน้า Security ของ rustup](https://rust-lang.github.io/rustup/security.html) ยังคงระบุตรง ๆ ว่า rustup ทำทุกอย่างผ่าน HTTPS แต่ยังไม่ตรวจลายเซ็นการดาวน์โหลด สิ่งที่ขยับไปแล้วคือ

- **default download backend** เปลี่ยนเป็น `reqwest` + `rustls` ตั้งแต่รุ่น 1.28 โดยใช้ `rustls-platform-verifier` ตรวจ certificate จาก store ของแพลตฟอร์ม ส่วน backend เก่าอย่าง curl ถูก deprecate
- [แผนรุ่น 1.30](https://blog.rust-lang.org/inside-rust/2026/07/03/rustup-update-1.30/) (ประกาศกลางปี 2026) จะ**ลบ backend `curl` ทิ้ง** และมีการพูดถึงงาน **signing & mirroring support** อย่างจริงจัง
- Rust Project Goals 2026 มีเป้าหมาย [Implement Verifiable Mirroring Prototype](https://goals.rust-lang.org/2026/mirroring.html) ที่จะทำ mirror ของ rustup และ cargo ให้ตรวจสอบได้ด้วย crypto ผ่าน **The Update Framework (TUF)** โดยมี key signing ceremony, การออกแบบหลายกุญแจ และเริ่มทดลองกับ GitHub Actions runners บน Azure ก่อน เป้าหมายนี้ระบุตรง ๆ ว่าทุกวันนี้ผู้ใช้ต้องเชื่อ HTTPS endpoint ของ infra provider แบบไว้ใจล้วน ๆ
- ทางออกระยะสั้นที่หนังสือแนะคือ installer แบบ **standalone** ที่ยัง[เซ็นด้วย GPG key ของทีม Rust](https://forge.rust-lang.org/infra/other-installation-methods.html) และตรวจเองได้ รวมถึงการติดตั้งผ่าน package manager ของ OS

### จัดการ toolchain ให้ตรวจสอบได้

- **pin toolchain ไว้ใน repo** ด้วย `rust-toolchain.toml` ทั้ง channel, targets และ components แบบที่โปรเจกต์ rust-blog นี้ทำ CI จึงใช้ toolchain ตัวเดียวกันกับเครื่อง dev เป๊ะ แทนที่จะพึ่งค่าของแต่ละเครื่อง
- เช็คก่อน build ทุกครั้งว่าไม่มี override แปลกปลอมค้างอยู่

```shell
$ rustup show
$ rustup override list
```

- **อัปเดต stable สม่ำเสมอ** เพราะจุดปล่อยรุ่นย่อยในปี 2026 ก็มีทั้งเรื่อง miscompilation ของตัวคอมไพเลอร์เอง (1.97.1, 1.98.1) และ CVE ใน dependency ของ cargo เอง (1.94.1) การค้างเวอร์ชันเก่านาน ๆ จึงไม่ใช่ความปลอดภัย
- ใช้ `rustup component add clippy rustfmt` สำหรับ component อย่างที่หนังสือแนะนำ อย่าติดตั้ง lint/format tool เป็น dependency ในโปรเจกต์
- ถ้าจำเป็นต้องใช้ nightly จริง ๆ ให้สลับเฉพาะคำสั่ง (`cargo +nightly fmt`) ไม่สลับทั้งเครื่อง ตามที่หนังสือยกตัวอย่างไว้
- งาน safety-critical ระดับที่ต้องมีใบรับรองสามารถใช้ **Ferrocene** ซึ่งเป็น downstream distribution ที่เปิดซอร์สและสอบผ่าน TÜV SÜD ตาม ISO 26262 (ASIL D), IEC 61508 (SIL 3) และ IEC 62304 (Class C) โดยรุ่น [26.05.0 (กรกฎาคม 2026)](https://ferrous-systems.com/blog/ferrocene-26-05-0/) เพิ่งได้ `core::fmt` เข้าใบรับรอง และมี certified subset ของ `core` ระดับ ASIL B / SIL 2 ด้วย นี่คือตัวอย่างจริงของคำว่า certified toolchain ใน Rule DENV-TIERS

> ข้อสังเกตของผม เป้า TUF ของทีม Rust เป็นงานที่ถูกต้องและจำเป็น แต่กว่าจะถึงวันนั้นอีกพักใหญ่ คำตอบที่ใช้ได้วันนี้คือ pin ทุกอย่างที่ pin ได้ (toolchain, lockfile, CI action) แล้วเชื่อน้อยที่สุดเท่าที่จำเป็น

## Cargo

Cargo คือหัวใจของทุกอย่างตามที่หนังสือเขียนไว้ ทั้งสร้างโปรเจกต์ คอมไพล์ รัน test ดาวน์โหลด dependency และ publish ประเด็นความปลอดภัยที่ยกมาคือ

1. **checksum ของ dependency** ถูกเก็บใน `Cargo.lock` และถูกเทียบกับของที่ดาวน์โหลดจริงทุกครั้ง ถ้าไม่ตรงจะ fail ทันที แบบนี้

```text
error: checksum for `sha256 v1.6.0` changed between lock files

this could be indicative of a few possible errors:

    * the lock file is corrupt
    * a replacement source in use (e.g., a mirror) returned a different checksum
    * the source itself may be corrupt in one way or another

unable to verify that `sha256 v1.6.0` is the same as when the lockfile was generated
```

2. **`Cargo.lock` ต้องถูก track ใน version control** (Rule DENV-CARGO-LOCK) ส่วนครั้งแรกที่สร้างไฟล์ยังใช้โมเดล TOFU (*Trust On First Use*) อยู่นั่นเอง
3. **อย่า override `debug-assertions` และ `overflow-checks`** ใน `[profile.dev]` / `[profile.test]` (Rule DENV-CARGO-OPTS) เพราะจะทำให้บั๊กที่ปกติถูกจับได้หลุดรอดออกไป
4. **อย่า override `RUSTC`, `RUSTC_WRAPPER` และ `RUSTFLAGS`** ตอน build (Rule DENV-CARGO-ENVVARS) เพราะจะทำให้เราไม่รู้แน่ชัดว่า compiler ตัวจริงที่ใช้คืออะไร และ options อะไรถูกส่งไป

### สิ่งที่หนังสือไม่ได้พูดถึง (เพราะยังไม่เกิด)

หนังสือเตือนเรื่อง TOFU ไว้ และปี 2026 ก็เป็นปีที่คำเตือนนั้นถูกพิสูจน์อยู่ แต่ฝั่ง crates.io ก็ขยับเหมือนกัน มาดูกันว่าเปลี่ยนอะไรไปบ้าง

- **Trusted Publishing** ใช้ OIDC จาก CI แลก token อายุ 30 นาทีสำหรับ publish แทน API token อายุยาว เริ่มจาก GitHub Actions (2025) แล้วเพิ่ม GitLab CI/CD ต้นปี 2026 พร้อมโหมด **Trusted Publishing Only** ที่เจ้าของ crate เปิดบังคับได้ว่าห้าม publish ด้วย token ธรรมดา
- **Security tab** บนหน้า crate แสดง advisory จาก RustSec ตรง ๆ และในอัปเดต[กลางปี 2026](https://blog.rust-lang.org/2026/07/13/crates-io-development-update/) ก็เพิ่มแบนเนอร์เตือน crate ที่ถูก flag ว่า unmaintained รวมถึงป้ายชวนคิดว่าอาจไม่ต้องใช้ dependency นั้นแล้วสำหรับ crate ที่มีของใน standard library แทนได้
- **Code tab** เปิดให้เราดูไฟล์จริงที่ cargo ดาวน์โหลดได้บนหน้าเว็บ crates.io ต่างจาก GitHub repo ที่อาจไม่ตรงกัน และกำลังจะมี diff viewer ระหว่างสองเวอร์ชัน ซึ่งช่วยเรื่องการ review ก่อนอัปเกรดมาก
- [นโยบายแจ้งเตือนมัลแวร์](https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update/) เปลี่ยนจาก blog post ทุกครั้ง เป็นออก **RustSec advisory เสมอ** และ blog เฉพาะเคสที่มีการใช้งานจริงหรือถูกโจมตีจริง ทำให้ RSS ของ RustSec กลายเป็นช่องทางติดตามที่ขาดไม่ได้

แต่อย่าลืมว่า trusted publishing ไม่ได้แก้ปัญหา**การรันโค้ด** build script ยังคงรันด้วยสิทธิ์เต็มของผู้ใช้เสมอ และนั่นคือช่องที่ arrayref ใช้

### ชุดตรวจ dependency ที่ควรมี

- [`cargo-audit`](https://rustsec.org/) เทียบ `Cargo.lock` กับ RustSec advisory DB ใช้ `cargo audit --deny warnings` ใน CI และถ้าติดตั้ง [`cargo-auditable`](https://github.com/rustsec/rustsec) ไว้ก็สั่ง `cargo audit bin` สแกน dependency tree ที่ฝังใน binary จริงได้
- `cargo deny` ตรวจ advisories, licenses, bans และ sources ครบกว่า - [บทความ cargo-deny ในบล็อกนี้](/post/cargo-deny)เขียนละเอียดไว้แล้ว
- [`cargo-vet`](https://mozilla.github.io/cargo-vet/) ของ Mozilla บันทึกผล audit ของ dependency แบบมี criteria อย่าง `safe-to-run` / `safe-to-deploy` รองรับ diff audit (รีวิวเฉพาะส่วนที่ต่างจากเวอร์ชันเดิม ทำให้ audit ถูกลงมาก) และ import audit จากองค์กรที่เราเชื่อถือได้
- [`cargo-crev`](https://github.com/crev-dev/cargo-crev) ใช้ web of trust ของนักรีวิว สั่ง `cargo crev crate verify` ดูสถานะความน่าเชื่อถือของทุก crate ได้
- ฝึกวินัย lockfile: ใช้ `cargo build --locked` หรือ `--frozen` ใน CI เสมอ ถ้าต้องอัปเดตให้ใช้ `cargo update -p <crate> --precise <version>` แล้ว **อ่าน diff ของ `Cargo.lock`** ทุกครั้ง
- `cargo install` ก็รัน build script เหมือนกัน จึงควรใช้ `cargo install --locked` และระบุเวอร์ชันตายตัว
- **ตรวจ build script เป็นพิเศษ** dependency ที่เพิ่ม `build-dependencies` ด้าน network อย่าง `ureq`/`reqwest`/`rustls` ทั้งที่ไม่มีเหตุต้องใช้ ถือเป็นสัญญาณอันตรายทันที ตรวจได้จาก Code tab ของ crates.io หรือ `cargo vendor` แล้ว grep
- ถ้าอยากรัดกุมระดับ ACL มี [`cackle`](https://github.com/cackle-rs/cackle) ที่กำหนดได้ว่า crate ไหนเรียก API อะไรได้บ้าง และรัน build script ใน bubblewrap sandbox ได้ ส่วน cargo เองกำลังทดลอง [`[host] runner`](https://github.com/rust-lang/cargo/issues/16591) (unstable, ต้นปี 2026) ให้ห่อการรัน build script ด้วย sandbox ภายนอกได้ และมี[ข้อเสนอ explicit opt-in](https://github.com/rust-lang/cargo/issues/17408) ให้ปฏิเสธ build script ของ dependency ทั้งหมดยกเว้นที่อนุญาต (สิงหาคม 2026) ทั้งคู่ยังไม่ stable แต่ทิศทางชัดว่าเรื่องนี้จะกลายเป็น feature ของ cargo ในอนาคต

## กรณีศึกษา: arrayref กับคืนวันที่ 20 สิงหาคม 2026

ถ้าจะมีเหตุการณ์ที่ยืนยันทุกอย่างที่หนังสือเตือนเรื่อง build script นี่คือเหตุการณ์นั้นครับ

**ไทม์ไลน์ที่เกิดขึ้นจริง** (เวลาสากล)

- 01:17 UTC แฮกเกอร์สร้างบัญชี GitHub ปลอมเป็น David Tolnay นักพัฒนาเจ้าของ `proc-macro2`
- 01:55 เผยแพร่ `proc-macro1@1.0.106` ซึ่งเป็นสำเนาของ `proc-macro2` ไว้สร้างความน่าเชื่อถือ
- 07:11 อัปเดตเป็น `proc-macro1@1.0.107` ที่มีโค้ดอันตรายใน `build.rs`
- 07:15 บัญชีของ David Roundy ถูกใช้ publish `arrayref@0.3.10` ที่เพิ่ม dependency `proc-macro1` เข้ามา **ทั้งที่ `arrayref` ไม่เคยมี dependency เพิ่มมา 10 ปี** พร้อมกับ yank เวอร์ชัน 0.3.5–0.3.9 ทิ้ง เพื่อบังคับให้ `cargo update` เลือกเวอร์ชันอันตราย (เทคนิคนี้ทำให้ warning เรื่อง yank กลายเป็นอาวุธ ซึ่งน่ารู้มาก)
- 07:34 และ 07:37 เผยแพร่ `internment@0.8.7` กับ `append-only-vec@0.1.9` ด้วยวิธีเดียวกันครบสามตัวใน 23 นาที
- 07:54 มีรายงานถึงทีม Rust ([advisory-db#3161](https://github.com/rustsec/advisory-db/issues/3161)) และ 08:03 ลบ `proc-macro1`
- 08:41 ลบ `arrayref@0.3.10` (ออนไลน์ 86 นาที), 09:04 ลบ `internment@0.8.7` (90 นาที) และ 09:25 ลบ `append-only-vec@0.1.9` (107 นาที)

**ทำไมมันถึงอันตรายมาก**

`arrayref` มียอดดาวน์โหลดสะสมราว 245 ล้านครั้ง อยู่ใน dependency graph ของ `egui`, `iced`, `blake3` และ tooling ฝั่ง Solana/Ethereum โดยที่หลายคนไม่เคยอ้างชื่อมันตรง ๆ ด้วยซ้ำ (มันเข้ามาทาง `winit` → `tiny-skia` เป็นต้น) ส่วนตัว `build.rs` ของ `proc-macro1` จะดาวน์โหลด payload เฉพาะ OS แล้วรันแบบ detached พร้อมปิด TLS verification, บน Windows ใช้ `wscript` + PowerShell แบบ hidden, จากนั้น beacon ไปที่ C2 และล้วง credential จาก Chrome/Brave/Edge พร้อมติด persistence ในเครื่อง (Registry Run, systemd user service, LaunchAgent) นี่ไม่ใช่แค่ malware ทดลอง แต่เป็น infostealer ที่มี DGA fallback และมีการวิเคราะห์ว่ามี infrastructure overlap กับแคมเปญของเกาหลีเหนือ ([Wiz](https://www.wiz.io/blog/rust-supply-chain-attack-on-arrayref-significant-overlap-with-dprk-campaigns), [StepSecurity](https://www.stepsecurity.io/blog/arrayref-rust-crate-supply-chain-attack), [JFrog](https://research.jfrog.com/post/arrayref-proc-macro1-crates-io/))

**บทเรียนที่เอาไปใช้ได้ทันที**

1. **การติดตั้ง dependency กับการรันโค้ดของคนอื่นเป็นเรื่องเดียวกัน** เพราะ build script รันตอน compile ด้วยสิทธิ์เดียวกับเรา ถ้า build ในเครื่องที่มี SSH key, cloud credential หรือ token ของ CI นั่นคือสิ่งที่มัลแวร์มองเห็น
2. **yank ไม่ใช่สัญญาณให้อัปเกรดแบบไม่ดู** เหตุการณ์นี้ใช้ yank เป็นกลไกบังคับอัปเกรดโดยตรง ถ้าเห็น crate อายุสิบปีถูก yank หลายเวอร์ชันพร้อมกัน ให้หยุดแล้วตรวจก่อน
3. **CI ที่ไม่ใช้ `--locked` คือความเสี่ยง** ถ้า runner สร้าง lockfile ใหม่ทุกครั้ง ชั่วโมงที่เกิดเหตุจะกลายเป็นชั่วโมงที่ทุก pipeline เสี่ยงเท่ากันหมด
4. ถ้าสงสัยว่าติด ให้ตรวจ `Cargo.lock`, pin กลับไปเวอร์ชันปลอดภัย (`arrayref = "=0.3.9"`), ลบไฟล์ที่ถูก drop, **rotate credential ทุกตัวที่เครื่องนั้นเข้าถึงได้** และ rebuild artifact จากเครื่องที่สะอาด และ RustSec ออก advisory ไว้ครบคือ [RUSTSEC-2026-0260](https://rustsec.org/advisories/RUSTSEC-2026-0260.html) (arrayref), 0266 (internment), 0262 (append-only-vec) และ 0265 (proc-macro1) ส่วน NVD ให้คะแนน CVSS 9.8 กับ [CVE-2026-77649](https://nvd.nist.gov/vuln/detail/cve-2026-77649) ด้วย

## Rustfmt

rustfmt เป็นเครื่องมือจัดรูปแบบโค้ดตาม style guide กำหนดค่าผ่าน `rustfmt.toml` ได้ และหนังสือจัดให้เป็น **Recommendation (DENV-FORMAT)** ไม่ใช่ Rule นั่นคือควรใช้ เพื่อให้ codebase มี style ตรงกัน

เรื่องที่หนังสือย้ำหนักกว่าคือ `cargo fix` ซึ่งอยู่บ้านเดียวกัน เพราะมันแก้โค้ดให้อัตโนมัติ แต่ให้หลักประกันน้อยมากกับความถูกต้องของสิ่งที่แก้ โดยเฉพาะ `--edition-idioms` ที่มีเคสเปลี่ยน semantics ของโปรแกรมจริง ๆ หนังสือจึงออก Rule **DENV-AUTOFIX** ว่าการแก้โค้ดอัตโนมัติทุกชนิด *ต้อง* ถูกตรวจโดยนักพัฒนาก่อน commit

### มุมความปลอดภัยจริง ๆ ของ rustfmt

rustfmt ไม่ได้ทำให้โค้ดปลอดภัยขึ้นโดยตรง แต่คุณค่าด้านความปลอดภัยของมันคือ **การทำให้ diff รีวิวง่าย** ซึ่งสำคัญกว่าที่คิด เพราะ control ที่แข็งแรงที่สุดของเราคือ code review และ review จะพลาดง่ายมากถ้า diff เต็มไปด้วยการจัดรูปแบบที่สลับไปมา

จุดที่พลาดบ่อยในปี 2026 คือเรื่อง **style edition** Rust 2024 เพิ่ม `style_edition` ให้ควบคุมรูปแบบแยกจาก edition ของภาษา แต่ค่า default ของ `rustfmt` เมื่อรันตรง ๆ คือ 2015 ในขณะที่ `cargo fmt` อ่าน edition จาก `Cargo.toml` ผลคือ format-on-save ใน editor กับ `cargo fmt --check` ใน CI อาจได้ผลไม่ตรงกัน แล้วเราก็เสียเวลาไปกับ diff ปลอม ๆ แทนที่จะได้ตรวจการเปลี่ยนแปลงจริง

ทางแก้คือ pin ทั้ง `edition` และ `style_edition` ไว้ใน `rustfmt.toml` ตามที่[คู่มือ Rust 2024 แนะนำ](https://doc.rust-lang.org/stable/edition-guide/rust-2024/rustfmt-style-edition.html) และอย่าเปิด `unstable_features = true` เพื่อใช้ option ที่ยังไม่ stable เพราะเท่ากับผูกโปรเจกต์เข้ากับ nightly ซึ่งขัดกับ Rule DENV-STABLE โดยตรง

### จัดรูปแบบให้เป็นด่านตรวจได้จริง

```toml
# rustfmt.toml
edition = "2024"
style_edition = "2024"
max_width = 100
```

- ใช้ `cargo fmt --all -- --check` เป็นด่านแรกของ CI (โปรเจกต์นี้ก็ใช้แบบนั้น) ถ้าไม่ผ่านคือยังไม่ต้องคุยเรื่องอื่น
- ถ้าอยากใช้ option ที่ยัง unstable ให้ตั้งใจตัดสินใจว่าโปรเจกต์ยอมใช้ nightly เฉพาะคำสั่ง format หรือไม่ แล้วเขียนไว้ใน README ให้ทีมรู้ตรงกัน
- แยก commit รูปแบบออกจาก commit logic เมื่อทำได้ เพื่อให้คนรีวิวเห็นการเปลี่ยนแปลงจริงใน diff สั้น ๆ
- และอย่าลืม Rule DENV-AUTOFIX: `cargo fix` โดยเฉพาะ `--edition` และ `--edition-idioms` ต้องมีคนอ่านทุกบรรทัดก่อน commit ไม่ใช่เชื่อ rustfix

## Clippy

Clippy เป็น linter ที่รวม lint หลายร้อยตัว แบ่งกลุ่มตามชนิดปัญหา และหนังสือออก Rule **DENV-LINTER** ตรง ๆ ว่าต้องใช้อย่างสม่ำเสมอในงานที่ต้องการความปลอดภัย ส่วนคำแนะนำของ clippy เอง โดยเฉพาะกลุ่ม `clippy::nursery` ที่ยังพัฒนาไม่นิ่ง ต้องถูกตรวจก่อน commit ทุกครั้ง

### ลินต์ที่เกี่ยวกับความปลอดภัยโดยตรง

Clippy มีกลุ่ม lint ที่ทำงานต่างกัน และกลุ่มที่ deny-by-default คือ `clippy::correctness` ซึ่งหมายความว่าโค้ดผิดชัด ๆ จะไม่ผ่าน compile อยู่แล้ว ส่วน `clippy::suspicious` จับโค้ดที่ดูเหมือนไม่ได้ตั้งใจ ที่เหลือเป็น `pedantic`, `restriction` และ `nursery` ที่เราเลือกเปิดรายตัวได้

สำหรับงานความปลอดภัย ลินต์ที่ผมว่าคุ้มที่สุดคือ

- **`clippy::unwrap_used` / `clippy::expect_used`** กัน panic ที่กลายเป็น denial of service
- **`clippy::indexing_slicing`** กันการ index โดยไม่ตรวจ bounds
- **`clippy::arithmetic_side_effects`** กัน overflow/underflow แบบเงียบ ๆ ใน release build
- **`clippy::undocumented_unsafe_blocks`** บังคับให้ทุก unsafe block มีคำอธิบายว่าทำไมมัน sound ซึ่งมักทำให้เราพบว่า assumption ของตัวเองผิดตั้งแต่ตอนเขียนคอมเมนต์
- **`clippy::macro_metavars_in_unsafe`** จับ macro ที่ยอมให้ผู้เรียกแอบใส่โค้ดเข้า unsafe block โดยไม่ต้องเขียน unsafe เอง
- **`clippy::unsafe_derive_deserialize`** เตือนการ derive `Deserialize` ให้ type ที่มี invariant ด้านความปลอดภัย เพราะ deserialization สร้างค่าจากข้อมูลที่ไม่เชื่อถือ

เปิดใช้ได้ทั้งใน `Cargo.toml` และ `clippy.toml` แต่ที่สำคัญคือ Clippy ตรวจแค่ workspace ของเรา ไม่สแกน dependency ให้ ดังนั้นมันเป็นด่านของ**โค้ดที่เราเขียน** ไม่ใช่เครื่องมือกัน supply chain

### 2026: safety-critical lints กำลังจะเข้า Clippy

ปีนี้มี[เป้าหมายที่ได้รับการรับเข้า](https://goals.rust-lang.org/2026/safety-critical-lints-in-clippy.html)ให้ Safety-Critical Rust Consortium (SCRC) เข้ามาเพิ่ม lint สำหรับงาน functional safety ลงใน Clippy โดยตรง เป้าหมายคือ 50–200 lint ใน 1–2 ปีข้างหน้า จากการเทียบ MISRA C:2025 กับภาษา Rust ซึ่งมี guideline ที่นำมาใช้ตรง ๆ ได้ 68 ข้อ ในจำนวนนั้น 13 ข้อมี lint ใน rustc/Clippy อยู่แล้ว และราว 50 ข้อต้องเขียน lint ใหม่ ทีม Clippy จะเป็นพี่เลี้ยงให้ผู้เขียน lint จาก consortium แล้วค่อย ๆ ส่งต่อ review ให้ SCRC ดูแลเอง งานนี้จะทำให้การทำ safety และ security ของ Rust สองฝั่งนี้ได้เครื่องมือร่วมกัน

### ตั้งค่าลินต์ให้เป็นจริง

```toml
# Cargo.toml
[lints.clippy]
unwrap_used = "deny"
undocumented_unsafe_blocks = "deny"
arithmetic_side_effects = "warn"
indexing_slicing = "warn"
```

```shell
$ cargo clippy --all-targets --all-features -- -D warnings
```

- รัน `cargo clippy --fix` ได้ แต่ตาม Rule DENV-AUTOFIX ต้องอ่าน diff ก่อน commit เสมอ
- ลินต์แนว restriction เหมาะกับการ cherry-pick ไม่ใช่เปิดทั้งกลุ่ม เพราะจะจมอยู่กับ warning จนไม่มีใครอ่าน
- ถ้าต้องการจับ UB ที่ static analysis มองไม่เห็น ลองรัน test ผ่าน [`cargo-careful`](https://github.com/RalfJung/cargo-careful) ที่เปิด runtime check เพิ่มจาก Miri บนโค้ดจริง ซึ่งเป็นคู่หูที่ดีของ Clippy

## แล้วมันหมายความว่าอะไรกับเรา

หลังไล่อ่านบทนี้พร้อมอัปเดตปี 2026 สิ่งที่เห็นชัดคือคำแนะนำของ ANSSI ไม่ได้ล้าสมัยเลย ทุกข้อล้วนถูกเหตุการณ์จริงตอกย้ำ แต่ช่องว่างของปี 2026 อยู่ที่สิ่งที่หนังสือเล่มนี้เขียนไว้สั้น ๆ ว่า TOFU และ build script นั่นแหละ

ถ้าจะสรุปเป็นชุดปฏิบัติที่ทำได้จริงวันนี้

- **pin ให้มากที่สุด** ทั้ง `rust-toolchain.toml`, `Cargo.lock`, เวอร์ชันของ GitHub Action (แบบ commit SHA) และ `rustfmt.toml` ให้ตรงกันทั้งเครื่องและ CI
- **บังคับ `--locked` ใน CI** และอ่าน diff ของ lockfile ทุกครั้งที่อัปเดต dependency
- **ให้ cargo audit / cargo deny เป็นด่านบังคับ** และติดตาม RustSec RSS เพราะนโยบายใหม่ของ crates.io ส่งสัญญาณผ่านช่องทางนั้นเป็นหลัก
- **ถือว่า build script คือโค้ดที่รันบนเครื่องเรา** ตรวจ `build.rs` และ `build-dependencies` ของทุก dependency ใหม่ และถ้าสงสัยให้ build ใน container/VM หรือ sandbox
- **ให้ clippy กับ rustfmt ทำงานอัตโนมัติใน CI** แล้วเก็บสมองไว้ใช้กับการ review logic และ unsafe ที่เหลือ
- **อย่ารอให้ rustup ตรวจลายเซ็นเสร็จ** เพราะ TUF mirroring และ signing เป็นงานระยะยาว ระหว่างนี้ใช้ installer ที่ตรวจ GPG ได้ หรือ package manager ของ OS และอัปเดต stable สม่ำเสมอ

สำหรับซอฟต์แวร์สุขภาพที่ผมเขียนอยู่ ข้อสุดท้ายเกี่ยวกับ build script นี้สำคัญเป็นพิเศษ เพราะเครื่องที่ build คือเครื่องที่มี credential ของระบบจริงอยู่เต็มไปหมด และการ rotate credential ทั้งหมดเพราะเผลอ `cargo update` ในวันที่โชคร้าย ก็ไม่ใช่ราคาที่เราอยากจ่ายเท่ากับการใส่ `--locked` ตั้งแต่แรก

สุดท้ายนี้ หนังสือเล่มนี้เขียนไว้ตั้งแต่ยุคที่ rustup ยังไม่ตรวจลายเซ็น และทุกวันนี้มันก็ยังไม่ตรวจ ซึ่งอาจฟังดูน่าผิดหวัง แต่สิ่งที่เปลี่ยนไปคือเรามีเครื่องมือมากขึ้นเรื่อย ๆ ในการจำกัดวงความเสียหาย เมื่อ build script ตัวหนึ่งตัดสินใจทำสิ่งที่ไม่ควรทำระหว่าง `cargo build` ของเราครับ

---

**Credit & Reference:**
- [Secure Rust Guidelines: Development environment (ANSSI)](https://anssi-fr.github.io/rust-guide/devenv.html) (บทต้นทาง, Rules DENV-STABLE, DENV-TIERS, DENV-CARGO-LOCK, DENV-CARGO-OPTS, DENV-CARGO-ENVVARS, DENV-FORMAT, DENV-LINTER, DENV-AUTOFIX)
- [Rustup Security](https://rust-lang.github.io/rustup/security.html) และ [rustup update: แผนรุ่น 1.30](https://blog.rust-lang.org/inside-rust/2026/07/03/rustup-update-1.30/)
- [Rust Project Goal 2026: Implement Verifiable Mirroring Prototype (TUF)](https://goals.rust-lang.org/2026/mirroring.html)
- [Ferrocene: qualified Rust toolchain](https://ferrocene.dev/) และ [Ferrocene 26.05.0](https://ferrous-systems.com/blog/ferrocene-26-05-0/)
- [Supply chain attack on arrayref (Rust Blog, 20 ส.ค. 2026)](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/)
- [JFrog: Compromised Rust crates on crates.io](https://research.jfrog.com/post/arrayref-proc-macro1-crates-io/), [StepSecurity: arrayref supply chain attack](https://www.stepsecurity.io/blog/arrayref-rust-crate-supply-chain-attack), [Wiz: arrayref และความเชื่อมโยงกับ DPRK](https://www.wiz.io/blog/rust-supply-chain-attack-on-arrayref-significant-overlap-with-dprk-campaigns)
- [RustSec advisories: RUSTSEC-2026-0260](https://rustsec.org/advisories/RUSTSEC-2026-0260.html), [RUSTSEC-2026-0266](https://rustsec.org/advisories/RUSTSEC-2026-0266.html), [RUSTSEC-2026-0262](https://rustsec.org/advisories/RUSTSEC-2026-0262.html), [RUSTSEC-2026-0265](https://rustsec.org/advisories/RUSTSEC-2026-0265.html) และ [NVD CVE-2026-77649](https://nvd.nist.gov/vuln/detail/cve-2026-77649)
- [Security advisory for Cargo (CVE-2026-33055/33056)](https://blog.rust-lang.org/2026/03/21/cve-2026-33056/)
- [crates.io development update ม.ค. 2026](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/) และ [ก.ค. 2026](https://blog.rust-lang.org/2026/07/13/crates-io-development-update/), [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
- [นโยบายแจ้งเตือนมัลแวร์ของ crates.io (ก.พ. 2026)](https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update/)
- [Cargo: host.runner สำหรับห่อ build script execution](https://github.com/rust-lang/cargo/issues/16591) และ [ข้อเสนอ explicit opt-in สำหรับ dependency build scripts](https://github.com/rust-lang/cargo/issues/17408)
- [cargo-audit/RustSec](https://rustsec.org/), [cargo-vet](https://mozilla.github.io/cargo-vet/), [cargo-crev](https://github.com/crev-dev/cargo-crev), [cackle](https://github.com/cackle-rs/cackle), [cargo-careful](https://github.com/RalfJung/cargo-careful)
- [Clippy lint categories](https://doc.rust-lang.org/stable/clippy/lints.html) และ [Rust Project Goal 2026: Safety-Critical Lints in Clippy](https://goals.rust-lang.org/2026/safety-critical-lints-in-clippy.html)
- [Rustfmt style edition (Rust 2024 Edition Guide)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/rustfmt-style-edition.html)
