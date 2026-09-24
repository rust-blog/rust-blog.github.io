---
title: "เมื่อ AI เขียน Rust editor 17,000 บรรทัด"
date: "2026-09-24"
description: "เรียบเรียงและต่อยอดจากบทความ Synthetic sagas ของ Jamie Brandon"
tags: [rust, testing, ai]
author: "suradet-ps"
---

บทความนี้ผมเรียบเรียงและต่อยอดจาก [Synthetic sagas](https://www.scattered-thoughts.net/writing/synthetic-sagas/) ของ [Jamie Brandon](https://www.scattered-thoughts.net/) (เผยแพร่ 20 กันยายน 2026) ซึ่งเล่าประสบการณ์ rewrite text editor ส่วนตัวชื่อ [focus](https://github.com/jamii/focus) เองใหม่จากศูนย์ด้วย AI แทบทั้งหมด และผมได้แทรกเครื่องมือใน ecosystem ของ Rust ที่ผมไปค้นคว้าเพิ่มในแต่ละหัวข้อ เพื่อให้หยิบไปใช้กับโปรเจกต์ของตัวเองได้จริง

Jamie เริ่มโปรเจกต์นี้เป็น learning project แต่ตอนนี้เขาเลิกใช้ editor เวอร์ชันเก่าแล้ว เพราะเวอร์ชันใหม่ที่ AI ช่วยเขียนนั้นลื่นกว่าเดิม แถมยังมีฟีเจอร์มากกว่าตัวเก่า โค้ดรวมกันประมาณ 17,000 บรรทัด (13,831 บรรทัดใน `focus-core` + 3,661 บรรทัดใน `focus`) งานส่วนใหญ่ทำโดย opus 5 ผ่าน claude code บน subscription ราคา $20/เดือน

## AI ผู้ช่วยที่เริ่ม "เถียงเป็น"

ในบทความก่อนหน้า [Artificial adventures](https://www.scattered-thoughts.net/writing/artificial-adventures/) Jamie ยังไม่กล้า merge โค้ดที่ AI เขียนโดยไม่อ่านละเอียด แต่ตอนนี้เขา commit โค้ดที่ AI สร้างได้หลัง manual test เล็กน้อยและ glance ผ่าน diff เท่านั้น ซึ่งเขาเองก็ไม่แน่ใจว่าที่ทำได้เพราะ architecture กับ testing strategy ที่วางไว้ตั้งแต่แรก หรือเพราะโมเดลเก่งขึ้นกันแน่

แต่ที่แน่ ๆ คือโมเดลเก่งขึ้นจริง เขาไม่ต้อง handhold มากเหมือนเดิม โมเดลทำตาม instruction เรื่องการเขียน test ได้จริง และมักจับ edge case ที่เขาคิดไม่ถึง บางครั้งถึงขั้น push back เมื่อ instruction ผิด โดย opus เขียนไว้ประมาณว่า *"I didn't implement your instructions literally because that would have caused this obvious issue"* แล้วก็ถูกของมัน

แต่กรณีแบบนี้ยังหาได้ยาก ส่วนใหญ่โมเดลยังอยู่ในโหมด **evil genie** คือพยายามฝ่าทุกอุปสรรคเพื่อทำตามคำสั่งให้จบ แทนที่จะบอกว่าเราสั่งผิด Jamie เลยเปลี่ยนนิสัยการเขียน prompt จาก "Do X" เป็น **"I want to do X. Any questions?"** เพื่อเปิดช่องให้โมเดลถามก่อนลงมือ ซึ่งช่วยดักปัญหาได้หลายครั้ง

> ข้อสังเกตของผม เทคนิคนี้ใช้ได้ผลก็ต่อเมื่อเรามีคำตอบให้มันจริง ๆ ถ้า instruction กำกวมแล้วเราเปิดให้ถาม โมเดลจะถามถูกจุด แต่ถ้าเราไม่รู้คำตอบเอง มันก็ได้แค่เดาเหมือนเดิม

## deterministic simulations

ฟีเจอร์ที่ใหญ่ที่สุดของรอบนี้คือ tests

โค้ดถูกแบ่งเป็นสอง crate โดยพยายามย้าย logic ให้อยู่ใน `focus-core` ให้มากที่สุด มันรับ input events แล้วคืนรายการ characters/rects ที่ต้อง render โดยคุยกับโลกภายนอกผ่าน `&mut dyn IO` ส่วน crate `focus` เก็บ implementation จริงของ `IO` trait, CLI และ daemonization

```sh
> scc focus-core/src
Language                 Files     Lines   Blanks  Comments     Code Complexity
Rust                        34     13831     1067      1856    10908       1202

> scc focus/src
Language                 Files     Lines   Blanks  Comments     Code Complexity
Rust                         8      3661      288       720     2653        257
```

ผลตอบแทนคือเขียน end-to-end test ให้ `focus-core` ได้ง่ายมาก (แม้จะน่าเบื่อ) และที่สำคัญคือ **โมเดลอยากเขียน unit test มาก** ถ้าเราไม่ให้ stable public interface ที่มันทดสอบได้ มันจะไปเจาะรูตาม boundary ของเราทุกจุด

```rust
#[test]
fn dir_picker_ctrl_enter_descends_into_selected_dir() {
    let (mut app, mut io, window_id) = common::scratch_app();
    insert_file(&mut io, "/proj/src/main.rs", "");

    common::control_key(&mut app, &mut io, window_id, Key::Character("m"));
    common::tick(&mut app, &mut io);

    // ctrl+enter descends into the first (and only) listed dir (proj/).
    common::control_key(&mut app, &mut io, window_id, Key::Named(NamedKey::Enter));
    common::tick(&mut app, &mut io);

    // The path editor now shows /proj/.
    assert_eq!(buffer_text(&app, DIR_PATH), "/proj/");
    // The list now shows src/.
    assert!(buffer_text(&app, DIR_LIST).contains("src/"));
    app.assert_invariants();
}
```

Jamie บอกตรง ๆ ว่า test พวกนี้ส่วนใหญ่เป็น "slop" เขาแทบไม่เคยอ่านมันด้วยซ้ำ แต่เขาก็เห็นมันจับ regression ได้และทำให้เจ้า AI รู้ตัวว่าทำอะไรพัง

นอกจากนั้นยังมี fuzzer ที่ทำงานง่ายมาก คือเปิดแอป ยิง random keypress หลายพันครั้ง แล้วเรียก `app.assert_invariants()` แม้ coverage จะต่ำ แต่ก็ยังขุด bug ออกมาได้เรื่อย ๆ หลังจากใช้งานจริงมาหลายสัปดาห์ เขาไม่เจอ bug หรือ crash เลย

แน่นอนว่าความพยายามเรื่อง testing ต้องขึ้นกับ risk และ impact สำหรับโปรเจกต์ที่มีผู้ใช้คนเดียวและไม่มีข้อมูลสำคัญที่เสี่ยงหาย การพึ่ง slop แล้วค่อยแก้เมื่อเจอ bug จริงก็คุ้มกว่า แต่เขาเองยังไม่รู้ว่าถ้าต้อง ship ของสำคัญจริง ๆ วิธีคิดจะเปลี่ยนไปแค่ไหน

ข้อจำกัดของวิธีนี้คือ `IO` trait เป็นแค่ทางออกแบบบ้าน ๆ ใน Rust เราไม่มีทาง inject mock ให้ stdlib ได้อย่างสมเหตุสมผล และไลบรารีส่วนใหญ่ก็ไม่ได้เขียนแบบ sans-io ทำให้เราเอามันมาใช้กับ `IO` trait ของเราไม่ได้ ผลคือการเรียกไลบรารีจำนวนหนึ่งถูกบังคับให้อยู่นอก `focus-core` ซึ่ง test ยาก เช่น daemonization ที่ regress บ่อย Jamie มีความคิดคร่าว ๆ ว่าจะใช้ WASI หรือ hypervisor ทำ [Antithesis](https://antithesis.com/)-lite ให้ทุกอย่างเข้าไปอยู่ใน simulatable boundary เดียว แต่ยังไม่ใช่ roadmap ตอนนี้

### ต่อยอด: Rust มีอะไรให้ใช้บ้าง

- **sans-io** เป็น pattern ที่แยก logic ของ protocol ออกจาก I/O ทั้งหมด (ชื่อเดิมมาจากฝั่ง Python อย่าง `h11`/`wsproto`) ใน Rust มี crate [`sansio`](https://github.com/webrtc-rs/sansio) ที่ให้ `Protocol` trait แบบ push-pull และไลบรารีระดับ production หลายตัวก็แยก core แบบไม่แตะ I/O ไว้แล้ว เช่น rustls และ quinn ทำให้เราทดสอบ logic ได้โดยไม่ต้องมี socket จริง
- **Deterministic simulation** สำหรับงาน async/distributed มี [`turmoil`](https://github.com/tokio-rs/turmoil) จากทีม tokio ที่รันหลาย host ใน thread เดียว ควบคุม network ได้ทั้ง drop/partition/delay และ [`madsim`](https://github.com/madsim-rs/madsim) ที่ได้แรงบันดาลใจจาก FoundationDB และถูกใช้จริงใน RisingWave ทั้งคู่ให้ seed ที่ทำให้ reproduce bug ได้แบบ deterministically
- **WASI + wasmtime** เป็นอีกเส้นทางหนึ่งที่ตรงกับความคิดเรื่อง Antithesis-lite ของ Jamie คือยกทั้งโปรแกรมเข้า sandbox ที่คุมเวลาและ I/O ได้ ทำให้ behavior ซ้ำได้ทุกครั้ง และ canary ตรวจ resource limit ได้
- ถ้าอยากได้ property/fuzz จริงจังกว่า fuzzer เขียนเอง มี [`proptest`](https://github.com/proptest-rs/proptest) (shrinking counterexample ให้), [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) และ [`bolero`](https://github.com/camshaft/bolero) ที่รัน fuzz และ property test ผ่าน interface เดียวกันได้

## avoiding decay

คำถามที่ Jamie กำลังหาคำตอบคือ "เราจะรีวิวโค้ดน้อยลงได้แค่ไหน" ซึ่งขึ้นอยู่กับว่าเราวาด boundary รอบ behaviour ได้แน่นแค่ไหน

ตัวอย่างที่เขายกมาคือ tokenizer ของ Python ที่ AI เขียน เขาแทบไม่ต้องอ่านเลย เพราะมันทำตาม highlighting interface เดิม control flow เป็น loop ใหญ่ที่เห็นได้ชัดว่า linear ตามขนาด buffer และโค้ดอยู่ใน module เดียว ผลกระทบถ้าพังจึงจำกัด เขาแค่เปิดไฟล์ Python ดูว่า highlight ดูสมเหตุสมผลแล้ว commit ต่อ

แต่การเปลี่ยนแปลงส่วนใหญ่ไม่ได้ง่ายแบบนั้น เขากลัวว่าจะ vibecode ตัวเองเข้ากับกองขยะ โดยที่ความพังค่อย ๆ สะสมทีละนิดและมองไม่ออกใน commit ใหญ่ ๆ เขาเลยทดลองสร้าง **snapshot หลายมุมของโค้ด**

API snapshots ตัวหนึ่ง parse โค้ดทั้งโปรเจกต์แล้ว snapshot public กับ internal interface ออกมา หน้าตา public interface บางส่วนเป็นแบบนี้

```rust
## pub mod focus_core::buffer

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct BufferId(/* private fields */);

pub struct Buffers {
    /* private fields */
}

impl Buffers {
    pub fn keys(&self) -> impl Iterator<Item = BufferId> + '_;
}

pub fn from_file(app: &mut App, io: &mut dyn IO, absolute_path: PathBuf) -> BufferId;

impl BufferId {
    pub fn text(self, app: &App) -> &BStr;
}
```

ต่อให้เขาแค่ glance ผ่าน commit เขาจะอ่าน diff ของ interface snapshot อย่างละเอียด เพื่อจับ leaky abstraction ที่แอบโผล่มา

### ต่อยอด: snapshot API ด้วยเครื่องมือสำเร็จรูป

- [`cargo-public-api`](https://github.com/cargo-public-api/cargo-public-api) ใช้ rustdoc JSON สร้างรายการ public API ทั้งหมด แล้วเขียนเป็น snapshot test ได้เลย โดยมี `assert_eq_or_update` ให้อัปเดต snapshot เมื่อตั้งใจเปลี่ยน API จริง ๆ เหมาะกับการรันใน CI เพื่อกัน breaking change ที่หลุดมาโดยไม่ตั้งใจ
- [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks) ตรวจว่า API ที่เปลี่ยนไปละเมิด semver หรือไม่ (เช่น ลบ public item ใน minor version) ใช้เป็น gate ใน CI ได้ทันที
- [`insta`](https://github.com/mitsuhiko/insta) เป็น snapshot testing crate ที่เป็นที่นิยมที่สุดใน Rust ใช้เก็บ snapshot อะไรก็ได้ (ไม่จำกัดแค่ API) พร้อม review UI ใน terminal และ `cargo insta review`

Jamie เขียนตัว parse เองเพราะอยากได้ snapshot ทั้ง public และ internal interface ในรูปแบบที่อ่านง่าย แต่สำหรับโปรเจกต์ทั่วไป การเริ่มจาก `cargo-public-api` + `insta` นั้นเร็วและพอเพียงแล้ว

เขายังมี test แบบ [cackle](https://github.com/cackle-rs/cackle) ที่คอยหาการเรียก IO function โดยไม่ตั้งใจจากใน `focus-core` ซึ่งเป็นวิธีเฉพาะกิจ ไม่ได้รัดกุมอะไร น่าจะ bypass ได้ง่าย และต้องปิด crate บางตัวที่ build script มีปัญหา พร้อม allowlist การ seed hashmap จาก `/dev/random` แต่ก็ยังดีกว่าไม่มีอะไรเลย

> **cackle คืออะไร**: จริง ๆ แล้ว cackle เป็น "code ACL checker" สำหรับ Rust ที่วิเคราะห์ว่าแต่ละ crate ใน dependency tree เรียก API อะไรบ้าง (และแซนด์บ็อกซ์ build script ได้ด้วย) Jamie ใช้มันในลักษณะกำหนดกฎว่า `focus-core` ห้ามแตะ IO API แล้วให้ CI ตรวจ ถ้าใครเผลอเรียก ก็จะเห็น backtrace ว่าถูกเรียกจากไหน
>
> การติดตั้งคือ `cargo install --locked cargo-acl` แล้วรัน `cargo acl` ข้อจำกัดคือปัจจุบันรองรับ Linux เป็นหลัก และมี [cackle-action](https://github.com/cackle-rs/cackle-action) สำหรับ GitHub Actions ด้วย

ใน fuzzer เองก็มี assertion ว่า input จาก fuzzer ต้องไม่ทำให้ frame budget แตก และมี unit test คร่าว ๆ สำหรับ asymptotic behaviour แบบนี้

```rust
#[test]
fn rust_highlighting_is_linear() {
    check("speed.rs", include_str!("fixtures/indent.rs"));
}

/// Open a file of `sample` repeated to two sizes, and require that the
/// bigger one costs about what its size says it should.
fn check(name: &str, sample: &str) {
    let small = time_open(name, &repeat(sample, SMALL));
    let large = time_open(name, &repeat(sample, LARGE));

    let growth = large.as_secs_f64() / small.as_secs_f64();
    let rate = (LARGE as f64 / 1e6) / large.as_secs_f64();
    let report =
        format!("{name}: {SMALL} bytes in {small:.2?}, {LARGE} in {large:.2?} ({rate:.1} MB/s)");

    assert!(
        growth <= MAX_GROWTH,
        "{report}\n{}x the text took {growth:.1}x the time, which is not linear",
        LARGE / SMALL,
    );
    assert!(
        rate >= MIN_MB_PER_SECOND,
        "{report}\nslower than {MIN_MB_PER_SECOND} MB/s, which is an order of magnitude off",
    );
}
```

test แบบนี้จำเป็นจริง ๆ เพราะโมเดลรุ่นล่าสุดยังเขียนโค้ดที่บังเอิญเป็น quadratic ได้บ่อย Jamie สนใจว่าเราจะทำให้ test แบบนี้ robust ขึ้นได้อย่างไร เพราะเรามีทฤษฎีและวิธีปฏิบัติเยอะมากในการระบุ logical behaviour ของโค้ด แต่เรื่อง performance characteristic เรามีน้อยกว่ามาก โมเดลที่เขา fuzz อยู่ตอนนี้คือ *"เปิดได้ไม่เกิน 16 หน้าต่าง ไฟล์ละไม่เกิน 200kb และ input เดียวต้องประมวลผล+render ไม่เกิน 16ms บน laptop เครื่องนี้"* แต่เขาอยากได้อะไรที่ละเอียดกว่านั้น

### ต่อยอด: วัด performance ให้ deterministic

การวัดด้วย wall-clock เป็นปัญหามาตรฐานบน CI เพราะเครื่องมี noise สูง สิ่งที่ช่วยได้คือ

- [`gungraun`](https://github.com/gungraun/gungraun) (เดิมชื่อ iai-callgrind ก่อนเปลี่ยนชื่อตอนเวอร์ชัน 0.17) วัดผ่าน Valgrind's Callgrind, Cachegrind และ DHAT ได้ instruction count, cache hit และ estimated cycles ที่เสถียรพอจะตั้ง regression threshold ได้ เช่น "ถ้า `Ir` (instructions retired) เพิ่มเกิน 10% ให้ CI fail" ซึ่งแม่นกว่าการวัดเวลาและเหมาะกับ CI มาก
- [`divan`](https://github.com/nvzqz/divan) เบาและใช้ง่าย รองรับ counter (เช่น bytes processed), allocation profiling ผ่าน `AllocProfiler` และมี flag `--test` สำหรับรัน benchmark ทั้งหมดหนึ่งรอบเพื่อเช็คว่าไม่ panic ก่อนขึ้น CI
- [`criterion`](https://github.com/bheisler/criterion.rs) เหมาะกับ benchmark บนเครื่องตัวเองที่ต้องการการวิเคราะห์เชิงสถิติ
- [`dhat`](https://docs.rs/dhat) ช่วยตามหาว่า allocation โผล่จากไหน ซึ่งเป็นสาเหตุคลาสสิกของ latency ที่ไม่คงที่

ที่ยังขาดอยู่คือภาษากลางสำหรับระบุ performance specification แบบเดียวกับที่ `assert_eq!` ระบุ correctness ได้ งานอย่าง Jamie จึงต้องเขียน test แบบ growth ratio เอง ซึ่งก็เป็นช่องว่างของวงการที่ยังน่าลงไปเล่นอยู่

## pointer-free functions

ส่วนนี้เกี่ยวกับ AI แค่ทางอ้อม แต่เป็นผลพลอยได้จากโปรเจกต์นี้ Jamie ลงเอยด้วย architecture แบบ relational/data-oriented ที่เก็บทุกอย่างแยก collection แล้วอ้างกันด้วย handle

```rust
pub struct Buffers {
    pub(crate) buffer_count: usize,

    source: Map<BufferId, Source>,
    text: Map<BufferId, BString>,
    highlight: Map<BufferId, Highlight>,
    newlines: Map<BufferId, Vec<usize>>,
    last_modified_time: Map<BufferId, Duration>,
    undos: Map<BufferId, Vec<Vec<Vec<Edit>>>>,
    doing: Map<BufferId, Vec<Vec<Edit>>>,
    redos: Map<BufferId, Vec<Vec<Vec<Edit>>>>,

    // Can be set by editor.
    pub(crate) last_center_offset: Map<BufferId, usize>,
}
```

`Map` ตรงนี้เป็นแค่ wrapper บาง ๆ รอบ `Vec` ที่ใช้ typed key (เช่น `BufferId`) แทน integer

เขาเล่าว่าลองจัดโครงสร้างแบบอื่นมาหลายแบบแล้ว ทุกแบบลงเอยด้วย ownership/lifetime ที่ต้องใช้ judgement แก้ ซึ่งทำให้ AI หลงไปขุด rabbit hole และเขียนโค้ดประหลาดออกมา แต่พอใช้ handle ชีวิตง่ายขึ้นมาก และภายใน function เดียวกัน rust ก็ฉลาดพอจะรู้ว่า `buffers.text` กับ `buffers.source` ไม่ได้ยืมทับกัน

### ต่อยอด: crate ที่ทำงานนี้ให้

- [`slotmap`](https://docs.rs/slotmap) ให้ key ที่ unique ต่อการ insert ทุกครั้ง ใช้ `new_key_type!` สร้าง key ชนิดใหม่แยกต่อ collection ทำให้ไม่เผลอเอา key ผิดชุดมาใช้ และมี secondary map ให้ associate ข้อมูลเพิ่มโดยไม่ต้อง hash
- [`generational-arena`](https://docs.rs/generational-arena) เพิ่ม generation counter เพื่อแก้ปัญหา ABA (key เก่าชี้ slot ที่ถูกรีไซเคิลแล้วไม่ได้) ซึ่งเป็นสิ่งที่ `Map<BufferId, T>` แบบ `Vec` ธรรมดาไม่ได้กันให้
- [`typed-index-collections`](https://docs.rs/typed-index-collections) ให้ `TiVec<K, V>` / `TiSlice<K, V>` ที่ mirror API ของ `Vec`/slice ทุกเมธอด และ [`index_vec`](https://docs.rs/index_vec) ให้ `IndexVec` พร้อม macro `define_index_type!` ซึ่งเบาที่สุดในกลุ่มนี้
- ตัวอย่างใช้งานจริงในระดับ production คือ [`naga::Arena`](https://docs.rs/naga) ของ wgpu ที่เก็บ IR ของ shader แล้วอ้างด้วย typed `Handle`
- แนวคิด generational index/ECS นี้มาจาก talk [Using Rust For Game Development](https://www.youtube.com/watch?v=aKLntZcp27M) ของ Catherine West ที่ RustConf 2018 ซึ่งยังเป็น talk ที่คุ้มในการกลับไปดูอยู่

ข้อแลกเปลี่ยนที่ต้องคิดคือ ถ้าใช้ generation key ทุกการ lookup จะต้องเช็ค generation เพิ่ม (ช้ากว่า index ตรง ๆ เล็กน้อย) แต่ได้ความปลอดภัยว่า handle ที่ตายแล้วใช้ไม่ได้อีก ส่วน `Map` ของ focus เลือกแบบง่ายที่สุดเพราะ handle ถูกจัดการ lifecycle ด้วยมืออยู่แล้ว

## single-threaded lifestyle

มีสองวิธีที่คนนิยมใช้กับอีเมล

- เช็คทุก 5 นาที หรือทุกครั้งที่รู้สึกกังวล แล้วตอบสั้นที่สุดเท่าที่จะทำให้เรื่องพ้นมือตัวเองได้
- เช็ควันละครั้ง แล้วตอบให้ละเอียดและรอบคอบพอที่คนส่งจะไม่ต้องตอบกลับ

Jamie เห็นคนพูดถึง agentic workflow แบบมี git worktree เยอะ ๆ agent หลายตัวรันขนานกันแล้ว merge ทีหลัง เขาลองแล้วบอกว่ามันเป็นความวุ่นวายที่เหนื่อยมาก เขาจึงยังทำแบบ single-threaded เป็นหลัก วิธีของเขาคือเขียน design ให้ละเอียดมากสำหรับงานก้อนใหญ่ ทำ synchronous pass ให้โมเดลถามคำถาม แล้วออกไปเดินเล่น อ่าน paper หรือทำงาน design ต่อ จนมือถือแจ้งเตือนว่าถึงเวลากลับมาดูผลลัพธ์

แบบนี้ productive พอ ๆ กันและสนุกกว่ามาก แถมได้แสงแดดเยอะกว่าปกติด้วย ถ้ามีหลายโปรเจกต์พร้อมกันหรือมี experiment ที่ขนานกันได้จริง เขาอาจลองผสม workstream แต่ก็ยังพยายามทำทีละอย่างให้จบก่อน เพื่อไม่ให้สมองพังกับ context switching

ผลคือ rhythm การทำงานเปลี่ยนไป เขาฟัง death metal และเพลง electronic จังหวะเร็วน้อยลง ฟัง grime กับ acid rock มากขึ้น และโฟกัสแบบเข้มข้นน้อยลง

## mass produced personalization

Jamie บอกว่ารู้สึกว่า calibration เรื่อง "โค้ดเยอะแค่ไหนถึงเรียกว่าเยอะเกิน" หายไปแล้ว editor ตัวนี้ใหญ่กว่าโปรเจกต์งานอดิเรกที่เขาเคยรับได้ แต่เขาไม่ต้อง maintain มันด้วยมืออีกแล้ว จึงยัง manageable มาก แถมใช้เวลาน้อยกว่าเวอร์ชันเก่ามากแต่มีฟีเจอร์มากกว่า เขาใช้ 15 นาทีออกแบบฟีเจอร์ ไปปีนหน้าผา 2-3 ชั่วโมง แล้วกลับมาใช้ 15 นาทีรีวิวและทดสอบระหว่างรอข้าวต้ม สองสามรอบก็เพียงพอจะเพิ่มฟีเจอร์ใหญ่ (vcs integration) ที่เวอร์ชันเก่าไม่เคยทำสำเร็จ

ภาพใหญ่ที่เขาเห็นคือ **quality-vs-effort frontier กำลังเลื่อน** ปลายหนึ่ง ทีมที่แคร์คุณภาพจริง ๆ จะมีแรงงานและเครื่องมือตรวจ bug มากขึ้น อีกปลายหนึ่ง งานที่เมื่อก่อนไม่มีทาง ship ได้ ตอนนี้กลับ ship ออกมาได้ แม้คุณภาพจะกลาง ๆ

เขาคาดว่าจะเจอซอฟต์แวร์คุณภาพต่ำล้นตลาด แต่การบ่นเรื่องคุณภาพโดยลืมว่าทางเลือกเดิมไม่ใช่ซอฟต์แวร์คุณภาพสูง แต่คือ "ไม่มีซอฟต์แวร์เลย" นั้นเป็นความผิดพลาด สองปีก่อนแทบไม่มีปัจเจกบุคคลคนไหนจ้าง programmer ได้ ตอนนี้ใครมี $20 ก็มี personal programmer ทำงานให้ แต่มันยังเป็นแบบ genie คือได้ผลดีก็ต่อเมื่อเรารู้จักขอสิ่งที่สมเหตุสมผล ซึ่งต้องรู้ก่อนว่าอะไรคือสิ่งที่ควรขอ

และแม้มีบัตเลอร์ AI การ deploy และ maintain stateful app ก็ยังยากสำหรับคนไม่เทคนิค แต่ถ้าจับโมเดลวันนี้ไปต่อกับระบบที่มีอยู่แล้วอย่าง Notion ที่จัดการ state, versioning, collaboration และ UI ให้เสร็จ ความฝันเรื่อง end-user programming ที่มีมานานก็อาจเป็นจริงได้ในที่สุด

Jamie บอกว่า AI ทำให้ทักษะหลายอย่างของเขาด้อยค่าลง แต่แทนที่จะโศกเศร้ากับสถานะเดิม เขาตื่นเต้นที่จะได้เห็นโลกที่ทุกคนออกแบบซอฟต์แวร์ได้

> *"I know that only artisanal hand-crafted goods truly have a soul, but I still love to ride my factory-produced bike along smooth factory-produced asphalt while using my factory-produced phone to take photos of murals painted with factory-produced spray-cans."*

## แล้วมันหมายความว่าอะไรกับเรา

ในมุมของผมซึ่งทำงานเภสัชกรรมและเขียน Rust เป็นงานอดิเรก บทความนี้ให้คำตอบกับคำถามที่ผมคิดบ่อยว่า "จะเชื่อโค้ดที่ AI เขียนได้ยังไง" คำตอบไม่ใช่ "อ่านทุกบรรทัด" และไม่ใช่ "เชื่อมัน" แต่คือ **สร้างระบบที่ทำให้ความผิดพลาดถูกจำกัดวงและตรวจจับได้** ซึ่งเป็นงานวิศวกรรมล้วน ๆ

- แบ่ง core ที่มี logic ออกจาก shell ที่แตะ I/O (sans-io)
- ให้ interface ที่ stable พอจะเขียน end-to-end test ได้
- fuzz + assert invariant แทนการเดา
- snapshot interface เพื่อจับ leaky abstraction
- วัด asymptotic และ performance budget เป็น test
- ใช้ handle + collection แยกส่วนแทน pointer graph ที่ซับซ้อน

สำหรับซอฟต์แวร์สุขภาพที่ผมเขียนอยู่ เครื่องมือชุดนี้ยิ่งสำคัญ เพราะ AI ทำให้เราสร้างได้เร็วขึ้น แต่ความรับผิดชอบต่อผู้ป่วยไม่ได้ลดลงตาม แนวคิดแบบ [property tests ที่ผมเขียนไว้ในบทความ favi-child](/post/favi-child-property-tests) จึงเป็นคู่หูที่ขาดไม่ได้ของ workflow แบบนี้

## อ้างอิง

- [Synthetic sagas - Jamie Brandon](https://www.scattered-thoughts.net/writing/synthetic-sagas/) (บทความต้นทาง)
- [Artificial adventures - Jamie Brandon](https://www.scattered-thoughts.net/writing/artificial-adventures/)
- [focus - minimalist text editor](https://github.com/jamii/focus)
- [turmoil](https://github.com/tokio-rs/turmoil), [madsim](https://github.com/madsim-rs/madsim), [Antithesis](https://antithesis.com/)
- [sansio](https://github.com/webrtc-rs/sansio), [cackle](https://github.com/cackle-rs/cackle), [cargo-public-api](https://github.com/cargo-public-api/cargo-public-api), [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks), [insta](https://github.com/mitsuhiko/insta)
- [gungraun (iai-callgrind)](https://github.com/gungraun/gungraun), [divan](https://github.com/nvzqz/divan), [criterion](https://github.com/bheisler/criterion.rs)
- [slotmap](https://docs.rs/slotmap), [generational-arena](https://docs.rs/generational-arena), [typed-index-collections](https://docs.rs/typed-index-collections), [index_vec](https://docs.rs/index_vec)

สุดท้าย Jamie เขียนปิดไว้ว่า *"I wouldn't mind slowing down."* ซึ่งผมเข้าใจดี เพราะบางทีการได้ออกไปเดินเล่นแล้วกลับมาอ่าน diff ก็ให้ผลดีกว่าการนั่งเฝ้าหน้าจอทั้งวันครับ
