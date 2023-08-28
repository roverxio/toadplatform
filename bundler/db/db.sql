create table if not exists users (
    email varchar not null unique,
    wallet_address varchar(42) not null unique,
    salt varchar not null,
    deployed boolean not null default false
);

create table if not exists user_transactions (
    id integer primary key autoincrement,
    user_address varchar(42) not null,
    transaction_id varchar not null,
    from_address varchar(42) not null,
    to_address varchar(42) not null,
    amount numeric not null,
    currency varchar not null,
    type varchar(6) not null,
    status varchar(10) not null,
    metadata jsonb not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create table if not exists supported_currencies (
    currency varchar not null,
    contract_address varchar not null,
    exponent int not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
