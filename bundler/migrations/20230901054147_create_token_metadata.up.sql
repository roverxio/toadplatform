-- Add up migration script here
alter table if exists supported_currencies rename to token_metadata;
