# Toad Platform (based on ERC-4337)

## Background
ERC-4337 is the Ethereum community's first attempt to simplify the wallet experience for users coming from a more familiar "web2 way of life". E-mail login and authorisation is a de-facto - if not, almost a standard across mobile/web applications.  Web3 "sign-in" and authorisation is based on private keys that has properties that make it impenetrable (until now), but are also known to be unwieldy, hard to manage making them cumbersome. 

This code has been majorly influenced by [eth-infinitism](https://github.com/eth-infinitism)'s example implementation of ERC-4337.

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
Smart Contract Wallet (SCW) contract is deployed for every user that's onboarded on the Toad system
### Smart Contract Factory 
Smart Contract Factory deploys SCWs for users
### EntryPoint
eth-infinitism's reference implementation of the Entry Point spec for local testing.
### Paymaster
   #### Token Paymaster
   Reference implementation of Token Paymaster
   #### Verifying Paymaster 
   Reference implementation of Verifying Paymaster

<ins>NOTE</ins>: instructions have been tested on 13.5. support for more platforms will be added.
## Running the node locally
If you are running this project on localhost, you need to have a local node running with the contracts deployed. We use Foundry's Anvil to run a local node for development and testing:
1. Follow the instructions in the [foundry installation guide](https://book.getfoundry.sh/getting-started/installation) to set up foundry tool kit
2. Navigate to the `contracts/` folder
3. To install all the contract dependencies, run
   ```
    bash foundry_setup.sh
   ```
4. Populate the `contracts/.env` with the following values
    ```
    RPC_URL=http://localhost:8545<your_anvil_instance>
   PRIVATE_KEY=<pick_this_up_from_anvil>
   CHAIN_ID=31337
   ENTRYPOINT_SALT=1
   SIMPLE_ACCOUNT_FACTORY_SALT=2
   TEST_ERC20_SALT=3
   VERIFYING_PAYMASTER_SALT=4
    ```
   The salt values can be changed if required. This will affect the addresses at which the contracts are deployed.
5. To start a local anvil node and deploy the contracts, run
    ```
    bash script/deploy_local.sh
   ```

On successful deployment, you can find the deployed contract addresses, `signer/owner` address and a `key`. Use this info to populate the `bundler/.env` and `bundler/config/*.toml` files, as directed in the following section

<ins>NOTE</ins>: In case an instance of `anvil` is running and the contracts already deployed, stop the instance using
```
pkill -f anvil
```


### Running the Relay/Bundler
1. Set up a postgres database
2. Navigate to `bundler/.env.example` and set the environment variables mentioned there (using the export command)
   1. By default, the server uses "Staging.toml" as the config file. If you want to use a different config file, set the `RUN_ENV` environment variable to the path of the config file. `RUN_ENV` can be one of:
      1. Development
      2. Production
      3. Staging
   2. If your RUN_ENV is "Development", set INFURA_KEY to an empty string
   3. Use `key` obtained in the previous section to populate `WALLET_PRIVATE_KEY` and `VERIFYING_PAYMASTER_PRIVATE_KEY`
3. Create a copy of `config/Staging.toml` and rename it as `config/Development.toml`. Set the values in the config file as per your requirements.
   ```
   cp config/Staging.toml config/Development.toml
   ```
4. Run
    ```
   cargo run
   ```

<ins>NOTE</ins>: If there are any changes in the schema or the queries, run 
   ```
   cargo sqlx prepare --database-url $DATABASE_URL
   ```
   and add the generated files or the github workflow will fail. Files will be generated under `bundler/.sqlx`

The project does not come with a "Production.toml", but you can create one and use it. The config file should be in the same format as "Development.toml".

## Account Abstraction Deployment & Testing

A Foundry project for deployment and testing of the ERC-4337 contracts

### Set Up
- git clone https://github.com/Club-Defy/roverx-rpc
- cd `contracts`
- execute `foundry_setup.sh`
- git clone https://github.com/roverxio/toad
- cd `deployment-testing`
- execute `dependencies.sh`
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
