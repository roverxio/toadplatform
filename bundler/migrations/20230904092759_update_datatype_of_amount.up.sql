-- Add up migration script here
alter table if exists user_transactions alter column amount type numeric using amount::numeric;