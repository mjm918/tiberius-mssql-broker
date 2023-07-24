#[derive(Clone, Debug)]
pub struct SqlConfig {
    pub host: String,
    pub instance: Option<String>,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub trust_cert: bool,
    pub allow_encrypt: bool,
    pub max_pool: u32,
    pub sql_browser: bool,
}

impl Default for SqlConfig {
    fn default() -> Self {
        Self {
            host: "(local)".to_string(),
            instance: None,
            port: 1433,
            username: "sa".to_string(),
            password: "".to_string(),
            database: "master".to_string(),
            trust_cert: true,
            allow_encrypt: true,
            max_pool: 1,
            sql_browser: false,
        }
    }
}
