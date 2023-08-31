-- Add up migration script here
create table if not exists users (
    email varchar not null unique,
    wallet_address varchar(42) not null unique,
    salt varchar not null,
    deployed boolean not null default false
);
