use deadpool::managed::{Object, PoolError};
use log::info;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, error::Error, SqlBrowser};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::config::SqlConfig;
use crate::deadpool::{Manager, Pool};

pub struct LongPooling {
    pool: Pool,
}

impl LongPooling {
    pub fn new(sql_config: &SqlConfig) -> Result<Self, tiberius::error::Error> {
        let pool = Manager::new(sql_config.sql_browser.to_owned())
            .host(sql_config.host.as_str())
            .port(sql_config.port.to_owned())
            .database(sql_config.database.as_str())
            .instance_name(match sql_config.instance {
                Some(ref instance) => instance,
                None => ""
            })
            .authentication(AuthMethod::sql_server(sql_config.username.as_str(), sql_config.password.as_str()))
            .trust_cert()
            .encryption(
                match sql_config.allow_encrypt {
                    true => EncryptionLevel::On,
                    false => EncryptionLevel::Off
                }
            )
            .max_size(sql_config.max_pool as usize)
            .wait_timeout(5)
            .pre_recycle_sync(|_client, _metrics| {
                Ok(())
            })
            .create_pool()?;
        Ok(Self {
            pool
        })
    }

    pub async fn client(&self) -> Result<Object<Manager>, PoolError<tiberius::error::Error>> {
        let pool = self.pool.get().await?;
        Ok(pool)
    }
}


pub async fn client(sql_config: &SqlConfig) -> Result<Client<Compat<TcpStream>>, tiberius::error::Error> {
    let mut config = Config::new();

    config.host(sql_config.host.as_str());
    match sql_config.instance {
        None => {}
        Some(ref instance) => {
            config.instance_name(instance);
        }
    }
    config.port(sql_config.port.to_owned());
    config.database(sql_config.database.as_str());
    config.authentication(AuthMethod::sql_server(sql_config.username.as_str(), sql_config.password.as_str()));

    config.encryption(
        match sql_config.allow_encrypt {
            true => EncryptionLevel::On,
            false => EncryptionLevel::Off
        }
    );
    if sql_config.trust_cert {
        config.trust_cert();
    }

    let tcp = if sql_config.sql_browser {
        TcpStream::connect_named(&config).await?
    } else {
        TcpStream::connect(config.get_addr()).await?
    };
    tcp.set_nodelay(true)?;

    let client = match Client::connect(config, tcp.compat_write()).await {
        Ok(client) => client,
        Err(Error::Routing { host, port }) => {
            info!("redirecting to {}:{}",&host,&port);
            let mut config = Config::new();
            config.host(&host);
            config.port(port);
            config.authentication(AuthMethod::sql_server(sql_config.username.as_str(), sql_config.password.as_str()));

            let tcp = TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;
            Client::connect(config, tcp.compat_write()).await?
        }
        Err(e) => Err(e)?,
    };

    Ok(client)
}

