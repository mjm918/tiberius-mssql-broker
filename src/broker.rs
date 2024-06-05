use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use log::{error, trace};
use serde_json::Value as Json;
use tiberius::{error::Error, ExecuteResult, Result};

use crate::cnv;
use crate::config::SqlConfig;
use crate::connection::LongPooling;
use crate::decode::Decode;
use crate::json_ext::{JsonExt, JsonMapExt};
use crate::value::Value;

#[derive(Debug)]
pub struct ListenEvent {
    pub inserted: Option<Vec<HashMap<String, Value>>>,
    pub updated: Option<Vec<HashMap<String, Value>>>,
    pub deleted: Option<Vec<HashMap<String, Value>>>,
}

fn conversation_queue(name: &str) -> String {
    format!("ListenerQueue_{}", name)
}

fn conversation_service(name: &str) -> String {
    format!("ListenerService_{}", name)
}

fn conversation_trigger(name: &str) -> String {
    format!("tr_Listener_{}", name)
}

fn install_proc_listener(name: &str) -> String {
    format!("sp_InstallListenerNotification_{}", name)
}

fn uninstall_proc_listener(name: &str) -> String {
    format!("sp_UninstallListenerNotification_{}", name)
}

const SCHEMA: &str = "dbo";

pub struct Broker {
    pool: LongPooling,
    cnf: SqlConfig,
    table: String,
    identifier: u64,
    producer: kanal::Sender<Vec<ListenEvent>>,
    definition: HashMap<String, String>,
}

impl Broker {
    pub fn new(
        pool: LongPooling,
        cnf: SqlConfig,
        table: String,
        identifier: u64,
        producer: kanal::Sender<Vec<ListenEvent>>,
    ) -> Self {
        Self {
            pool,
            cnf,
            table,
            identifier,
            producer,
            definition: HashMap::new(),
        }
    }

    pub async fn start(&mut self) -> std::result::Result<(), Error> {
        trace!("stopping previous listeners");
        self.stop().await?;

        trace!("installing procedures");

        let install_sql = Self::install_procedure_sql().expect("install procedure sql");
        let uninstall_sql = Self::uninstall_procedure_sql().expect("uninstall procedure sql");
        let call_procedure = Self::call_install_procedure_sql().expect("call procedure sql");

        self.exec(install_sql.as_str()).await?;
        self.exec(uninstall_sql.as_str()).await?;
        self.exec(call_procedure.as_str()).await?;
        self.definitions().await?;
        trace!("started listening to changes");

        loop {
            match self.receive_event().await {
                Ok(result) => {
                    if result.len() > 0 {
                        trace!("received {:?}", &result);
                        self.producer.send(result).expect("send notification from sql");
                    }
                }
                Err(err) => {
                    trace!("{:?}",err);
                }
            }
        }
    }

    async fn receive_event(&mut self) -> Result<Vec<ListenEvent>> {
        let q = conversation_queue(self.identifier.to_string().as_str());
        let sql = r#"
				DECLARE @ConvHandle UNIQUEIDENTIFIER
				DECLARE @message VARBINARY(MAX)
				USE [<database>]
				WAITFOR (RECEIVE TOP(1) @ConvHandle=Conversation_Handle
					, @message=message_body FROM <schema>.[<queue>]), TIMEOUT 60000;
				BEGIN TRY END CONVERSATION @ConvHandle; END TRY BEGIN CATCH END CATCH
				SELECT CAST(@message AS NVARCHAR(MAX))
			"#
            .replace("<database>", self.cnf.database.as_str())
            .replace("<queue>", &q)
            .replace("<schema>", &SCHEMA);
        let client = self.pool.client().await;
        let mut conn = client.expect("Mssql Connection is closed");
        let stream = conn.simple_query(sql.as_str()).await?;
        let rows = stream
            .into_results()
            .await?;

        let mut results = vec![];
        for first in rows {
            for row in first {
                for column_data in row {
                    let v = Value::decode(&column_data).unwrap();
                    match v {
                        Value::String(opt_xml) => {
                            match opt_xml {
                                None => {}
                                Some(xml) => {
                                    match quickxml_to_serde::xml_str_to_json(xml.as_ref(), &quickxml_to_serde::Config::new_with_defaults())/*serde_xml_rs::from_str::<Json>(xml.as_str())*/ {
                                        Ok(json) => {
                                            results.push(self.normalize(&json));
                                        }
                                        Err(_) => {}
                                    };
                                }
                            }
                        }
                        _ => {
                            error!("unexpected data {:?}",column_data)
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    pub async fn stop(&mut self) -> Result<ExecuteResult> {
        let sql = Self::call_uninstall_procedure_sql().expect("call uninstall procedure sql");
        self.exec(sql.as_str()).await
    }

    pub async fn clean(&mut self) -> Result<ExecuteResult> {
        let sql = Self::cleanup_sql().expect("cleanup sql");
        self.exec(sql.as_str()).await
    }

    async fn exec(&mut self, sql: &str) -> Result<ExecuteResult> {
        let id = self.identifier.to_string();
        let sql = sql
            .replace("<database>", &self.cnf.database)
            .replace("<user>", &self.cnf.username)
            .replace("<procedure>", install_proc_listener(&id).as_str())
            .replace("<uninstall_procedure>", uninstall_proc_listener(&id).as_str())
            .replace("<service>", conversation_service(&id).as_str())
            .replace("<queue>", conversation_queue(&id).as_str())
            .replace("<trigger>", conversation_trigger(&id).as_str())
            .replace("<schema>", &SCHEMA)
            .replace("<table>", &self.table);

        trace!("To Execute: {}",sql);

        let client = self.pool.client().await;
        let mut conn = client.expect("Mssql Connection is closed");
        conn.execute(sql, &[]).await
    }

    pub async fn definitions(&mut self) -> Result<()> {
        self.definition.clear();
        let sql = format!(
	        r#"
	        SELECT COLUMN_NAME, DATA_TYPE
	        FROM INFORMATION_SCHEMA.COLUMNS
	        WHERE TABLE_NAME = '{}' ORDER BY ORDINAL_POSITION;
	        "#,
	        self.table
	    );
        let client = self.pool.client().await;
        let mut conn = client.expect("Mssql Connection is closed");
        let stream = conn.simple_query(sql).await?;
        let rows = stream
            .into_results()
            .await?;
        for first in rows {
            for row in first {
                let mut count = 0;
                let mut column_name = format!("");
                let mut type_def = format!("");

                for column_data in row {
                    let v = Value::decode(&column_data).unwrap();
                    match v {
                        Value::String(def) => {
                            if count == 0 {
                                column_name = format!("{}", def.unwrap());
                            } else if count == 1 {
                                type_def = format!("{}", def.unwrap());
                            }
                        }
                        _ => {}
                    }
                    count += 1;
                }
                self.definition.insert(column_name, type_def);
            }
        }
        Ok(())
    }

    fn normalize(&self, root: &Json) -> ListenEvent {
        let obj = root.to_object();
        let value = obj.get_object("root");
        let mut ev = ListenEvent {
            inserted: None,
            updated: None,
            deleted: None,
        };
        match value.get("deleted") {
            None => {}
            Some(deleted) => {
                trace!("received event [delete]");
                ev.deleted = Some(self.parse_root_row(deleted));
            }
        }
        match value.get("inserted") {
            None => {}
            Some(inserted) => {
                trace!("received event [insert]");
                ev.inserted = Some(self.parse_root_row(inserted));
            }
        }
        match value.get("updated") {
            None => {}
            Some(updated) => {
                trace!("received event [update]");
                ev.updated = Some(self.parse_root_row(updated));
            }
        }
        ev
    }

    fn parse_root_row(&self, value: &Json) -> Vec<HashMap<String, Value>> {
        let action = value.to_object();
        let rows = action.get_array("row");
        let mut result = vec![];
        for row in rows {
            trace!("event [row] {:?}",&row);
            result.push(self.parse_row(&row));
        }
        result
    }

    fn parse_row(&self, value: &Json) -> HashMap<String, Value> {
        let hm = value.to_object();
        let mut res = HashMap::new();
        for (column, v) in hm {
            let column_value = v.any_to_str();

            let tmp = "".to_string();
            let data_type = self.definition.get(column.as_str()).unwrap_or(&tmp);

            let converted = cnv::convert_from_str_to_rusttype(column_value.as_str(), data_type);
            trace!("converted value {:?}",converted);

            res.insert(column.clone(), converted);
        }
        res
    }

    fn install_procedure_sql() -> std::io::Result<String> {
        fs::read_to_string(Self::sql_path("install-procedure.sql"))
    }

    fn uninstall_procedure_sql() -> std::io::Result<String> {
        fs::read_to_string(Self::sql_path("uninstall-procedure.sql"))
    }

    fn call_install_procedure_sql() -> std::io::Result<String> {
        fs::read_to_string(Self::sql_path("call-install.sql"))
    }

    fn call_uninstall_procedure_sql() -> std::io::Result<String> {
        fs::read_to_string(Self::sql_path("call-uninstall.sql"))
    }

    fn cleanup_sql() -> std::io::Result<String> {
        fs::read_to_string(Self::sql_path("cleanup.sql"))
    }

    fn sql_path(name: &str) -> PathBuf {
        let path = Path::new(".")
            .join("sql")
            .join(name);
        path
    }
}

/*impl Drop for Broker {
    fn drop(&mut self) {
        let _ = Box::new(async move {
            self.stop().await
        });
    }
}*/
