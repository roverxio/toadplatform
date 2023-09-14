-- Add down migration script here
ALTER TABLE IF EXISTS users DROP COLUMN IF EXISTS name;
ALTER TABLE IF EXISTS users DROP COLUMN IF EXISTS external_user_id;
ALTER TABLE IF EXISTS users DROP COLUMN IF EXISTS owner_address;
