-- Add up migration script here
DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT table_name
            FROM information_schema.tables
            WHERE table_name = 'token_metadata'
        ) THEN
            ALTER TABLE IF EXISTS supported_currencies RENAME TO token_metadata;
        END IF;
    END
$$;
