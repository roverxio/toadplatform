-- Add up migration script here
CREATE TABLE IF NOT EXISTS token_metadata
(
    chain            VARCHAR                                            NOT NULL,
    symbol           VARCHAR                                            NOT NULL,
    contract_address VARCHAR                                            NOT NULL,
    exponent         INTEGER                                            NOT NULL,
    token_type       VARCHAR                                            NOT NULL,
    name             VARCHAR                                            NOT NULL,
    is_supported     BOOLEAN                  DEFAULT true              NOT NULL,
    created_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE (symbol, chain)
);
