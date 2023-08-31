-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_transactions (
    id serial primary key,
    user_address varchar(42) not null,
    transaction_id varchar not null,
    from_address varchar(42) not null,
    to_address varchar(42) not null,
    amount varchar not null,
    currency varchar not null,
    type varchar(6) not null,
    status varchar(10) not null,
    metadata jsonb not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
