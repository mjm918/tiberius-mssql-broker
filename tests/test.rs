#[cfg(test)]
mod tests {
	use lib_mssql::config::SqlConfig;
	use lib_mssql::connection::{client, LongPooling};

	fn config() -> SqlConfig {
		let config = SqlConfig {
			host: "d3.qne.cloud".to_string(),
			port: 1433,
			username: "qnebss@qned3".to_string(),
			password: "QnE123!@#".to_string(),
			database: "OUC4Qy".to_string(),
			trust_cert: true,
			allow_encrypt: true,
			max_pool: 0,
		};
		config
	}

	#[tokio::test]
	async fn simple_connect() {
		let config = config();

		let conn = client(&config).await;
		assert!(conn.is_ok(),"{:?}",conn.err());

		let mut conn = conn.unwrap();

		let res = conn.simple_query("select * from accounts;").await;
		assert!(res.is_ok(),"{:?}",res.err());
	}

	#[tokio::test]
	async fn pooling_connect() {
		let config = config();

		let pool = LongPooling::new(&config);
		assert!(pool.is_ok(),"{:?}",pool.err());

		let pool = pool.unwrap();
		let client = pool.client().await;
		assert!(client.is_ok(),"{:?}",client.err());
	}

	#[tokio::test]
	async fn query() {
		let config = config();

		let pool = LongPooling::new(&config);
		assert!(pool.is_ok(),"{:?}",pool.err());

		let pool = pool.unwrap();
		let client = pool.client().await;
		assert!(client.is_ok(),"{:?}",client.err());

		let mut client = client.unwrap();
		let res = client.simple_query(r#"
			SELECT * FROM Accounts;
		"#).await;

		let res = res.unwrap();
		let rows = res.into_results().await;
		assert!(rows.is_ok(),"{:?}",rows.err());

		let rows = rows.unwrap();
		assert!(rows.len() > 0);
	}
}