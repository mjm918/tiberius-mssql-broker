use std::any::type_name;

use chrono::{DateTime, FixedOffset, Offset, TimeZone, Utc};
use tiberius::numeric::BigDecimal;

use crate::value::Value;

pub fn convert_from_str_to_rusttype(value: &str, mssql_type: &str) -> Value {
    match mssql_type {
        "bigint" => match value.parse::<i64>() {
            Err(_) => Value::BigInt(None),
            Ok(v) => Value::BigInt(Some(v))
        },
        "int" => match value.parse::<i32>() {
            Err(_) => Value::Int(None),
            Ok(v) => Value::Int(Some(v))
        },
        "bit" | "tinyint" => match value.parse::<i8>() {
            Err(_) => Value::TinyInt(None),
            Ok(v) => Value::TinyInt(Some(v))
        },
        "smallint" => match value.parse::<i16>() {
            Err(_) => Value::SmallInt(None),
            Ok(v) => Value::SmallInt(Some(v))
        },
        "numeric" | "money" | "smallmoney" => match value.parse::<f64>() {
            Err(_) => Value::BigDecimal(None),
            Ok(v) => {
                let bd = BigDecimal::try_from(v);
                match bd {
                    Ok(dec) => Value::BigDecimal(Some(Box::new(dec))),
                    Err(_) => Value::BigDecimal(None),
                }
            }
        },
        "decimal" | "real" => match value.parse::<f64>() {
            Err(_) => Value::Double(None),
            Ok(v) => Value::Double(Some(v))
        },
        "float" => match value.parse::<f32>() {
            Err(_) => Value::Float(None),
            Ok(v) => Value::Float(Some(v))
        },
        "date" => match parse_date(value) {
            Some(dt) => Value::ChronoDate(Some(Box::new(dt.date_naive()))),
            None => Value::Error(value.to_string())
        },
        "datetimeoffset" | "datetime2" | "datetime" => match parse_datetime(value) {
            Some(dt) => Value::ChronoDateTimeWithTimeZone(Some(Box::new(dt))),
            None => Value::Error(value.to_string())
        },
        "text" | "varchar" | "nvarchar" | "ntext" | "varbinary" | "binary" | "image" => Value::String(Some(Box::new(value.to_owned()))),
        _ => Value::String(Some(Box::new(value.to_owned())))
    }
}

pub fn parse_datetime(value: &str) -> Option<DateTime<FixedOffset>> {
    match anydate::parse(value) {
        Ok(dt) => Some(dt),
        Err(_) => {
            let dt = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.3f"));
            match dt {
                Ok(datetime) => Some(Utc.fix().from_utc_datetime(&datetime)),
                Err(_) => None
            }
        }
    }
}

pub fn parse_date(value: &str) -> Option<DateTime<FixedOffset>> {
    match anydate::parse(value) {
        Ok(dt) => Some(dt),
        Err(_) => match chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            Ok(date) => Some(Utc.fix().from_utc_datetime(&date.and_time(chrono::NaiveTime::MIN))),
            Err(_) => None
        }
    }
}

pub fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

