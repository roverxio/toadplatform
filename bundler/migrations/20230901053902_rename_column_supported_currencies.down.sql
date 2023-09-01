-- Add down migration script here
alter table if exists supported_currencies rename column symbol to currency;
