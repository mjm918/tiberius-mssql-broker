#[cfg(test)]
mod tests {
	use std::time::Duration;
	use lib_mssql::config::SqlConfig;
	use lib_mssql::MssqlConnection;

	#[tokio::test]
	async fn test() {
		let config = SqlConfig {
			host: "d3.qne.cloud".to_string(),
			port: 1433,
			username: "qnebss@qned3".to_string(),
			password: "QnE123!@#".to_string(),
			database: "OUC4Qy".to_string(),
			trust_cert: true,
			allow_encrypt: true,
			max_pool: 1,
		};
		let conn = MssqlConnection::establish(&config).await;

		assert!(conn.is_ok());

		let conn = conn.unwrap();
		let (sx, rx) = kanal::unbounded();
		let broker = conn.listen(1,format!("SalesOrders"), sx).await;
		assert!(broker.is_ok());
		let broker = broker.unwrap();
		/*let _ = tokio::spawn(async move {
			let _ = broker.await;
		}).await;*/
		tokio::time::sleep(Duration::from_secs(3)).await;
	}
}