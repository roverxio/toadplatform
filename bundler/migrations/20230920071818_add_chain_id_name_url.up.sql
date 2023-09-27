-- Add up migration script here
ALTER TABLE IF EXISTS token_metadata ADD COLUMN IF NOT EXISTS chain_id int;
ALTER TABLE IF EXISTS token_metadata ADD COLUMN IF NOT EXISTS chain_name varchar;
ALTER TABLE IF EXISTS token_metadata ADD COLUMN IF NOT EXISTS token_image_url text;