-- Add up migration script here
alter table if exists token_metadata add constraint symbol_chain_unique_key unique (symbol, chain);
