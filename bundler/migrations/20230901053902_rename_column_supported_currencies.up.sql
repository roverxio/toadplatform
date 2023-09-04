-- Add up migration script here
DO $$
    BEGIN
        IF EXISTS (
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'supported_currencies' AND column_name = 'currency'
        ) AND NOT EXISTS (
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'supported_currencies' AND column_name = 'symbol'
        ) THEN
            ALTER TABLE supported_currencies RENAME COLUMN currency TO symbol;
        END IF;
    END
$$;
