-- Add up migration script here
CREATE TABLE IF NOT EXISTS users
(
    email          VARCHAR               NOT NULL,
    wallet_address VARCHAR(42)           NOT NULL,
    salt           NUMERIC               NOT NULL,
    deployed       BOOLEAN DEFAULT false NOT NULL
);
