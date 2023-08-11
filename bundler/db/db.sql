create table if not exists users (
    email varchar not null unique,
    wallet_address varchar(42) not null unique,
    salt varchar not null,
    deployed boolean not null default false
);

create table if not exists transactions (
    wallet_address varchar(42) not null,
    transaction_hash varchar(66) not null,
    created_at timestamp not null default current_timestamp
)
