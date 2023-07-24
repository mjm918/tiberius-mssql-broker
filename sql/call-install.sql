USE [<database>]
IF OBJECT_ID('<schema>.<procedure>', 'P') IS NOT NULL
    EXEC <schema>.<procedure>
