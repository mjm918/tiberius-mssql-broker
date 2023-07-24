USE [<database>]
IF OBJECT_ID('<schema>.<uninstall_procedure>', 'P') IS NOT NULL
    EXEC <schema>.<uninstall_procedure>
