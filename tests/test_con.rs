#[cfg(test)]
mod tests {
	use lib_mssql::config::SqlConfig;
	use lib_mssql::MssqlConnection;

	#[tokio::test]
	async fn test() {
		let config = SqlConfig {
			host: "d3.qne.cloud".to_string(),
			instance: None,
			port: 1433,
			username: "qnebss@qned3".to_string(),
			password: "QnE123!@#".to_string(),
			database: "OUC4Qy".to_string(),
			trust_cert: true,
			allow_encrypt: true,
			max_pool: 10,
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
