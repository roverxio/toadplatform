-- Add down migration script here
alter table if exists token_metadata rename to supported_currencies;
