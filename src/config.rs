#[derive(Clone, Debug)]
pub struct SqlConfig {
	pub host: String,
	pub port: u16,
	pub username: String,
	pub password: String,
	pub database: String,
	pub trust_cert: bool,
	pub allow_encrypt: bool,
	pub max_pool: u32
}

impl Default for SqlConfig {
	fn default() -> Self {
		Self {
			host: "(local)".to_string(),
			port: 1433,
			username: "sa".to_string(),
			password: "".to_string(),
			database: "master".to_string(),
			trust_cert: true,
			allow_encrypt: true,
			max_pool: 1
		}
	}
}