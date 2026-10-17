---
title: "เมื่อ unwrap_or_default() กลืน error เงียบ ๆ"
date: "2026-10-12"
description: "เคสจริงจาก TB Plus เมื่อตารางประวัติจ่ายยาเปิดมาแล้วว่างเปล่า ไม่มี error หรือ log แจ้งเตือน ไล่หาสาเหตุตั้งแต่การใช้ unwrap_or_default() จนถึงข้อจำกัดของ sqlx และการส่งข้อมูล DECIMAL ของ MySQL"
tags: [rust, tauri]
author: "suradet-ps"
---

ในคลินิกวัณโรค โรงพยาบาลสระโบสถ์ พยาบาลต้องตรวจดูประวัติการรับยาของผู้ป่วยอยู่ตลอดเพื่อดูว่าใครกินยาต่อเนื่องหรือใครมีความเสี่ยงที่จะขาดยา ระบบที่ช่วยดึงข้อมูลส่วนนี้คือ TB Plus ([github.com/suradet-ps/tb-plus](https://github.com/suradet-ps/tb-plus)) ซึ่งเป็น desktop app ที่เชื่อมต่อแบบ read-only ไปยังฐานข้อมูล HOSxP

วันหนึ่งมีเคสที่หน้าจอ **ประวัติการจ่ายยา** เปิดขึ้นมาแล้วว่างเปล่า ไม่มีข้อมูล ไม่มีแจ้งเตือน และไม่มี log error ใด ๆ ขึ้นเลย ดูเผิน ๆ เหมือนคนไข้รายนี้ไม่เคยได้ยามาก่อน ทั้งที่พอไปเช็คในตาราง `opitemrece` ของ HOSxP โดยตรง ข้อมูลก็อยู่ครบทุกแถว

นี่คือเคส silent error ที่อันตรายมากสำหรับระบบงานคลินิก เพราะโปรแกรมไม่ได้ crash แต่กลับแสดงผลข้อมูลผิดพลาดโดยที่เราไม่รู้ตัว และพอย้อนรอยกลับไปดู ก็พบว่าต้นตอมาจาก method พื้นฐานอย่าง `unwrap_or_default()` ใน Rust ร่วมกับพฤติกรรมบางอย่างของ MySQL driver

## การทำงานของ `unwrap_or_default()`

ใน Rust เราจัดการกับความไม่แน่นอนผ่าน enum สองตัวคือ `Option<T>` กับ `Result<T, E>` ([The Rust Book บทที่ 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html))

```rust
pub enum Option<T> {
    None,
    Some(T),
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

- `Option<T>` ใช้บอกว่า **มีค่าหรือไม่** เช่น ค้นหาข้อมูลแล้วไม่เจอแถวที่ต้องการ (`None`)
- `Result<T, E>` ใช้บอกว่า **สำเร็จหรือเกิด error** และถ้าพัง `Err(E)` จะเก็บสาเหตุไว้ด้วยเสมอ เช่น ต่อ network ไม่ติด หรือ decode ข้อมูลผิด type

ปัญหาเกิดขึ้นเมื่อเราจัดการกับ `Result` ด้วย `unwrap_or_default()` ลองดู signature จาก standard library

```rust
impl<T, E> Result<T, E> {
    pub fn unwrap_or_default(self) -> T
    where
        T: Default;
}

impl<T> Option<T> {
    pub fn unwrap_or_default(self) -> T
    where
        T: Default;
}
```

จุดที่ต้องระวังคือ trait bound อยู่ที่ `T: Default` ฝั่งเดียว โดยไม่สนใจประเภทของ `E` เลย แปลว่าถ้าเป็น `Err` มันจะทิ้ง error payload ทิ้งทันที แล้วคืนค่าเริ่มต้นของ `T` กลับมาแทน โดยไม่มีการ log หรือเตือนอะไรทั้งสิ้น

ตัวอย่างการทำงานทั่วไป

```rust
let good_year_from_input = "1909";
let bad_year_from_input = "190blarg";
let good_year = good_year_from_input.parse().unwrap_or_default();
let bad_year = bad_year_from_input.parse().unwrap_or_default();

assert_eq!(1909, good_year);
assert_eq!(0, bad_year);
```

ถ้าเป็น input ฟอร์มทั่วไป การที่ parse ไม่ผ่านแล้วได้ `0` อาจจะพอรับได้ในบาง use case แต่ถ้า `0` นั้นนำไปใช้เป็น business data (เช่น ยอดจ่ายยา = 0) ความหมายของข้อมูลจะเพี้ยนทันที

## ปัญหาเมื่อใช้คู่กับ `Vec`

`Vec<T>` implement `Default` ให้ได้ผลลัพธ์เป็น `Vec::new()` (vector ว่าง) ทำให้ `Result<Vec<T>, E>` สามารถเรียก `unwrap_or_default()` ได้เสมอ

```rust
let failed: Result<Vec<String>, QueryError> = Err(QueryError("boom"));
let rows = failed.unwrap_or_default();
assert!(rows.is_empty()); // กลายเป็น vector ว่างไปเฉย ๆ โดยที่ error หายไปแล้ว
```

ลองดูผลเปรียบเทียบระหว่างการใช้ `unwrap_or_default()` กับการใช้ `?` operator

```text
1) rows = [], len = 0
2) missing = []
3) short_circuit = Err(ParseIntError { kind: InvalidDigit })
4) bad_year = 0
5) propagate = Err(QueryError("expected a DOUBLE as 4 or 8 bytes, got 5 bytes"))
```

จะเห็นว่าแทนที่เราจะได้ error message ชัดเจนแบบข้อ 5 สิ่งที่ได้กลับเป็น `[]` ในข้อ 1 ทำให้ downstream logic คิดว่า query ผ่านเรียบร้อยแต่ไม่มีข้อมูล

## จุดเกิดเหตุใน TB Plus

ในโค้ดส่วนดึงข้อมูลยา (`crates/tb-database/src/mysql.rs`) เดิมมีคอมเมนต์บันทึกปัญหานี้ไว้

```rust
/// - `CAST(o.qty AS DOUBLE)` avoids sqlx DECIMAL→f64 mapping issues that
///   silently kill the entire result set with unwrap_or_default().
```

ลำดับเหตุการณ์ที่เกิดขึ้นคือ

1. ตาราง `opitemrece` ของ HOSxP เก็บจำนวนยาในคอลัมน์ `qty` เป็นชนิดข้อมูล `DECIMAL`
2. Struct ฝั่ง Rust ประกาศรับคอลัมน์นี้เป็น `Option<f64>`

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct DispensingRecord {
    pub vstdate: String,
    pub icode: String,
    pub drug_name: Option<String>,
    pub qty: Option<f64>,
    // ...
}
```

3. ตอนดึงข้อมูล มีการเขียนโค้ดย่อไว้แบบนี้

```rust
let rows: Vec<DispensingRecord> = q.fetch_all(pool).await.unwrap_or_default();
```

4. `fetch_all()` มีพฤติกรรมคือ ถ้ามีแถวใดแถวหนึ่ง decode ไม่ผ่าน มันจะคืนค่าเป็น `Err` ทั้งชุดทันที (ข้างในใช้ `try_collect` ของ `sqlx-core` เจอ error ตัวแรกก็ตัดจบ)
5. พอต่อท้ายด้วย `unwrap_or_default()` แทนที่จะได้ `Err` แจ้งว่า decode พัง มันกลับแปลงเป็น `Vec::new()` ทันที

ผลลัพธ์คือ UI แสดงว่าคนไข้ไม่มีประวัติรับยาเลย ทั้งที่ปัญหาจริง ๆ คือ decode ข้อมูลล้มเหลว และเนื่องจากโปรเจกต์เปิด lint `unwrap_used = "deny"` ไว้ ตัว lint ดักจับแค่ `.unwrap()` แต่ไม่ได้ตรวจ `unwrap_or_default()` โค้ดจึงผ่านการ compile และ code review มาได้แบบปกติ

## ทำไม sqlx ไม่ยอมแปลง `DECIMAL` เป็น `f64`

พอไปดู implementation ของ `sqlx` ใน `sqlx-mysql/src/types/float.rs` (เช็คเทียบทั้ง 0.8.6 และ 0.9.0) จะเห็นว่า sqlx จงใจปฏิเสธการแปลง `DECIMAL` ไปเป็น float

```rust
fn real_compatible(ty: &MySqlTypeInfo) -> bool {
    // NOTE: `DECIMAL` is explicitly excluded because floating-point numbers have different semantics.
    matches!(ty.r#type, ColumnType::Float | ColumnType::Double)
}
```

และในฟังก์ชัน decode ของ `f64`

```rust
impl Decode<'_, MySql> for f64 {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => {
                let buf = value.as_bytes()?;
                match buf.len() {
                    4 => LittleEndian::read_f32(buf) as f64,
                    8 => LittleEndian::read_f64(buf),
                    other => {
                        // Users may try to decode a DECIMAL as floating point;
                        // inform them why that's a bad idea.
                        return Err(format!(
                            "expected a DOUBLE as 4 or 8 bytes, got {other} bytes; \
                             note that decoding DECIMAL as `f64` is not supported \
                             due to differing semantics"
                        )
                        .into());
                    }
                }
            }
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
```

เหตุผลทางเทคนิคคือ `DECIMAL` เป็น fixed-point (ฐานสิบ) ที่เก็บค่าแม่นยำแน่นอน ส่วน `f64` เป็น IEEE-754 floating-point (ฐานสอง) ซึ่งมีปัญหาเรื่อง rounding error ทาง sqlx จึงไม่อยากแปลงให้แบบ implicit และแนะนำให้ใช้ crate เฉพาะทางอย่าง `bigdecimal` หรือ `rust_decimal` แทน

แต่จุดที่น่าสนใจคือ ทำไมใน block `MySqlValueFormat::Binary` ถึงเอา buffer length มาเช็ค 4 หรือ 8 ไบต์?

## MySQL ส่ง `DECIMAL` มาเป็น Text แม้จะใช้ Binary Protocol

ปกติเวลาเราใช้ prepared statement ใน MySQL เราคาดหวังว่าข้อมูลจะส่งมาแบบ binary data แต่สำหรับ `DECIMAL` ใน source code ของ MySQL 8.0 (`sql/protocol_classic.h`) ระบุไว้ว่า

```cpp
class Protocol_binary final : public Protocol_text {
  // ...
  // Decimals are sent as text, also over the binary protocol.
  using Protocol_text::store_decimal;
  // ...
};
```

และใน `sql/protocol_classic.cc`

```cpp
bool Protocol_text::store_decimal(const my_decimal *d, uint prec, uint dec) {
  // ...
  int string_length = DECIMAL_MAX_STR_LENGTH + 1;
  int error [[maybe_unused]] =
      decimal2string(d, pos + 1, &string_length, prec, dec);
  // ...
  *pos = string_length;
  packet->length((pos + 1 + string_length) - packet->ptr());
  return false;
}
```

นั่นหมายความว่า MySQL ส่งข้อมูล `DECIMAL` ออกมาเป็น **ข้อความ ASCII เสมอ** เช่น `"12.50"` หรือ `"100"`

เมื่อ `sqlx` ได้รับข้อความนี้ผ่าน binary protocol มันจึงเอาความยาวของ string มาเข้า switch-case
- ถ้า string ยาว 4 ตัวอักษร -> เข้าเคส `read_f32`
- ถ้า string ยาว 8 ตัวอักษร -> เข้าเคส `read_f64`
- ถ้ายาวเป็นตัวเลขอื่น -> คืน `Err`

ในกรณีของ TB Plus ค่าที่ส่งมาคือ `"12.50"` ซึ่งมีความยาว 5 ไบต์ มันจึงตกไปที่เคส `other` และ return error

```text
expected a DOUBLE as 4 or 8 bytes, got 5 bytes;
note that decoding DECIMAL as `f64` is not supported due to differing semantics
```

แต่เพราะเราใส่ `unwrap_or_default()` ไว้ error ข้อความนี้จึงหายไป และกลายเป็น `[]` แทน

## กรณีที่อันตรายกว่า: ข้อมูลบังเอิญยาว 4 หรือ 8 ไบต์

จากโค้ด decode ด้านบน ถ้าข้อมูล string บังเอิญยาว 4 หรือ 8 ไบต์พอดี เช่น `"12.5"` (4 ไบต์) หรือ `"12345678"` (8 ไบต์) `sqlx` จะนำรหัส ASCII ของตัวอักษรเหล่านั้นไป parse ตรง ๆ ผ่าน `read_f32` หรือ `read_f64`

```text
"12.5"     (ASCII: 0x31, 0x32, 0x2E, 0x35) -> แปลงเป็น f32/f64 ได้ 6.489308930213156e-7
"12345678" (8 bytes)                       -> แปลงเป็น f64 ได้ 6.821320051701325e-38
```

ผลลัพธ์คือ decode สำเร็จ ไม่เกิด error แต่ตัวเลขที่ได้กลายเป็นค่าขยะแทน ซึ่งการที่ความยาวจะเป็นกี่ไบต์นั้น ขึ้นอยู่กับ precision และ scale ที่ตารางใน MySQL ประกาศไว้ด้วย

## การแก้ไขปัญหา

แนวทางแก้ปัญหาใน TB Plus มี 2 ส่วน

### 1. ทำ `CAST` ตั้งแต่ query บน MySQL
แปลง type ให้เป็น `DOUBLE` ตั้งแต่ฝั่ง database เพื่อให้ส่งข้อมูลมาเป็น IEEE-754 binary 8 ไบต์จริง ๆ ซึ่งจะ map เข้ากับ `f64` ของ Rust ได้อย่างถูกต้อง

```rust
pub async fn get_dispensing_history(
    pool: &MySqlPool,
    hn: &str,
    all_icodes: &[String],
    icode_to_class_map: &HashMap<String, String>,
) -> Result<Vec<DispensingRecord>> {
    let placeholders = in_placeholders(all_icodes.len());
    let sql = format!(
        "SELECT \
            DATE_FORMAT(o.vstdate, '%Y-%m-%d') AS vstdate, \
            o.icode, \
            TRIM(CONCAT_WS(' ', COALESCE(d.name, o.icode), d.strength)) AS drug_name, \
            CAST(o.qty AS DOUBLE) AS qty, \
            d.units, \
            NULL AS drug_class \
        FROM opitemrece o \
        LEFT JOIN drugitems d ON o.icode = d.icode \
        WHERE o.hn = ? \
          AND o.icode IN ({placeholders}) \
        ORDER BY o.vstdate DESC, o.icode"
    );
    // ...
```

*(หมายเหตุ: ในบริบทนี้ `qty` คือจำนวนเม็ด/ขวดยาที่จ่าย ซึ่งรับได้กับ precision ของ `f64` แต่ถ้าระบบไหนเป็นยอดเงินหรือต้องการความแม่นยำสูง ควรใช้ type `rust_decimal::Decimal` แทน)*

### 2. เลิกใช้ `unwrap_or_default()` แล้ว propagate error ด้วย `?`

```rust
    let mut rows: Vec<DispensingRecord> = tokio::time::timeout(
        std::time::Duration::from_secs(QUERY_TIMEOUT_SECS),
        q.fetch_all(pool),
    )
    .await
    .map_err(|_| anyhow::anyhow!("MySQL query timed out after {QUERY_TIMEOUT_SECS}s"))?
    .map_err(anyhow::Error::from)?;
```

ถ้ามีปัญหาเรื่อง network หรือ type mismatch ตัว error จะถูกส่งกลับขึ้นไปให้ชั้นบนจัดการและแสดงแจ้งเตือนบนหน้า UI ทันที

## แนวทางการใช้งาน `unwrap_or_default()` ที่เหมาะสม

`unwrap_or_default()` ยังคงมีประโยชน์ แต่ควรใช้เฉพาะในจุดที่การ fallback กลับไปเป็น default value ไม่ส่งผลกระทบต่อ logic หลักของโปรแกรม เช่น

**เคสที่ใช้ได้:**
- ค่าคอนฟิกเสริม หรือ UI decoration (เช่น splash message, ธีม):
```rust
let splash = settings.get_splash_messages().await.unwrap_or_default();
```
- ค่า config ที่ถ้าไม่มีในระบบ ให้ใช้ค่า default ได้:
```rust
pub async fn get_json_or_default<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T> {
    Ok(self.get_json::<T>(key).await?.unwrap_or_default())
}
```

**เคสที่ไม่ควรใช้**
- ข้อมูลทางคลินิก ยอดเงิน หรือข้อมูล transaction ที่ความว่างเปล่า (`[]` หรือ `0`) มีความหมายทางธุรกิจ
- การเรียก database query หรือ network request ทั่วไป ถ้าล้มเหลวควร bubble error ขึ้นไป หรืออย่างน้อยต้อง log เก็บไว้

```rust
// ถ้าจำเป็นต้อง fallback จริง ๆ ควร log ไว้เสมอ
let rows = fetch_data().await.unwrap_or_else(|err| {
    tracing::warn!("Failed to fetch data: {err}");
    Vec::new()
});
```

และถ้าทำงานกับ iterator ของ `Result` แล้วต้องการ collect ให้หยุดทันทีที่เจอ error ควรใช้ท่านี้แทน

```rust
let parsed: Result<Vec<i32>, _> =
    ["1", "2", "x"].iter().map(|s| s.parse::<i32>()).collect();

assert!(parsed.is_err()); // หยุดและคืน Err ทันที ไม่ได้กลายเป็น vector เปล่า
```

## สรุป

1. **`unwrap_or_default()` บน `Result` คือการ drop error ทิ้งเงียบ ๆ** เนื่องจาก trait bound บังคับแค่ `T: Default` ทำให้ compiler ไม่สนใจว่า `Err` ข้างในคืออะไร
2. **Clippy `unwrap_used = "deny"` ดักไม่เจอ** เพราะตรวจแค่ `.unwrap()` และ `.expect()` เท่านั้น
3. **`DECIMAL` กับ `f64` ไม่ควร map กันตรง ๆ ใน sqlx** MySQL ส่ง `DECIMAL` เป็น text เสมอ ซึ่งถ้าฝั่ง Rust พยายาม decode เข้า `f64` จะเสี่ยงทั้ง decode fail หรือได้ค่าตัวเลขเพี้ยนจากการอ่าน byte ASCII
4. **ถ้าข้อมูลสำคัญ ให้ propagate error ด้วย `?` เสมอ** การปล่อยให้โปรแกรมขึ้น error dialog หรือแจ้งเตือนว่าโหลดไม่สำเร็จ ยังปลอดภัยกว่าการที่ระบบแสดงผลว่าไม่มีข้อมูล ทั้งที่จริง ๆ query พังอยู่ข้างหลัง

---

**Credit & Reference:**
- [ซอร์สโค้ด TB Plus](https://github.com/suradet-ps/tb-plus)
- [`Result::unwrap_or_default` (std docs)](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap_or_default)
- [The Rust Book: Chapter 9 Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [sqlx-mysql 0.9.0: src/types/float.rs](https://github.com/launchbadge/sqlx/blob/main/sqlx-mysql/src/types/float.rs)
- [MySQL 8.0 Source Code: `sql/protocol_classic.h`](https://github.com/mysql/mysql-server/blob/8.0/sql/protocol_classic.h) และ [`sql/protocol_classic.cc`](https://github.com/mysql/mysql-server/blob/8.0/sql/protocol_classic.cc)
