DECLARE @ConvHandle UNIQUEIDENTIFIER
DECLARE @message VARBINARY(MAX)
USE [AED_MOBILE]
WAITFOR (RECEIVE TOP(1) @ConvHandle=Conversation_Handle
    , @message=message_body FROM <schema>.[<queue>]), TIMEOUT 30000;
BEGIN TRY END CONVERSATION @ConvHandle; END TRY BEGIN CATCH END CATCH
SELECT CAST(@message AS NVARCHAR(MAX))