# Toad Wallet System (based on ERC-4337)

## Background
ERC-4337 is the Ethereum community's first attempt to simply the wallet experience for users coming more familiar "web2 way of life". E-mail login and authorisation is a de-facto - if not, almost a standard across mobile/web applications.  Web3 "sign-in" and authorisation is based on private keys that has properties that make it impenetrable (until now), but are also known to be unwieldy, hard to manage making them cumbersome.    

# Components
## Toad Relay
 The Toad Relay is currently a component residing with the Bundler but will soon be extracted as a separate component in the near future. 
 Refer to the Bundler section for now.   
## Bundler
Bundler is a component being built as per the [ERC4337](https://eips.ethereum.org/EIPS/eip-4337#rpc-methods-eth-namespace) (Account Abstraction via Entry Point Contract specification). Bundler is a Rust based implementation that exposes REST APIs and also acts as a Relayer.
It uses Actix web framework to expose REST APIs.
MSRP: `rustc 1.71.0 (8ede3aae2 2023-07-12)`
## Contracts
### Smart Contract Wallet
Smart Contract Wallet (SCW) contract is deployed for every user that's onboarded on the Toad system. The current implementation of SCWs is basic and close to the eth-infinitism's reference implementation of the same.
### Smart Contract Factory 
Smart Contract Factory deploys SCWs for users. Current implementation is close to eth-infinitism's reference implementation.
### EntryPoint
eth-infinitism's reference implementation of the Entry Point spec for local testing.
### Paymaster
   #### Token Paymaster
   Reference implementation of Token Paymaster based on eth-infintism
   #### Verifying Paymaster 
   Reference implementation of Verifying  Paymaster based on eth-infintism


## Running the node locally
If you are running this project on localhost, you need to have a local node running with the contracts deployed. We use Foundry' Anvil to run a local node for development and testing:
1. Follow the instructions in the [foundry installation guide](https://book.getfoundry.sh/getting-started/installation) to set up foundry tool kit
2. Navigate to the `contracts/` folder
3. Run `bash foundry_setup.sh` to install all the contract dependencies
4. Populate the `contracts/.env` with the following values
    ```
    RPC_URL=http://localhost:8545
   PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   CHAIN_ID=31337
    ```
5. run `bash deploy_local.sh`. This should start a local anvil node and deploy the contracts

You can find the deployed contracts in the console logs. Copy the very first private-key from the logs and use it to populate the values of `WALLET_PRIVATE_KEY` and `VERIFYING_PAYMASTER_PRIVATE_KEY` in env variables. This key is the deployer for all the contracts.

<ins>NOTE</ins>: In case an instance of `anvil` is already running, run `pkill -f anvil` to stop the instance before `run deploy_local.sh`

### Running the Relay/Bundler
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

A Foundry project for deployment and testing of the ERC-4337 contracts

### Set Up
- git clone https://github.com/Club-Defy/roverx-rpc
- cd `contracts`
- execute `foundry_setup.sh`
- Build `forge build`
- Testing `forge test`
- To deploy the contracts execute: `deploy_local.sh`

### Smart Contract Dependencies
All the dependent libraries are under `contracts/lib` directory, we have bash script (`contracts/foundry_setup.sh`) which will install all the dependencies required
- Forge Std (default)
- Openzeppelin Contracts (v4.9.3)
- Uniswap/v3-periphery (v1.3.0)
- Uniswap/v3-core (v1.0.0)

### Testing
All the test case are referred from [account-abstractions](https://github.com/eth-infinitism/account-abstraction) code from eth-infinitism. Test cases from eth-infinitism's code base have been ported using forge (the tool of our choice). Some test cases that required RPC calls are yet to be ported. 
