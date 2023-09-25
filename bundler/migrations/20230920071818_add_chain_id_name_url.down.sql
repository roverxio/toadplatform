-- Add down migration script here
ALTER TABLE IF EXISTS token_metadata DROP COLUMN IF EXISTS chain_id;
ALTER TABLE IF EXISTS token_metadata DROP COLUMN IF EXISTS chain_name;
ALTER TABLE IF EXISTS token_metadata DROP COLUMN IF EXISTS token_image_url;