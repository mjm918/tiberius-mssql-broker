USE [<database>]

DECLARE @msg  VARCHAR(MAX)
DECLARE @crlf CHAR(1)
SET @crlf = CHAR(10)
SET @msg = 'Current user must have following permissions: '
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

IF OBJECT_ID('<schema>.<uninstall_procedure>', 'P') IS NULL
    BEGIN
        EXEC ('
                        CREATE PROCEDURE <schema>.<uninstall_procedure>
                        AS
                        BEGIN
                            -- Notification Trigger drop statement.

                IF OBJECT_ID (''<schema>.tr_Listener_1'', ''TR'') IS NOT NULL
                    DROP TRIGGER <schema>.[tr_Listener_1];

                            -- Service Broker uninstall statement.

                DECLARE @serviceId INT
                SELECT @serviceId = service_id FROM sys.services
                WHERE sys.services.name = ''<service>''
                DECLARE @ConvHandle uniqueidentifier
                DECLARE Conv CURSOR FOR
                SELECT CEP.conversation_handle FROM sys.conversation_endpoints CEP
                WHERE CEP.service_id = @serviceId AND ([state] != ''CD'' OR [lifetime] > GETDATE() + 1)
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
                IF OBJECT_ID (''<schema>.<queue>'', ''SQ'') IS NOT NULL
	                DROP QUEUE <schema>.[<queue>];

                            IF OBJECT_ID (''<schema>.<procedure>'', ''P'') IS NOT NULL
                                DROP PROCEDURE <schema>.<procedure>

                            DROP PROCEDURE <schema>.<uninstall_procedure>
                        END
                        ')
    END
