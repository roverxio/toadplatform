# Sync pipelines
This serves as a simple data pipeline to read data from a "transactions" and "token_transfers" tables and push them as "credit" transactions to "user_transactions" table. "user_transactions" is a table maintained by the "bundler" service and keeps a track of all credits and debits happening for a user's wallet. "transactions" and "token_transfers" tables are created from the schema mentioned under [Ethereum ETL schema](https://github.com/blockchain-etl/ethereum-etl-postgres/tree/master/schema). [Ethereum ETL](https://github.com/blockchain-etl/ethereum-etl/) runs as a separate service and keeps updating the tables with new data using the stream option.</br>

## How to run
1. Clone the repo
2. Install the dependencies using `pip install -r requirements.txt`
3. Create a copy of config.yaml.example as config_development.yaml and fill in the details. The file to be used comes from env variable "ENV", with the default as "development". The config file can be one of:
   1. config_development.yaml - for local development
   2. config_staging.yaml - for staging
   3. config_production.yaml - for production</br>To set the env variable, run `export ENV=development` in the terminal.
4. Run the script using `python main.py token_transfers` to sync ERC20 credit transactions
5. Run the script using `python main.py transactions` to sync credit transactions for the chain native currency
6. Ideally, the script should be run as a cron job to sync the data periodically. This is written to be run as a cron job every minute

## How it works
1. The script reads the last synced block number from the "transaction_last_synced_blocktimestamp" file for transactions and from "erc20_last_synced_blocktimestamp" file for token_transfers. If the file is not present, it starts from the time mentioned in config file.
2. It then queries the respective table for all the transactions that have happened since the last synced block time.
3. It goes on to push the transactions to the "user_transactions" table. It also updates the respective file with the latest block number.
4. The destination database is sqlite as of now. When we move to a different database, we will have to change a few things wrt the insert queries and the way we connect to the database.

## Formatting
The code is formatted using [Black formatter](https://github.com/psf/black). To format the code, run `black .` in the terminal before pushing any change.