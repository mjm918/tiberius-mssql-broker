pub mod connection;
pub mod config;
pub mod deadpool;
pub mod decode;
pub mod encode;
pub mod error;

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
pub use sea_query;

use futures_core::Stream;
use tiberius::{Client, Query};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;
use rayon::prelude::*;

use crate::config::SqlConfig;
use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::Error;
use sea_query::Value;
use crate::connection::LongPooling;

#[derive(Debug)]
pub struct ExecResult {
	pub rows_affected: u64,
	pub last_insert_id: Value,
}

impl Display for ExecResult {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_map()
			.key(&"rows_affected")
			.value(&self.rows_affected)
			.key(&"last_insert_id")
			.value(&self.last_insert_id)
			.finish()
	}
}

pub struct MssqlConnection {
	inner: Option<Client<Compat<TcpStream>>>,
	pool: Option<LongPooling>,
	cfg: SqlConfig
}

impl MssqlConnection {

	pub async fn establish(cfg: &SqlConfig) -> Result<Self, tiberius::error::Error> {
		let connect = connection::client(cfg).await?;
		let pooling = LongPooling::new(cfg)?;
		Ok(Self {
			cfg: cfg.to_owned(),
			inner: Some(connect),
			pool: Some(pooling)
		})
	}

	pub async fn select(
		&mut self,
		sql: &str,
		params: Vec<Value>,
	) -> Result<Vec<HashMap<String, Value>>, Error> {
		let mut q = Query::new(sql);
		for x in params {
			x.encode(&mut q)?;
		}
		let v = q
			.query(self.inner.as_mut().ok_or_else(||Error::from("MssqlConnection is close"))?)
			.await
			.map_err(|e| Error::from(e.to_string()))?;
		let mut results = Vec::with_capacity(v.size_hint().0);
		let s = v
			.into_results()
			.await
			.map_err(|e| Error::from(e.to_string()))?;
		for item in s {
			let (sx, rx) = kanal::unbounded();
			item.into_iter().for_each(|r|{
				let columns = r.columns().to_owned();
				let mut row = HashMap::with_capacity(columns.len());
				let mut count = 0;
				for x in r {
					let v = Value::decode(&x).unwrap();
					match columns.get(count) {
						None => {}
						Some(col) => {
							let name = col.name();
							row.insert(name.to_string(), v);
						}
					}
					count += 1;
				}
				sx.send(row).unwrap();
			});
			drop(sx);
			while let Ok(row) = rx.recv() {
				results.push(row);
			}
		}
		Ok(results)
	}

	pub async fn exec(
		&mut self,
		sql: &str,
		params: Vec<Value>,
	) -> Result<ExecResult, Error> {
		let mut q = Query::new(sql);
		for x in params {
			x.encode(&mut q)?;
		}
		let v = q
			.execute( self.inner.as_mut().ok_or_else(||Error::from("MssqlConnection is close"))?)
			.await
			.map_err(|e| Error::from(e.to_string()))?;
		Ok(ExecResult {
			rows_affected: {
				let mut rows_affected = 0;
				for x in v.rows_affected() {
					rows_affected += x.clone();
				}
				rows_affected
			},
			last_insert_id: Value::Int(None),
		})
	}

	async fn ping(&mut self) -> Result<(), Error> {
		let ping = self.inner
			.as_mut().expect("Mssql Connection is closed")
			.query("SELECT 1", &[])
			.await
			.map_err(|e|Error::from(e));
		match ping {
			Ok(_) => Ok(()),
			Err(err) => Err(err)
		}
	}

	pub async fn close(&mut self) -> Result<(), Error> {
		if let Some(v) = self.inner.take() {
			v.close().await.map_err(|e|Error::from(e))?;
		}
		Ok(())
	}
}