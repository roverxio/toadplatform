-- Add up migration script here
CREATE INDEX token_transfers_block_number_index ON token_transfers(block_number DESC);