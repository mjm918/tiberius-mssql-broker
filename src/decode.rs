use chrono::{FixedOffset, NaiveDateTime, Utc};
use num_traits::ToPrimitive;
use tiberius::numeric::BigDecimal;
use tiberius::ColumnData;
use sea_query::Value;

use crate::error::Error;

pub trait Decode {
	fn decode(row: &ColumnData<'static>) -> Result<Value, Error>;
}

impl Decode for Value {
	fn decode(row: &ColumnData<'static>) -> Result<Value, Error> {
		Ok(match row {
			ColumnData::U8(v) => Value::TinyUnsigned(v.clone()),
			ColumnData::I16(v) => Value::SmallInt(v.clone()),
			ColumnData::I32(v) => Value::Int(v.clone()),
			ColumnData::I64(v) => Value::BigInt(v.clone()),
			ColumnData::F32(v) => Value::Float(v.clone()),
			ColumnData::F64(v) => Value::Double(v.clone()),
			ColumnData::Bit(v) => Value::Bool(v.clone()),
			ColumnData::String(v) => match v {
				None => Value::String(None),
				Some(v) => Value::String(Some(Box::new(v.to_string()))),
			},
			ColumnData::Guid(v) => match v {
				None => Value::Uuid(None),
				Some(v) => Value::Uuid(Some(Box::new(v.clone()))),
			},
			ColumnData::Binary(v) => match v {
				None => Value::Bytes(None),
				Some(v) => Value::Bytes(Some(Box::new(v.to_vec()))),
			},
			ColumnData::Numeric(v) => match v {
				None => Value::Double(None),
				Some(_) => {
					let v: tiberius::Result<Option<BigDecimal>> = tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::Double(None),
							Some(v) => Value::Double(v.to_f64()),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::Xml(v) => match v {
				None => Value::String(None),
				Some(v) => Value::String(Some(Box::new(v.to_string()))),
			},
			ColumnData::DateTime(v) => match v {
				None => Value::ChronoDateTime(None),
				Some(_) => {
					let v: tiberius::Result<Option<NaiveDateTime>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoDateTime(None),
							Some(v) => Value::ChronoDateTime(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::SmallDateTime(m) => match m {
				None => Value::ChronoDateTime(None),
				Some(_) => {
					let v: tiberius::Result<Option<chrono::NaiveDateTime>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoDateTime(None),
							Some(v) => Value::ChronoDateTime(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::Time(v) => match v {
				None => Value::ChronoTime(None),
				Some(_) => {
					let v: tiberius::Result<Option<chrono::NaiveTime>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoTime(None),
							Some(v) => Value::ChronoTime(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::Date(v) => match v {
				None => Value::ChronoDate(None),
				Some(_) => {
					let v: tiberius::Result<Option<chrono::NaiveDate>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoDate(None),
							Some(v) => Value::ChronoDate(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::DateTime2(v) => match v {
				None => Value::ChronoDateTimeUtc(None),
				Some(_) => {
					let v: tiberius::Result<Option<chrono::DateTime<Utc>>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoDateTimeUtc(None),
							Some(v) => Value::ChronoDateTimeUtc(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
			ColumnData::DateTimeOffset(v) => match v {
				None => Value::ChronoDateTimeWithTimeZone(None),
				Some(_) => {
					let v: tiberius::Result<Option<chrono::DateTime<FixedOffset>>> =
						tiberius::FromSql::from_sql(row);
					match v {
						Ok(v) => match v {
							None => Value::ChronoDateTimeWithTimeZone(None),
							Some(v) => Value::ChronoDateTimeWithTimeZone(Some(Box::new(v))),
						},
						Err(e) => {
							return Err(Error::from(e.to_string()));
						}
					}
				}
			},
		})
	}
}