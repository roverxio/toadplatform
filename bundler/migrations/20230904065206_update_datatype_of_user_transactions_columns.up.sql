-- Add up migration script here
alter table if exists user_transactions alter column created_at type timestamp with time zone;
alter table if exists user_transactions alter column updated_at type timestamp with time zone;
