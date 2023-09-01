-- Add down migration script here
alter table token_metadata drop constraint if exists symbol_chain_unique_key;
