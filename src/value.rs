use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as Json;
use tiberius::numeric::BigDecimal;
use tiberius::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ArrayType {
    Bool,
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    TinyUnsigned,
    SmallUnsigned,
    Unsigned,
    BigUnsigned,
    Float,
    Double,
    String,
    Char,
    Bytes,
    Json,
    ChronoDate,
    ChronoTime,
    ChronoDateTime,
    ChronoDateTimeUtc,
    ChronoDateTimeLocal,
    ChronoDateTimeWithTimeZone,
    Uuid,
    BigDecimal,
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(Option<bool>),
    TinyInt(Option<i8>),
    SmallInt(Option<i16>),
    Int(Option<i32>),
    BigInt(Option<i64>),
    TinyUnsigned(Option<u8>),
    SmallUnsigned(Option<u16>),
    Unsigned(Option<u32>),
    BigUnsigned(Option<u64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<Box<String>>),
    Char(Option<char>),
    #[allow(clippy::box_collection)]
    Bytes(Option<Box<Vec<u8>>>),
    Json(Option<Box<Json>>),
    ChronoDate(Option<Box<NaiveDate>>),
    ChronoTime(Option<Box<NaiveTime>>),
    ChronoDateTime(Option<Box<NaiveDateTime>>),
    ChronoDateTimeUtc(Option<Box<DateTime<Utc>>>),
    ChronoDateTimeLocal(Option<Box<DateTime<Local>>>),
    ChronoDateTimeWithTimeZone(Option<Box<DateTime<FixedOffset>>>),
    Uuid(Option<Box<Uuid>>),
    BigDecimal(Option<Box<BigDecimal>>),
    Array(ArrayType, Option<Box<Vec<Value>>>),
    Null,
    Error(String),
}

