-- Add up migration script here
ALTER TABLE IF EXISTS users ADD COLUMN IF NOT EXISTS name VARCHAR NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS users ADD COLUMN IF NOT EXISTS external_user_id VARCHAR NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS users ADD COLUMN IF NOT EXISTS owner_address VARCHAR(42) NOT NULL DEFAULT '';
