-- Add down migration script here
DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT table_name
            FROM information_schema.tables
            WHERE table_name = 'supported_currencies'
        ) THEN
            ALTER TABLE IF EXISTS token_metadata RENAME TO supported_currencies;
        END IF;
    END
$$;

