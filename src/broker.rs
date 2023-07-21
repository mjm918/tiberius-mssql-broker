use tiberius::{Result, error::Error, ExecuteResult, Row};
use crate::config::SqlConfig;
use crate::connection::LongPooling;
use crate::deadpool::Manager;

pub enum ActionType {
	None,
	Insert,
	Update,
	Delete
}

pub const INSERT_TAG: &str = "inserted";
pub const DELETE_TAG: &str = "deleted";

pub const SQL_PERMISSIONS_INFO: &str = r#"
	DECLARE @msg VARCHAR(MAX)
	DECLARE @crlf CHAR(1)
	SET @crlf = CHAR(10)
	SET @msg = 'Current user must have following permissions: '
	SET @msg = @msg + '[CREATE PROCEDURE, CREATE SERVICE, CREATE QUEUE, SUBSCRIBE QUERY NOTIFICATIONS, CONTROL, REFERENCES] '
	SET @msg = @msg + 'that are required to start query notifications. '
	SET @msg = @msg + 'Grant described permissions with following script: ' + @crlf
	SET @msg = @msg + 'GRANT CREATE PROCEDURE TO [<username>];' + @crlf
	SET @msg = @msg + 'GRANT CREATE SERVICE TO [<username>];' + @crlf
	SET @msg = @msg + 'GRANT CREATE QUEUE  TO [<username>];' + @crlf
	SET @msg = @msg + 'GRANT REFERENCES ON CONTRACT::[DEFAULT] TO [<username>];' + @crlf
	SET @msg = @msg + 'GRANT SUBSCRIBE QUERY NOTIFICATIONS TO [<username>];' + @crlf
	SET @msg = @msg + 'GRANT CONTROL ON SCHEMA::[<schemaname>] TO [<username>];'

	PRINT @msg
"#;

pub const SQL_FORMAT_CREATE_INSTALLATION_PROCEDURE: &str = r#"
	USE [<database>]
	<permission_info>
	IF OBJECT_ID ('<schema>.<proc>', 'P') IS NULL
		BEGIN
			EXEC ('
				CREATE PROCEDURE <schema>.<proc>
				AS
				BEGIN
					-- Service Broker configuration statement.
					<broker_config>
					-- Notification Trigger check statement.
					<notification_trigger>
					-- Notification Trigger configuration statement.
					DECLARE @triggerStatement NVARCHAR(MAX)
					DECLARE @select NVARCHAR(MAX)
					DECLARE @sqlInserted NVARCHAR(MAX)
					DECLARE @sqlDeleted NVARCHAR(MAX)

					SET @triggerStatement = N''<notification_config>''

					SET @select = STUFF((SELECT '','' + ''['' + COLUMN_NAME + '']''
										 FROM INFORMATION_SCHEMA.COLUMNS
										 WHERE DATA_TYPE NOT IN  (''text'',''ntext'',''image'',''geometry'',''geography'') AND TABLE_SCHEMA = ''<schema>'' AND TABLE_NAME = ''<table>'' AND TABLE_CATALOG = ''<database>''
										 FOR XML PATH ('''')
										 ), 1, 1, '''')
					SET @sqlInserted =
						N''SET @retvalOUT = (SELECT '' + @select + N''
											 FROM INSERTED
											 FOR XML PATH(''''row''''), ROOT (''''inserted''''))''
					SET @sqlDeleted =
						N''SET @retvalOUT = (SELECT '' + @select + N''
											 FROM DELETED
											 FOR XML PATH(''''row''''), ROOT (''''deleted''''))''
					SET @triggerStatement = REPLACE(@triggerStatement
											 , ''%inserted_select_statement%'', @sqlInserted)
					SET @triggerStatement = REPLACE(@triggerStatement
											 , ''%deleted_select_statement%'', @sqlDeleted)
					EXEC sp_executesql @triggerStatement
				END
				')
		END
"#;

pub const SQL_FORMAT_CREATE_UNINSTALLATION_PROCEDURE: &str = r#"
	USE [<database>]
	<permission_info>
	IF OBJECT_ID ('<schema>.<prev_proc>', 'P') IS NULL
		BEGIN
			EXEC ('
				CREATE PROCEDURE <schema>.<prev_proc>
				AS
				BEGIN
					-- Notification Trigger drop statement.
					<uninstall_stmt>
					-- Service Broker uninstall statement.
					<notification_trigger_drop_stmt>
					IF OBJECT_ID (''<schema>.<next_proc>'', ''P'') IS NOT NULL
						DROP PROCEDURE <schema>.<next_proc>

					DROP PROCEDURE <schema>.<prev_proc>
				END
				')
		END
"#;

pub const SQL_FORMAT_INSTALL_SERVICE_BROKER_NOTIFICATION: &str = r#"
	-- Setup Service Broker
	IF EXISTS (SELECT * FROM sys.databases
						WHERE name = '<database>' AND is_broker_enabled = 0)
	BEGIN
		ALTER DATABASE [<database>] SET SINGLE_USER WITH ROLLBACK IMMEDIATE
		ALTER DATABASE [<database>] SET ENABLE_BROKER;
		ALTER DATABASE [<database>] SET MULTI_USER WITH ROLLBACK IMMEDIATE
		-- FOR SQL Express
		ALTER AUTHORIZATION ON DATABASE::[<database>] TO [<user>]
	END
	-- Create a queue which will hold the tracked information
	IF NOT EXISTS (SELECT * FROM sys.service_queues WHERE name = '<schema>.<queue>')
		CREATE QUEUE <schema>.[<queue>]
	-- Create a service on which tracked information will be sent
	IF NOT EXISTS(SELECT * FROM sys.services WHERE name = '<schema>.<service>')
		CREATE SERVICE [<service>] ON QUEUE <schema>.[<queue>] ([DEFAULT])
"#;

pub const SQL_FORMAT_UNINSTALL_SERVICE_BROKER_NOTIFICATION: &str = r#"
	DECLARE @serviceId INT
	SELECT @serviceId = service_id FROM sys.services
	WHERE sys.services.name = '<service>'
	DECLARE @ConvHandle uniqueidentifier
	DECLARE Conv CURSOR FOR
	SELECT CEP.conversation_handle FROM sys.conversation_endpoints CEP
	WHERE CEP.service_id = @serviceId AND ([state] != 'CD' OR [lifetime] > GETDATE() + 1)
	OPEN Conv;
	FETCH NEXT FROM Conv INTO @ConvHandle;
	WHILE (@@FETCH_STATUS = 0) BEGIN
		END CONVERSATION @ConvHandle WITH CLEANUP;
		FETCH NEXT FROM Conv INTO @ConvHandle;
	END
	CLOSE Conv;
	DEALLOCATE Conv;
	-- Droping service and queue.
	IF (@serviceId IS NOT NULL)
		DROP SERVICE [<service>];
	IF OBJECT_ID ('<schema>.<queue>', 'SQ') IS NOT NULL
		DROP QUEUE <schema>.[<queue>];
"#;

pub const SQL_FORMAT_DELETE_NOTIFICATION_TRIGGER:&str = r#"
	IF OBJECT_ID ('<schema>.<trigger>', 'TR') IS NOT NULL
                    DROP TRIGGER <schema>.[<trigger>];
"#;

pub const SQL_FORMAT_CHECK_NOTIFICATION_TRIGGER: &str = r#"
	IF OBJECT_ID ('<schema>.<trigger>', 'TR') IS NOT NULL
                    RETURN;
"#;

pub const SQL_FORMAT_CREATE_NOTIFICATION_TRIGGER: &str = r#"
	CREATE TRIGGER [<trigger>]
	ON <schema>.[<table>]
	AFTER INSERT, DELETE, UPDATE
	AS
	SET NOCOUNT ON;
	--Trigger <table> is rising...
	IF EXISTS (SELECT * FROM sys.services WHERE name = '<conversation>')
	BEGIN
		DECLARE @message NVARCHAR(MAX)
		SET @message = N'<root/>'
		IF (<tracking_mode> EXISTS(SELECT 1))
		BEGIN
			DECLARE @retvalOUT NVARCHAR(MAX)
			%inserted_select_statement%
			IF (@retvalOUT IS NOT NULL)
			BEGIN SET @message = N'<root>' + @retvalOUT END
			%deleted_select_statement%
			IF (@retvalOUT IS NOT NULL)
			BEGIN
				IF (@message = N'<root/>') BEGIN SET @message = N'<root>' + @retvalOUT END
				ELSE BEGIN SET @message = @message + @retvalOUT END
			END
			IF (@message != N'<root/>') BEGIN SET @message = @message + N'</root>' END
		END
		--Beginning of dialog...
		DECLARE @ConvHandle UNIQUEIDENTIFIER
		--Determine the Initiator Service, Target Service and the Contract
		BEGIN DIALOG @ConvHandle
			FROM SERVICE [<conversation>] TO SERVICE '<conversation>' ON CONTRACT [DEFAULT] WITH ENCRYPTION=OFF, LIFETIME = 60;
		--Send the Message
		SEND ON CONVERSATION @ConvHandle MESSAGE TYPE [DEFAULT] (@message);
		--End conversation
		END CONVERSATION @ConvHandle;
	END
"#;

pub const SQL_FORMAT_RECEIVE_EVENT: &str = r#"
	DECLARE @ConvHandle UNIQUEIDENTIFIER
	DECLARE @message VARBINARY(MAX)
	USE [<database>]
	WAITFOR (RECEIVE TOP(1) @ConvHandle=Conversation_Handle
				, @message=message_body FROM <schema>.[<conversation>]), TIMEOUT 60000;
	BEGIN TRY END CONVERSATION @ConvHandle; END TRY BEGIN CATCH END CATCH
	SELECT CAST(@message AS NVARCHAR(MAX))
"#;

pub const SQL_FORMAT_EXECUTE_PROCEDURE: &str = r#"
	USE [<database>]
	IF OBJECT_ID ('<schema>.<proc>', 'P') IS NOT NULL
		EXEC <schema>.<proc>
"#;

pub const SQL_FORMAT_GET_DEPENDENCY_IDENTITIES: &str = r#"
	USE [<database>]

	SELECT REPLACE(name , 'ListenerService_' , '')
	FROM sys.services
	WHERE [name] like 'ListenerService_%';
"#;

pub const SQL_FORMAT_FORCED_DATABASE_CLEANING: &str = r#"
	USE [<database>]
	DECLARE @db_name VARCHAR(MAX)
	SET @db_name = '<database>' -- provide your own db name
	DECLARE @proc_name VARCHAR(MAX)
	DECLARE procedures CURSOR
	FOR
	SELECT   sys.schemas.name + '.' + sys.objects.name
	FROM    sys.objects
	INNER JOIN sys.schemas ON sys.objects.schema_id = sys.schemas.schema_id
	WHERE sys.objects.[type] = 'P' AND sys.objects.[name] like 'sp_UninstallListenerNotification_%'
	OPEN procedures;
	FETCH NEXT FROM procedures INTO @proc_name
	WHILE (@@FETCH_STATUS = 0)
	BEGIN
	EXEC ('USE [' + @db_name + '] EXEC ' + @proc_name + ' IF (OBJECT_ID ('''
					+ @proc_name + ''', ''P'') IS NOT NULL) DROP PROCEDURE '
					+ @proc_name)
	FETCH NEXT FROM procedures INTO @proc_name
	END
	CLOSE procedures;
	DEALLOCATE procedures;
	DECLARE procedures CURSOR
	FOR
	SELECT   sys.schemas.name + '.' + sys.objects.name
	FROM    sys.objects
	INNER JOIN sys.schemas ON sys.objects.schema_id = sys.schemas.schema_id
	WHERE sys.objects.[type] = 'P' AND sys.objects.[name] like 'sp_InstallListenerNotification_%'
	OPEN procedures;
	FETCH NEXT FROM procedures INTO @proc_name
	WHILE (@@FETCH_STATUS = 0)
	BEGIN
	EXEC ('USE [' + @db_name + '] DROP PROCEDURE '
					+ @proc_name)
	FETCH NEXT FROM procedures INTO @proc_name
	END
	CLOSE procedures;
	DEALLOCATE procedures;
"#;

pub const SQL_FORMAT_INSTALL_SEVICE_BROKER_NOTIFICATION: &str = r#"
	-- Setup Service Broker
	IF EXISTS (SELECT * FROM sys.databases
						WHERE name = '<database>' AND is_broker_enabled = 0)
	BEGIN
		ALTER DATABASE [<database>] SET SINGLE_USER WITH ROLLBACK IMMEDIATE
		ALTER DATABASE [<database>] SET ENABLE_BROKER;
		ALTER DATABASE [<database>] SET MULTI_USER WITH ROLLBACK IMMEDIATE
		-- FOR SQL Express
		ALTER AUTHORIZATION ON DATABASE::[<database>] TO [<user>]
	END
	-- Create a queue which will hold the tracked information
	IF NOT EXISTS (SELECT * FROM sys.service_queues WHERE name = '<schema>.<queue>')
		CREATE QUEUE <schema>.[<queue>]
	-- Create a service on which tracked information will be sent
	IF NOT EXISTS(SELECT * FROM sys.services WHERE name = '<schema>.<service>')
		CREATE SERVICE [<service>] ON QUEUE <schema>.[<queue>] ([DEFAULT])
"#;

fn conversation_queue(name: &str) -> String {
	format!("ListenerQueue_{}",name)
}

fn conversation_service(name: &str) -> String {
	format!("ListenerService_{}",name)
}

fn conversation_trigger(name: &str) -> String {
	format!("tr_Listener_{}",name)
}

fn install_proc_listener(name: &str) -> String {
	format!("sp_InstallListenerNotification_{}",name)
}

fn uninstall_proc_listener(name: &str) -> String {
	format!("sp_UninstallListenerNotification_{}",name)
}

pub struct Broker<'a> {
	pool: LongPooling,
	cnf: SqlConfig,
	schema: &'a str,
	table: String,
	identifier: u64,
	producer: kanal::Sender<Vec<Vec<Row>>>
}

impl<'a> Broker<'a> {
	pub fn new(
		pool: LongPooling,
		cnf: SqlConfig,
		table: String,
		identifier: u64,
		producer: kanal::Sender<Vec<Vec<Row>>>
	) -> Self {
		Self {
			pool,
			cnf,
			schema: "dbo",
			table,
			identifier,
			producer
		}
	}

	pub async fn start(&mut self) -> std::result::Result<(), Error> {
		self.stop().await?;

		let id = self.identifier.to_string();
		let install_proc = install_proc_listener(id.as_str());
		let sql = SQL_FORMAT_EXECUTE_PROCEDURE
			.replace("<database>",self.cnf.database.as_str())
			.replace("<proc>",install_proc.as_str())
			.replace("<schema>",&self.schema);

		self.exec(self.install_proc_script().as_str()).await?;
		self.exec(self.uninstall_proc_script().as_str()).await?;
		self.exec(sql.as_str()).await?;
		loop {
			if let Ok(result) = self.receive_event().await {
				self.producer.send(result).expect("send notification from broker");
			}
		}
	}

	async fn receive_event(&mut self) -> Result<Vec<Vec<Row>>> {
		let q = conversation_queue(self.identifier.to_string().as_str());
		let sql = SQL_FORMAT_RECEIVE_EVENT
			.replace("<database>",self.cnf.database.as_str())
			.replace("<conversation>",&q)
			.replace("<schema>",&self.schema);
		let client = self.pool.client().await;
		let mut conn = client.expect("Mssql Connection is closed");
		let stream = conn.simple_query(sql.as_str()).await?;
		let rows = stream
			.into_results()
			.await?;
		Ok(rows)
	}

	pub async fn stop(&mut self) -> Result<ExecuteResult> {
		let id = self.identifier.to_string();
		let uninstall_proc = uninstall_proc_listener(id.as_str());
		let sql = SQL_FORMAT_EXECUTE_PROCEDURE
			.replace("<database>",self.cnf.database.as_str())
			.replace("<proc>",uninstall_proc.as_str())
			.replace("<schema>",&self.schema);

		self.exec(sql.as_str()).await
	}

	pub async fn clean(&mut self) -> Result<ExecuteResult> {
		let sql = SQL_FORMAT_FORCED_DATABASE_CLEANING
			.replace("<database>",self.cnf.database.as_str());
		self.exec(sql.as_str()).await
	}

	async fn exec(&mut self, sql: &str) -> Result<ExecuteResult> {
		println!("To Execute: {}",sql);
		let client = self.pool.client().await;
		let mut conn = client.expect("Mssql Connection is closed");
		conn.execute("SELECT 1;", &[]).await
	}

	fn install_proc_script(&self) -> String {
		let id = self.identifier.to_string();
		let p = install_proc_listener(id.as_str());
		let c = conversation_trigger(id.as_str());
		let q = conversation_queue(id.as_str());
		let s = conversation_service(id.as_str());

		let install_service = SQL_FORMAT_INSTALL_SEVICE_BROKER_NOTIFICATION
			.replace("<database>", self.cnf.database.as_str())
			.replace("<user>",self.cnf.username.as_str())
			.replace("<queue>",&q)
			.replace("<service>",&s)
			.replace("<schema>",&self.schema);
		let install_trigger = SQL_FORMAT_CREATE_NOTIFICATION_TRIGGER
			.replace("<table>",&self.table)
			.replace("<conversation>",&s)
			.replace("<trigger>",&s)
			.replace("<tracking_mode>","")
			.replace("<schema>",&self.schema);
		let uninstall_trigger = SQL_FORMAT_CHECK_NOTIFICATION_TRIGGER
			.replace("<trigger>",&c)
			.replace("<schema>",&self.schema);
		let install_proc = SQL_FORMAT_CREATE_INSTALLATION_PROCEDURE
			.replace("<database>",self.cnf.database.as_str())
			.replace("<permission_info>",SQL_PERMISSIONS_INFO)
			.replace("<schema>",&self.schema)
			.replace("<table>",&self.table)
			.replace("<proc>",&p)
			.replace("<broker_config>",install_service.replace("'", "''").as_str())
			.replace("<notification_trigger>",install_trigger.replace("'", "''").as_str())
			.replace("<notification_config>", uninstall_trigger.replace("'", "''").as_str());

		install_proc
	}

	fn uninstall_proc_script(&self) -> String {
		let id = self.identifier.to_string();
		let q = conversation_queue(id.as_str());
		let s = conversation_service(id.as_str());
		let t = conversation_trigger(id.as_str());
		let u = uninstall_proc_listener(id.as_str());
		let n = install_proc_listener(id.as_str());
		let uninstall_service = SQL_FORMAT_UNINSTALL_SERVICE_BROKER_NOTIFICATION
			.replace("<service>",&s)
			.replace("<schema>",&self.schema)
			.replace("<queue>",&q);
		let uninstall_notification = SQL_FORMAT_DELETE_NOTIFICATION_TRIGGER
			.replace("<trigger>",&t)
			.replace("<schema>",&s);
		let script = SQL_FORMAT_CREATE_UNINSTALLATION_PROCEDURE
			.replace("<database>",self.cnf.database.as_str())
			.replace("<prev_proc>",&u)
			.replace("<next_proc>",&n)
			.replace("<schema>",&self.schema)
			.replace("<permission_info>",SQL_PERMISSIONS_INFO)
			.replace("<notification_trigger_drop_stmt>",uninstall_service.as_str())
			.replace("<uninstall_stmt>",uninstall_notification.as_str());
		script
	}
}
