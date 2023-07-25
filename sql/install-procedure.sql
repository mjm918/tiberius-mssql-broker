USE [<database>]

DECLARE @msg  VARCHAR(MAX)
DECLARE @crlf CHAR(1)
SET @crlf = CHAR(10)
SET @msg = 'Current user must have the following permissions: '
SET @msg = @msg +
           '[CREATE PROCEDURE, CREATE SERVICE, CREATE QUEUE, SUBSCRIBE QUERY NOTIFICATIONS, CONTROL, REFERENCES] '
SET @msg = @msg + 'that are required to start query notifications. '
SET @msg = @msg + 'Grant described permissions with following script: ' + @crlf
SET @msg = @msg + 'GRANT CREATE PROCEDURE TO [<username>];' + @crlf
SET @msg = @msg + 'GRANT CREATE SERVICE TO [<username>];' + @crlf
SET @msg = @msg + 'GRANT CREATE QUEUE  TO [<username>];' + @crlf
SET @msg = @msg + 'GRANT REFERENCES ON CONTRACT::[DEFAULT] TO [<username>];' + @crlf
SET @msg = @msg + 'GRANT SUBSCRIBE QUERY NOTIFICATIONS TO [<username>];' + @crlf
SET @msg = @msg + 'GRANT CONTROL ON SCHEMA::[<schemaname>] TO [<username>];'

PRINT @msg

IF OBJECT_ID('<schema>.<procedure>', 'P') IS NULL
    BEGIN
        EXEC ('
                    CREATE PROCEDURE <schema>.<procedure>
                    AS
                    BEGIN
                        -- Service Broker configuration statement.

            -- Setup Service Broker
            IF EXISTS (SELECT * FROM sys.databases
                                WHERE name = ''<database>'' AND is_broker_enabled = 0)
            BEGIN
                ALTER DATABASE [<database>] SET SINGLE_USER WITH ROLLBACK IMMEDIATE
                ALTER DATABASE [<database>] SET ENABLE_BROKER;
                ALTER DATABASE [<database>] SET MULTI_USER WITH ROLLBACK IMMEDIATE
                -- FOR SQL Express
                ALTER AUTHORIZATION ON DATABASE::[<database>] TO [<user>]
            END
            -- Create a queue which will hold the tracked information
            IF NOT EXISTS (SELECT * FROM sys.service_queues WHERE name = ''<schema>.<queue>'')
                CREATE QUEUE <schema>.[<queue>]
            -- Create a service on which tracked information will be sent
            IF NOT EXISTS(SELECT * FROM sys.services WHERE name = ''<schema>.<service>'')
                CREATE SERVICE [<service>] ON QUEUE <schema>.[<queue>] ([DEFAULT])

                        -- Notification Trigger check statement.

            IF OBJECT_ID (''<schema>.<trigger>'', ''TR'') IS NOT NULL
                RETURN;

                        -- Notification Trigger configuration statement.
                        DECLARE @triggerStatement NVARCHAR(MAX)
                        DECLARE @select NVARCHAR(MAX)
                        DECLARE @sqlInserted NVARCHAR(MAX)
                        DECLARE @sqlDeleted NVARCHAR(MAX)

                        SET @triggerStatement = N''
            CREATE TRIGGER [<trigger>]
            ON <schema>.[<table>]
            AFTER INSERT, UPDATE, DELETE
            AS
            SET NOCOUNT ON;
            --Trigger <table> is rising...
            IF EXISTS (SELECT * FROM sys.services WHERE name = ''''<service>'''')
            BEGIN
                DECLARE @message NVARCHAR(MAX)
                SET @message = N''''<root/>''''
                IF ( EXISTS(SELECT 1))
                BEGIN
                    DECLARE @retvalOUT NVARCHAR(MAX)
                    %inserted_select_statement%
                    IF (@retvalOUT IS NOT NULL)
                    BEGIN SET @message = N''''<root>'''' + @retvalOUT END
                    %deleted_select_statement%
                    IF (@retvalOUT IS NOT NULL)
                    BEGIN
                        IF (@message = N''''<root/>'''') BEGIN SET @message = N''''<root>'''' + @retvalOUT END
                        ELSE BEGIN SET @message = @message + @retvalOUT END
                    END
                    IF (@message != N''''<root/>'''') BEGIN SET @message = @message + N''''</root>'''' END
                END
                --Beginning of dialog...
                DECLARE @ConvHandle UNIQUEIDENTIFIER
                --Determine the Initiator Service, Target Service and the Contract
                BEGIN DIALOG @ConvHandle
                    FROM SERVICE [<service>] TO SERVICE ''''<service>'''' ON CONTRACT [DEFAULT] WITH ENCRYPTION=OFF, LIFETIME = 60;
                --Send the Message
                SEND ON CONVERSATION @ConvHandle MESSAGE TYPE [DEFAULT] (@message);
                --End conversation
                END CONVERSATION @ConvHandle;
            END
        ''

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

