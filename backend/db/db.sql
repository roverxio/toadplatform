create table if not exists users (
    email varchar not null,
    wallet_address varchar(42)
);

create table if not exists transactions (
    wallet_address varchar(42) not null,
    transaction_hash varchar(66) not null,
    created_at timestamp not null default current_timestamp
)
