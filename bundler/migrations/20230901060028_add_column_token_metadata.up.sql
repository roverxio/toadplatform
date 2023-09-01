-- Add up migration script here
alter table if exists token_metadata add column if not exists token_type varchar not null default '';
alter table if exists token_metadata add column if not exists name varchar not null default '';
alter table if exists token_metadata add column if not exists is_supported boolean not null default true;
