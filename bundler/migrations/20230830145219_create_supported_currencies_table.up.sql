-- Add up migration script here
CREATE TABLE IF NOT EXISTS supported_currencies (
    chain varchar not null,
    currency varchar not null,
    contract_address varchar not null,
    exponent int not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
