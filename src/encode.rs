use tiberius::{Query, Uuid};
use crate::error::Error;
use sea_query::Value;

pub trait Encode {
	fn encode(self, q: &mut Query) -> Result<(), Error>;
}

impl Encode for Value {
	fn encode(self, q: &mut Query) -> Result<(), Error> {
		match self {
			Value::Bool(v) => {
				q.bind(v);
				Ok(())
			}
			Value::Int(v) => {
				q.bind(v);
				Ok(())
			}
			Value::BigInt(v) => {
				q.bind(v);
				Ok(())
			}
			Value::Unsigned(v) => {
				match v {
					None => {
						q.bind(0i32);
					}
					Some(uu32) => {
						q.bind(uu32 as i32);
					}
				}
				Ok(())
			}
			Value::BigUnsigned(v) => {
				match v {
					None => {
						q.bind(Some(0i64));
					}
					Some(uu32) => {
						q.bind(Some(uu32 as i64));
					}
				}
				Ok(())
			}
			Value::Float(v) => {
				q.bind(v);
				Ok(())
			}
			Value::Double(v) => {
				q.bind(v);
				Ok(())
			}
			Value::String(v) => {
				match v {
					None => {
						q.bind(Some(""));
					}
					Some(ss) => {
						q.bind(Some(ss.to_string()));
					}
				}
				Ok(())
			}
			Value::Bytes(v) => {
				match v {
					None => {
						q.bind(Some(vec![]));
					}
					Some(ss) => {
						q.bind(Some(ss.to_vec()));
					}
				}
				Ok(())
			}
			Value::ChronoDate(v) => {
				match v {
					None => {
						q.bind(Some(chrono::NaiveDate::MIN));
					}
					Some(ss) => {
						q.bind(Some(ss.as_ref().to_owned()));
					}
				}
				Ok(())
			}
			Value::ChronoDateTime(v) => {
				match v {
					None => {
						q.bind(Some(chrono::NaiveDateTime::MIN));
					}
					Some(ss) => {
						q.bind(Some(ss.as_ref().to_owned()));
					}
				}
				Ok(())
			}
			Value::ChronoTime(v) => {
				match v {
					None => {
						q.bind(Some(chrono::NaiveTime::MIN));
					}
					Some(ss) => {
						q.bind(Some(ss.as_ref().to_owned()));
					}
				}
				Ok(())
			}
			Value::ChronoDateTimeUtc(v)=> {
				match v {
					None => {
						q.bind(Some(chrono::DateTime::<chrono::FixedOffset>::MIN_UTC));
					}
					Some(ss) => {
						q.bind(Some(ss.as_ref().to_owned()));
					}
				}
				Ok(())
			},
			Value::Uuid(v) => {
				match v {
					None => {
						q.bind(Some(Uuid::NAMESPACE_OID));
					}
					Some(ss) => {
						q.bind(Some(ss.as_ref().to_owned()));
					}
				}
				Ok(())
			},
			_ => panic!("type not supported")
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	#[test]
	fn test_from() {
		let mut v = fastdate::DateTime::now().to_string();
		v.replace_range(10..11, "T");
		println!("{}", v.to_string());
		let n = chrono::NaiveDateTime::from_str(&v).unwrap();
		assert_eq!(n.to_string().replace(" ", "T").trim_end_matches("00"), v);
	}
}