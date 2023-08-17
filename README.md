# roverx-rpc

This is an implementation for ERC-4337 and test cases in foundry to test the contracts involved.

## Bundler
Bundler is a simple rust project that exposes REST APIs and acts as a relayer to [ERC4337](https://eips.ethereum.org/EIPS/eip-4337#rpc-methods-eth-namespace) (Account Abstraction via Entry Point Contract specification).
It uses actix for the REST APIs and an sqlite database for storing wallet deployment state and the salt. Rust version: `rustc 1.71.0 (8ede3aae2 2023-07-12)`

### Running the node locally
If you are running this project on localhost, you need to have a local node running with the contracts deployed. Here are the steps to set up your local node with the required contracts:
1. follow the instructions in the [foundry installation guide](https://book.getfoundry.sh/getting-started/installation) to set up foundry tool kit
2. navigate to the `contracts/` folder
3. run `bash foundry_setup.sh` to install all the contract dependencies
4. populate the `contracts/.env` with the following values
    ```
    RPC_URL=http://localhost:8545
   PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   CHAIN_ID=31337
   ENTRYPOINT_SALT=1
   SIMPLE_ACCOUNT_FACTORY_SALT=2
   TEST_ERC20_SALT=3
   VERIFYING_PAYMASTER_SALT=4
    ```
   The salt values can be changed if required. This will affect the addresses at which the contracts are deployed.
5. run `bash deploy_local.sh`. This should start a local anvil node and deploy the contracts

You can find the deployed contracts in the console logs. Copy the very first private-key from the logs and use it to populate the values of `WALLET_PRIVATE_KEY` and `VERIFYING_PAYMASTER_PRIVATE_KEY` in env variables. This key is the deployer for all the contracts.

<ins>NOTE</ins>: In case an instance of `anvil` is already running, run `pkill -f anvil` to stop the instance before `run deploy_local.sh`

### Running the bundler
1. navigate to `.env.example` and set the environment variables mentioned there (using the export command) (RUN_ENV can be one of "Development", "Production", "Staging")
2. if your RUN_ENV is "Development", set INFURA_KEY to an empty string. You will also need to create a copy of config/Staging.toml and rename it to config/Development.toml. Set the values in the config file as per your requirements.
3. run `bash db/setup_db.sh`
4. run `cargo run`

By default, the server uses "Development.toml" as the config file. If you want to use a different config file, set the `RUN_ENV` environment variable to the path of the config file. `RUN_ENV` can be one of:
1. Development
2. Production
3. Staging

The project does not come with a "Production.toml", but you can create one and use it. The config file should be in the same format as "Development.toml".

### Signing the userop
The server also uses a node service for signing called "signing-server". It needs to be running to be able to use this repo. You can find it under "roverx-rpc/signing-server". It is a simple node server that exposes a REST API for signing. You can find the instructions to run it in the README.md of the signing-server repo.


## Account Abstraction Deployment & Testing

A foundry project for deployment and testing of the Account Abstraction contracts

### Set Up
- git clone https://github.com/Club-Defy/roverx-rpc
- cd `deployment-testing`
- execute `dependencies.sh`
- Build `forge build`
- Testing `forge test`
- Deployment `forge script`

### Dependencies
All the dependent libraries are under `contracts/lib` directory, we have bash script (`contracts/foundry_setup.sh`) which will install all the dependencies required
- Forge Std (default)
- Openzeppelin Contracts (v4.9.3)
- Uniswap/v3-periphery (v1.3.0)
- Uniswap/v3-core (v1.0.0)

### Testing
All the test case are referred from [account-abstractions](https://github.com/eth-infinitism/account-abstraction) code from eth-infinitism. We have replicated most of the test cases in foundry, few cases could not be done as it includes RPC calls
