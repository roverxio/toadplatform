# Introduction
A simple rust project that exposes REST APIs and acts as a relayer to [ERC4337](https://eips.ethereum.org/EIPS/eip-4337#rpc-methods-eth-namespace) (Account Abstraction via Entry Point Contract specification).
It uses actix for the REST APIs and an sqlite database for storing wallet deployment state and the salt. Rust version: `rustc 1.71.0 (8ede3aae2 2023-07-12)`

## Pre-requisites
If you are running this project on localhost, you need to have a local node running with the contracts deployed. Also, the env variable `INFURA_KEY` will be set as an empty string. For testing, I used [eth-infinitism's account-abstraction](https://github.com/eth-infinitism/account-abstraction). You can use that as well. Here are the steps to deploy the contracts:
1. Clone the account-abstraction repo
2. Run `yarn install`
3. Go to [roverx-rpc's deplyer contracts](https://github.com/Club-Defy/roverx-rpc/tree/base/scw/provider/deployer) and copy
   1. 06_delpoy_TestERC20.ts (rename it to 4_deploy_TestERC20.ts)
   2. 13_deploy_VerifyingPaymaster.ts (rename it to 4_deploy_VerifyingPaymaster.ts)
4. Paste them in the account-abstraction repo under `deploy` folder
5. run `npx hardhat node`

This should start a hardhat node and deploy the contracts. You can find the deployed contracts in the console logs. Copy the very first private-key from the logs and use it to populate the values of "WALLET_PRIVATE_KEY" and "VERIFYING_PAYMASTER_PRIVATE_KEY" in env variables. This key is the deployer for all the contracts.

## How to run
1. Clone the repo
2. Navigate to ".env.example" and set the environment variables mentioned there (using the export command) (RUN_ENV can be one of "Development", "Production", "Staging")
3. If your RUN_ENV is "Development", set INFURA_KEY to an empty string. You will also need to create a copy of config/Staging.toml and rename it to config/Development.toml. Set the values in the config file as per your requirements.
4. run `bash db/setup_db.sh`
5. run `cargo run`

By deault, the server uses "Development.toml" as the config file. If you want to use a different config file, set the `RUN_ENV` environment variable to the path of the config file. `RUN_ENV` can be one of:
1. Development
2. Production
3. Staging

The project does not come with a "Production.toml", but you can create one and use it. The config file should be in the same format as "Development.toml".

## Signing
The server also uses a node service for signing called "signing-server". It needs to be running to be able to use this repo. You can find it under "roverx-rpc/signing-server". It is a simple node server that exposes a REST API for signing. You can find the instructions to run it in the README.md of the signing-server repo.
