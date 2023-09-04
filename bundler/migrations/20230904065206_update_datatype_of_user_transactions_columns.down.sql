-- Add down migration script here
alter table if exists user_transactions alter column created_at type timestamp without time zone;
alter table if exists user_transactions alter column updated_at type timestamp without time zone;
