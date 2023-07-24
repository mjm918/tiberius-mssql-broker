USE
[<database>]
                DECLARE
@db_name VARCHAR(MAX)
                SET @db_name = '<database>' -- provide your own db name
                DECLARE
@proc_name VARCHAR(MAX)
                DECLARE
procedures CURSOR
                FOR
SELECT sys.schemas.name + '.' + sys.objects.name
FROM sys.objects
         INNER JOIN sys.schemas ON sys.objects.schema_id = sys.schemas.schema_id
WHERE sys.objects.[type] = 'P'
  AND sys.objects.[name] like 'sp_UninstallListenerNotification_%' OPEN procedures;
FETCH NEXT FROM procedures INTO @proc_name WHILE (@@FETCH_STATUS = 0)
BEGIN
EXEC ('USE [' + @db_name + '] EXEC ' + @proc_name + ' IF (OBJECT_ID ('''
                                + @proc_name + ''', ''P'') IS NOT NULL) DROP PROCEDURE '
                                + @proc_name)
                FETCH NEXT FROM procedures INTO @proc_name
END
CLOSE procedures;
DEALLOCATE
procedures;
                DECLARE
procedures CURSOR
                FOR
SELECT sys.schemas.name + '.' + sys.objects.name
FROM sys.objects
         INNER JOIN sys.schemas ON sys.objects.schema_id = sys.schemas.schema_id
WHERE sys.objects.[type] = 'P'
  AND sys.objects.[name] like 'sp_InstallListenerNotification_%' OPEN procedures;
FETCH NEXT FROM procedures INTO @proc_name WHILE (@@FETCH_STATUS = 0)
BEGIN
EXEC ('USE [' + @db_name + '] DROP PROCEDURE '
                                + @proc_name)
                FETCH NEXT FROM procedures INTO @proc_name
END
CLOSE procedures;
DEALLOCATE
procedures;
