-- Add up migration script here
DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT constraint_name
            FROM information_schema.table_constraints
            WHERE table_name='token_metadata' AND constraint_name='symbol_chain_unique_key'
        ) THEN
            ALTER TABLE token_metadata ADD CONSTRAINT symbol_chain_unique_key UNIQUE (symbol, chain);
        END IF;
    END
$$;
