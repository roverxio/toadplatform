# Sync pipelines
This serves as a simple data pipeline to read data from a `transactions` and `token_transfers` tables and push them as `credit` transactions to `user_transactions` table. `user_transactions` is a table maintained by the "bundler" service and keeps a track of all credits and debits happening for a user's wallet. `transactions` and `token_transfers` tables are created from the schema mentioned under [Ethereum ETL schema](https://github.com/blockchain-etl/ethereum-etl-postgres/tree/master/schema). [Ethereum ETL](https://github.com/blockchain-etl/ethereum-etl/) runs as a separate service and keeps updating the tables with new data using the stream option.</br>

## Syncing transactions and token_transfers
1. Create a virtual environment with python3.9
2. Use [ethereum-etl repo](https://github.com/blockchain-etl/ethereum-etl/) and [ethereum-etl postgres](https://github.com/blockchain-etl/ethereum-etl-postgres) to stream/sync transactions and token_transfers onto local postgres DB

## How to run
1. Clone the [repo](https://github.com/roverxio/toadplatform)
2. Copy `config-example.toml` into `Config.toml` and fill in the details
3. Set env variables using `.env.example`
4. Run `cargo sqlx migrate run --ignore-missing` to create index on token_transfers block_number
5. Run the script using `cargo run token_transfers` to sync ERC20 credit transactions
6. Run the script using `cargo run transactions` to sync credit transactions for the chain native currency

## How it works
1. The script reads the last synced block number from the `transaction_last_sync_block.txt` file for transactions and from `erc20_last_sync_block.txt` file for token_transfers. If the file is not present, it starts from the block-number mentioned in config file
2. It then queries the respective table for all the transactions that have happened since the last synced block number
3. It goes on to push the transactions to the `user_transactions` table. It also updates the respective file with the latest block number

## Formatting
This code is formatted using `rustfmt` formatted