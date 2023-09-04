-- Add down migration script here
alter table if exists token_metadata alter column created_at type timestamp without time zone;
alter table if exists token_metadata alter column updated_at type timestamp without time zone;
