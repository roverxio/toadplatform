-- Add up migration script here
alter table if exists users alter column salt type numeric using salt::numeric;