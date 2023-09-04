-- Add up migration script here
alter table if exists token_metadata alter column created_at type timestamp with time zone;
alter table if exists token_metadata alter column updated_at type timestamp with time zone;
