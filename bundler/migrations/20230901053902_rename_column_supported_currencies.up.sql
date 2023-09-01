-- Add up migration script here
alter table if exists supported_currencies rename column currency to symbol;
