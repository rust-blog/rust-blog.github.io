---
title: "awesome-rust ไม่มีแค่ตัวเดียว: รวม 70+ awesome list สาย Rust ทั้ง ecosystem"
date: "2026-09-16"
description: "สำรวจ awesome list ของ Rust ให้ครบทุกมุม ตั้งแต่ embedded, blockchain, AI, เกม, และความปลอดภัย"
tags: [rust, opensource]
author: "suradet-ps"
---

หลายคนน่าจะคุ้นเคยกับ [rust-unofficial/awesome-rust](https://github.com/rust-unofficial/awesome-rust) คลังรวบรวมโปรเจกต์ Rust ยอดฮิตอันดับต้น ๆ ของวงการกันเป็นอย่างดี แต่จริง ๆ แล้ว Rust ยังมี **awesome list อยู่อีกหลายสิบคลัง** ที่แตกแขนงตามสายงานเฉพาะทางและเฟรมเวิร์กต่าง ๆ ซึ่งหลายคลังมีคุณภาพยอดเยี่ยมมาก แต่กลับไม่ค่อยมีใครรู้ว่ามีอยู่

บทความนี้ผมเลยรวบรวมลิสต์ทั้งหมดเท่าที่ค้นเจอ ณ เดือนกันยายน 2026 มาให้เรียบร้อยแล้ว พร้อมสรุปจุดเด่นสั้น ๆ เพื่อช่วยให้เลือกหยิบไปใช้งานกันได้ง่ายขึ้นครับ

## awesome list คืออะไร

awesome list คือคลัง repository บน GitHub ที่ชาวคอมมูนิตี้ช่วยกันคัดสรรโปรเจกต์ เครื่องมือ และแหล่งข้อมูลดี ๆ ในหัวข้อใดหัวข้อหนึ่ง มารวบรวมไว้ในรูปแบบไฟล์ Markdown ธรรมดา โดยมี [sindresorhus/awesome](https://github.com/sindresorhus/awesome) ทำหน้าที่เป็นเหมือนสารบัญแม่ของทุกวงการ

ความน่าสนใจของฝั่ง Rust คือ นอกจากจะมีคลังกลางหลักแล้ว ยังมีคลังเฉพาะทางที่แยกย่อยตามกลุ่มผู้ใช้อย่างชัดเจน ตั้งแต่ระดับ embedded, blockchain, AI ไปจนถึงงานเฉพาะทางอย่างระบบเสียง (audio) หรือเฟิร์มแวร์คีย์บอร์ด

## ตัวหลัก

- [rust-unofficial/awesome-rust](https://github.com/rust-unofficial/awesome-rust) - ลิสต์แม่บท รวมกว่า 1,131 รายการ ใน 102 หมวดหมู่ แนะนำให้เริ่มจากตัวนี้ก่อนเสมอ
- [TaKO8Ki/awesome-alternatives-in-rust](https://github.com/TaKO8Ki/awesome-alternatives-in-rust) - รวมซอฟต์แวร์ชื่อดังที่ถูกเขียนขึ้นใหม่ด้วย Rust เหมาะสำหรับคนที่อยากหาเครื่องมือทดแทนของเดิม
- [rust-boom/rust-boom](https://github.com/rust-boom/rust-boom) - ลิสต์เนื้อหาภาษาจีน ครอบคลุมทั้งแหล่งเรียนรู้ แหล่งข้อมูล และหนังสือแนะนำ
- [awesome-rust-com/awesome-rust](https://github.com/awesome-rust-com/awesome-rust) - อีกหนึ่งคลังในชื่อ awesome-rust ที่มีการจัดหมวดหมู่แตกต่างจากตัวหลัก
- [unpluggedcoder/awesome-rust-tools](https://github.com/unpluggedcoder/awesome-rust-tools) - รวมเครื่องมือสาย productivity เจ๋ง ๆ ที่พัฒนาด้วย Rust
- [jaywcjlove/awesome-rust-apps](https://github.com/jaywcjlove/awesome-rust-apps) - แหล่งรวมแอปพลิเคชันสำเร็จรูปที่สร้างขึ้นด้วยภาษา Rust
- [Correia-jpv/fucking-awesome-rust](https://github.com/Correia-jpv/fucking-awesome-rust) - awesome-rust ฉบับที่กำกับตัวเลขดาวและ fork ของทุกโปรเจกต์ไว้ข้างทุกรายการ พร้อมอัปเดตอัตโนมัติสม่ำเสมอ
- [amanbolat/awesome-rust-with-stars](https://github.com/amanbolat/awesome-rust-with-stars) - ลิสต์โปรเจกต์ Rust พร้อมจำนวนดาว จัดเรียงให้เห็นโปรเจกต์ยอดนิยมได้ไวขึ้น

ข้อสังเกตคือมี repo ที่ใช้ชื่อซ้ำกันว่า `awesome-rust` อยู่หลายเจ้าจนอาจทำให้สับสนได้ หากต้องการอ้างอิงข้อมูลมาตรฐาน ให้ยึดคลังของ **rust-unofficial** เป็นหลักครับ เนื่องจากเป็นคลังที่มีคนติดตามมากที่สุดและมีรายการอัปเดตมากกว่าหนึ่งพันรายการ

## Embedded และ low-level

- [rust-embedded/awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust) - คลังหลักสาย embedded รวบรวมทั้ง HAL, RTOS และบอร์ดพัฒนาต่าง ๆ ไว้อย่างครบครัน
- [esp-rs/awesome-esp-rust](https://github.com/esp-rs/awesome-esp-rust) - สำหรับชิป ESP32 โดยเฉพาะ ตั้งแต่เริ่มต้นสั่ง blink ไฟ ไปจนถึงงานระดับ production
- [aya-rs/awesome-aya](https://github.com/aya-rs/awesome-aya) - การพัฒนา eBPF ด้วย Rust ผ่านเฟรมเวิร์ก aya
- [AeroRust/awesome-space](https://github.com/AeroRust/awesome-space) - เทคโนโลยีด้านอวกาศและการบิน ตั้งแต่ไดรเวอร์ควบคุมดาวเทียมไปจนถึงระบบ ground station
- [dfrankland/awesome-rust-keyboard-firmware](https://github.com/dfrankland/awesome-rust-keyboard-firmware) - เฟิร์มแวร์คีย์บอร์ดที่พัฒนาขึ้นด้วย Rust ทั้งหมด

## Blockchain และ cryptography

- [rust-in-blockchain/awesome-blockchain-rust](https://github.com/rust-in-blockchain/awesome-blockchain-rust) - คลังหลักของสายบล็อกเชน รวบรวมไลบรารีสำคัญและเครื่องมือด้าน cryptography ไว้ครบครัน
- [polkadot-developers/awesome-substrate](https://github.com/polkadot-developers/awesome-substrate) - ทรัพยากรสำหรับ Polkadot SDK (ชื่อเดิม Substrate) และระบบ parachain
- [rust-cc/awesome-cryptography-rust](https://github.com/rust-cc/awesome-cryptography-rust) - รวบรวมไลบรารี cryptography แทบทุกหมวดหมู่
- [CosmWasm/awesome-cosmwasm](https://github.com/CosmWasm/awesome-cosmwasm) - แหล่งข้อมูลการเขียน smart contract บนแพลตฟอร์ม CosmWasm
- [Vid201/awesome-ethereum-rust](https://github.com/Vid201/awesome-ethereum-rust) - ecosystem ของ Ethereum ในฝั่งภาษา Rust
- [chalex-eth/awesome-ethers-rs](https://github.com/chalex-eth/awesome-ethers-rs) - แหล่งรวม ethers-rs และเครื่องมือที่เกี่ยวข้อง
- [BitcoinDevelopersAcademy/awesome-rust-bitcoin](https://github.com/BitcoinDevelopersAcademy/awesome-rust-bitcoin) - รวมไลบรารีและโปรเจกต์เกี่ยวกับ Bitcoin ในภาษา Rust
- [2nd-Layer/awesome-cardano-rust](https://github.com/2nd-Layer/awesome-cardano-rust) - แหล่งข้อมูลและการพัฒนาบน Cardano ด้วย Rust

## AI, ML และ LLM

- [vaaaaanquish/Awesome-Rust-MachineLearning](https://github.com/vaaaaanquish/Awesome-Rust-MachineLearning) - คลังหลักสาย ML ใน Rust มีทั้งไลบรารี งานวิจัย (paper) และบทความประกอบ
- [e-tornike/best-of-ml-rust](https://github.com/e-tornike/best-of-ml-rust) - จัดอันดับความนิยมและคุณภาพของไลบรารี ML ใน Rust
- [BurtonQin/Awesome-Rust-Neural-Network](https://github.com/BurtonQin/Awesome-Rust-Neural-Network) - รวบรวมโปรเจกต์ที่เกี่ยวข้องกับ neural network ใน Rust
- [malisper/awesome-ai-rust-rewrites](https://github.com/malisper/awesome-ai-rust-rewrites) - รวมเคสโปรเจกต์ที่ใช้ AI เข้ามาช่วย rewrite โค้ดใหม่ให้กลายเป็นภาษา Rust
- [dhilipsiva/awesome-rust-ml](https://github.com/dhilipsiva/awesome-rust-ml) - แหล่งรวม repository ด้าน ML/DL ที่เขียนด้วย Rust
- [ever-works/awesome-rust-ai-libraries](https://github.com/ever-works/awesome-rust-ai-libraries) - ไดเรกทอรีรวบรวม crate สาย AI พร้อมตัวเลขดาวและลิงก์เอกสาร
- [adventurewave-labs/awesome-rust-agentics](https://github.com/adventurewave-labs/awesome-rust-agentics) - รวมไลบรารีและเครื่องมือสำหรับ agentic AI ทั้ง agent framework, inference, MCP และ vector store

## Web, WASM และ cloud

- [tauri-apps/awesome-tauri](https://github.com/tauri-apps/awesome-tauri) - รวมแอป, ปลั๊กอิน และทรัพยากรทุกอย่างเกี่ยวกับ Tauri
- [ratatui/awesome-ratatui](https://github.com/ratatui/awesome-ratatui) - รวมแอปแนว TUI และไลบรารีเสริมต่าง ๆ ที่สร้างขึ้นด้วย Ratatui
- [jetli/awesome-yew](https://github.com/jetli/awesome-yew) - แหล่งรวมเครื่องมือสำหรับเฟรมเวิร์ก Yew และงาน WebAssembly ฝั่ง frontend
- [leptos-rs/awesome-leptos](https://github.com/leptos-rs/awesome-leptos) - ecosystem ของ Leptos (ซึ่งบล็อกนี้ก็พัฒนาขึ้นด้วยเฟรมเวิร์กตัวนี้เช่นกัน)
- [rustwasm/awesome-rust-and-webassembly](https://github.com/rustwasm/awesome-rust-and-webassembly) - ข้อมูล Rust + WebAssembly (แม้ถูก archive แล้ว แต่ยังใช้อ้างอิงทางเทคนิคได้เป็นอย่างดี)
- [iced-rs/awesome-iced](https://github.com/iced-rs/awesome-iced) - รวม widget, โค้ดตัวอย่าง และการนำ iced ไป integrate ใช้งาน
- [karimould/awesome-js-tooling-in-rust](https://github.com/karimould/awesome-js-tooling-in-rust) - รวบรวมเครื่องมือในฝั่ง JavaScript ที่ถูกเขียนขึ้นใหม่ด้วยความเร็วของ Rust
- [DioxusLabs/awesome-dioxus](https://github.com/DioxusLabs/awesome-dioxus) - ทรัพยากรและตัวช่วยพัฒนาทั้งหมดสำหรับเฟรมเวิร์ก Dioxus
- [awesome-rust-cloud-native/awesome-rust-cloud-native](https://github.com/awesome-rust-cloud-native/awesome-rust-cloud-native) - งานสาย Cloud native, Kubernetes และ container ในโลกของ Rust
- [szabgab/awesome-axum](https://github.com/szabgab/awesome-axum) - รวมบทเรียน (tutorial), โปรเจกต์ showcase และ extension เสริมสำหรับ axum
- [edo-zhou/awesome-gpui](https://github.com/edo-zhou/awesome-gpui) - GPUI จากค่าย Zed สำหรับนำมาพัฒนาแอปพลิเคชันบนเดสก์ท็อป
- [vonnieda/awesome-egui](https://github.com/vonnieda/awesome-egui) - รวบรวมโค้ดตัวอย่างและทรัพยากรสำหรับใช้งานร่วมกับ egui
- [nmoutschen/awesome-serverless-rust](https://github.com/nmoutschen/awesome-serverless-rust) - การพัฒนาระบบ serverless และ FaaS ด้วยภาษา Rust

## Game, graphics และ audio

- [bevyengine/bevy-assets](https://github.com/bevyengine/bevy-assets) - คลัง asset, ปลั๊กอิน และตัวอย่างเกมที่คอมมูนิตี้สร้างขึ้นบน Bevy Engine
- [ThierryBerger/bevy_awesome_prod](https://github.com/ThierryBerger/bevy_awesome_prod) - โปรเจกต์ Bevy ที่ถูกนำไปใช้งานจริงในเชิงพาณิชย์และระดับ production
- [nolantait/awesome-bevy](https://github.com/nolantait/awesome-bevy) - แหล่งรวมเนื้อหาการเรียนรู้ Bevy จาก taintedcoders
- [NightsWatchGames/awesome-rust-gamedev](https://github.com/NightsWatchGames/awesome-rust-gamedev) - แหล่งรวมข้อมูลด้านการพัฒนาเกมด้วย Rust ในภาพรวมทุก engine
- [kfrncs/awesome-rust-audio](https://github.com/kfrncs/awesome-rust-audio) - งานสาย audio programming ครอบคลุมทั้ง DSP, synth และการเขียนปลั๊กอินเสียง
- [arlyon/awesome-wow-rust](https://github.com/arlyon/awesome-wow-rust) - การทำ private server เกม World of Warcraft ด้วยภาษา Rust

## Security และ formal verification

- [osirislab/awesome-rust-security](https://github.com/osirislab/awesome-rust-security) - แหล่งรวมโปรเจกต์และทรัพยากรด้านความปลอดภัย (security) ในภาพรวม
- [newca12/awesome-rust-formalized-reasoning](https://github.com/newca12/awesome-rust-formalized-reasoning) - งาน formal verification, proof assistant และคณิตศาสตร์เชิงโครงสร้าง
- [ebalo55/awesome-offensive-rust](https://github.com/ebalo55/awesome-offensive-rust) - งานสาย offensive security และเครื่องมือสำหรับ red teaming ที่เขียนด้วย Rust
- [kevincouton/awesome-rust-migrations](https://github.com/kevincouton/awesome-rust-migrations) - รวบรวมแนวทางและกรณีศึกษาจริงจากการย้ายระบบเดิมมาเป็น Rust
- [iAnonymous3000/awesome-rust-security-guide](https://github.com/iAnonymous3000/awesome-rust-security-guide) - คู่มือเจาะลึกเรื่องความปลอดภัยใน Rust แบบเรียบเรียงให้อ่านง่าย

## Testing และคุณภาพโค้ด

- [hoodie/awesome-rust-testing](https://github.com/hoodie/awesome-rust-testing) - รวมทุกเรื่องน่ารู้เกี่ยวกับการเขียนเทสต์ (testing) ในภาษา Rust
- [BurtonQin/Awesome-Rust-Checker](https://github.com/BurtonQin/Awesome-Rust-Checker) - รวมเครื่องมือจำพวก checker และ verifier สำหรับตรวจเช็กคุณภาพโค้ด
- [alexwennerberg/awesome-small-rust](https://github.com/alexwennerberg/awesome-small-rust) - รวม crate ทางเลือกที่เน้นขนาดกะทัดรัดและใช้ dependency ให้น้อยที่สุด

## เฉพาะทางสุด ๆ (หัวข้อแคบแต่มีเจ้านี้เจ้าเดียว)

- [pka/awesome-georust](https://github.com/pka/awesome-georust) - ระบบสารสนเทศภูมิศาสตร์ (GIS) และงานภูมิสารสนเทศใน Rust
- [jespersm/awesome-rust-openapi-support](https://github.com/jespersm/awesome-rust-openapi-support) - เครื่องมือ generate โค้ดฝั่ง OpenAPI ทั้ง server และ client
- [commons-research/awesome-rust-datascience](https://github.com/commons-research/awesome-rust-datascience) - การนำ Rust ไปประยุกต์ใช้กับงานด้าน Data Science
- [owizdom/awesome-rust-quant](https://github.com/owizdom/awesome-rust-quant) - การเขียนโปรแกรมสำหรับงานสายการเงินเชิงปริมาณ (Quantitative Finance)
- [keller-mark/awesome-rust-vis](https://github.com/keller-mark/awesome-rust-vis) - รวม crate สำหรับการทำ data visualization
- [BabarZKhan/awesome-Rust-compilers](https://github.com/BabarZKhan/awesome-Rust-compilers) - รวมโครงสร้างพื้นฐานคอมไพเลอร์ Rust, IR แบบ SSA และ codegen backend ตั้งแต่ rustc internals ไปจนถึง MLIR/LLVM และ GPU

> คลังในกลุ่มนี้เป็นหัวข้อที่เฉพาะทางสูงมาก จึงอาจมีคนรู้จักไม่มากนัก แต่นั่นไม่ได้แปลว่าเนื้อหาข้างในจะด้อยคุณภาพ แนะนำให้ลองกดเข้าไปสำรวจดูเนื้อหาก่อนตัดสินใจครับ

## บทเรียนและชุมชน

- [ctjhoa/rust-learning](https://github.com/ctjhoa/rust-learning) - แหล่งรวมลิงก์บล็อก บทความ และวิดีโอคุณภาพสูงสำหรับฝึกฝนการเขียน Rust
- [sger/RustBooks](https://github.com/sger/RustBooks) - สารบัญรวบรวมหนังสือเกี่ยวกับภาษา Rust แทบทุกเล่มที่มีอยู่ในตลาด
- [RustBeginners/awesome-rust-mentors](https://github.com/RustBeginners/awesome-rust-mentors) - รวมรายชื่อพี่เลี้ยง (mentor) พร้อมระบุหมวดหมู่ที่แต่ละคนมีความถนัด
- [jamesmunns/awesome-rust-streaming](https://github.com/jamesmunns/awesome-rust-streaming) - รายชื่อสตรีมเมอร์สาย Rust ที่ยังมีผลงานออกมาสม่ำเสมอ
- [Robert-Steiner/awesome-rust-blog-posts](https://github.com/Robert-Steiner/awesome-rust-blog-posts) - รวมบล็อกโพสต์ที่น่าสนใจและสร้างแรงกระเพื่อมในวงการ
- [nikitaignatov/awesome-rust-talks](https://github.com/nikitaignatov/awesome-rust-talks) - แหล่งรวม talk และวิดีโอบรรยายจากงานคอนเฟอเรนซ์ต่าง ๆ
- [Evian-Zhang/awesome-rust-papers](https://github.com/Evian-Zhang/awesome-rust-papers) - รวบรวมงานวิจัยเชิงวิชาการเกี่ยวกับภาษา Rust แบ่งกลุ่มตามหัวข้อ พร้อมเว็บเวอร์ชันอ่านง่ายและ RSS
- [h1trust/awesome-hit-rust](https://github.com/h1trust/awesome-hit-rust) - ลิสต์จากคอมมูนิตี้มหาวิทยาลัย HIT (Harbin Institute of Technology) ทั้งโปรเจกต์ของนักศึกษาและแหล่งเรียนรู้

## ไม่ใช่ awesome list แต่ควรรู้จัก

แม้ 3 แหล่งข้อมูลนี้จะไม่เข้าข่าย awesome list แบบคลังอื่น ๆ แต่ก็จัดว่าเป็นแหล่งความรู้ชั้นยอดที่คนเขียน Rust ทุกคนควรเปิดอ่านและเซฟเก็บไว้

- [rust-unofficial/patterns](https://github.com/rust-unofficial/patterns) - รวบรวม Rust Design Patterns, anti-patterns และสำนวนการเขียนโค้ด (idioms) ที่ถูกต้อง
- [rust-unofficial/too-many-lists](https://github.com/rust-unofficial/too-many-lists) - เรียนรู้เรื่อง ownership ให้แตกฉานผ่านการลงมือเขียน linked list หลายรูปแบบ (มีฉบับแปลไทย)
- [nnethercote/perf-book](https://github.com/nnethercote/perf-book) - The Rust Performance Book คู่มือสอนวิธีจูนและ optimize โค้ดอย่างมีหลักการและเป็นระบบ

## วิธีหา awesome list เพิ่มเติมด้วยตัวเอง

หากต้องการสำรวจเพิ่มเติม หรืออยากค้นหาลิสต์ในหัวข้ออื่น ๆ นอกเหนือจากที่ระบุไว้ข้างต้น ลองนำเทคนิคเหล่านี้ไปใช้ดูครับ

- **ค้นหาบน GitHub ผ่าน Topic**: ลองพิมพ์ `topic:awesome-list topic:rust` ในช่องค้นหา เพื่อดึงรายชื่อคลังที่มีการติดแท็กเหล่านี้ไว้ออกมา
- **ค้นหาตามรูปแบบชื่อยอดนิยม**: ลองค้นหาด้วย pattern อย่าง `awesome-<framework>` (เช่น awesome-tauri, awesome-bevy) หรือ `awesome-<domain>-rust` (เช่น awesome-embedded-rust)
- **ตามรอยจากคลังแม่**: เปิดไปที่ [sindresorhus/awesome](https://github.com/sindresorhus/awesome) แล้วลองไล่ดูรายการในส่วน Programming Languages
- **ตรวจเช็กความพร้อมก่อนนำไปอ้างอิง**: ดูคอมมิตล่าสุดว่าไม่ควรถูกทิ้งร้างเกิน 1 ปี, ตัว repo ต้องยังไม่ถูก archive, ลิงก์ภายในยังกดเข้าได้ตามปกติ และมีจำนวนดาวในระดับที่น่าเชื่อถือ

จะเห็นได้ว่า ecosystem ของ awesome list ในโลก Rust นั้นกว้างใหญ่กว่าที่หลายคนคิด จากคลังหลักที่เป็นที่รู้จักแพร่หลาย ได้แตกแขนงออกเป็นลิสต์เฉพาะทางอีกกว่า 70 แห่ง ครอบคลุมตั้งแต่สาย embedded, blockchain, AI, เกม, security ไปจนถึงงานเสียงและเฟิร์มแวร์คีย์บอร์ด

หากกำลังลุยงานด้านไหนอยู่ ลองแวะเข้าไปสำรวจคลังในสายนั้น ๆ ดูครับ นอกจากจะได้เจอไลบรารีเจ๋ง ๆ ที่ไม่เคยรู้จักมาก่อนแล้ว ยังได้เห็นโปรเจกต์ของคนอื่นที่กำลังแก้โจทย์คล้าย ๆ กัน ซึ่งเป็นอีกหนึ่งวิธีที่ดีมากในการหาแรงบันดาลใจ ดีกว่าการนั่งไถดู GitHub Trending ทั่วไปอย่างแน่นอนครับ
