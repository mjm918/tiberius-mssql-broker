# Tiberius MSSQL Broker

Broker for MSSQL. Re-implementation of C# SQLDependencyEx.

Thanks to [deadpool-tiberius](https://github.com/Geo-W/deadpool-tiberius) and obviously [tiberius](https://github.com/prisma/tiberius) ❤️.

**Note**

Current user must have the following permissions:

```
CREATE PROCEDURE
CREATE SERVICE
CREATE QUEUE
SUBSCRIBE QUERY NOTIFICATIONS
CONTROL
REFERENCES
```

# Example:

**Broker example**

```rust
let mssql = MssqlConnection::establish(&SqlConfig{
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
}).await;
match mssql {
    Ok(conn) => {
        let (sx, rx) = kanal::unbounded::<Vec<ListenEvent>>();
        println!("started listening...");
        tokio::spawn(async move {
            while let Ok(evs) = rx.recv() {
                println!("{} {:?}",evs.len(),evs);
            }
        });
        let broker = conn.listen(1,format!("IV"), sx).await;
        match broker {
            Ok(_) => { }
            Err(err) => {
                println!("{:?}",err);
            }
        }
    }
    Err(err) => {
        println!("{:?}",err);
    }
}
```


**Query example**

```rust
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
let mut mssql = mssql.unwrap();

let res = mssql.select("SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE';", vec![]).await;
let res: Vec<HashMap<String,Value>> = res.unwrap();
```