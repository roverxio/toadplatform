-- Add down migration script here
DO $$
    BEGIN
        IF EXISTS (
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'supported_currencies' AND column_name = 'symbol'
        ) AND NOT EXISTS (
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'supported_currencies' AND column_name = 'currency'
        ) THEN
            ALTER TABLE supported_currencies RENAME COLUMN symbol TO currency;
        END IF;
    END
$$;

