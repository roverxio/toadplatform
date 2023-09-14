CREATE TABLE IF NOT EXISTS user_operations
(
    transaction_id VARCHAR                                            NOT NULL,
    user_operation JSONB                                              NOT NULL,
    status         VARCHAR                  DEFAULT 'initiated'       NOT NULL,
    created_at     TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at     TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
