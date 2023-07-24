# Tiberius MSSQL Broker

Broker for MSSQL. Re-implementation of C# SQLDependencyEx.

Thanks to deadpool-tiberius repository.

# Example:

```rust
// broker example 
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