-- Add down migration script here
alter table if exists users alter column salt type varchar;