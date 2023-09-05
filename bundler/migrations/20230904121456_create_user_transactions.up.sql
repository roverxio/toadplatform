-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_transactions
(
    id             SERIAL PRIMARY KEY,
    user_address   VARCHAR(42)                                        NOT NULL,
    transaction_id VARCHAR                                            NOT NULL,
    from_address   VARCHAR(42)                                        NOT NULL,
    to_address     VARCHAR(42)                                        NOT NULL,
    amount         NUMERIC                                            NOT NULL,
    currency       VARCHAR                                            NOT NULL,
    type           VARCHAR(6)                                         NOT NULL,
    status         VARCHAR(10)                                        NOT NULL,
    metadata       JSONB                                              NOT NULL,
    created_at     TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at     TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
