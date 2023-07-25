#[cfg(test)]
mod tests {
	use tiberius_mssql_broker::config::SqlConfig;
	use tiberius_mssql_broker::MssqlConnection;

	#[tokio::test]
	async fn test() {
		let config = SqlConfig {
			host: ".".to_string(),
			instance: Some("SQLEXPRESS".to_string()),
			port: 1433,
			username: "sa".to_string(),
			password: "julfikar123@".to_string(),
			database: "AED_MOBILE".to_string(),
			trust_cert: true,
			allow_encrypt: true,
			max_pool: 1,
			sql_browser: false,
		};

		let mssql = MssqlConnection::establish(&config).await;
		assert!(mssql.is_ok());
		let mut mssql = mssql.unwrap();

		let res = mssql.select("SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE';", vec![]).await;
		assert!(res.is_ok());

		let res = res.unwrap();
		assert!(res.len() > 0);

		println!("res {:?}",res);
	}
}
