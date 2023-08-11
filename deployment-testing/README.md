# Account Abstraction Deployment & Testing

A foundry project for deployment and testing of the Account Abstraction contracts

## Set Up
- git clone https://github.com/Club-Defy/roverx-rpc
- cd `deployment-testing`
- execute `dependencies.sh`
- Build `forge build`
- Testing `forge test`
- Deployment `forge script`

## Dependencies
All the dependent libraries are under `./lib` directory, we have bash script (`./foundry_setup.sh`) which will install all the dependencies required
- Forge Std (default)
- Openzeppelin Contracts (v4.9.3)
- Uniswap/v3-periphery (v1.3.0)
- Uniswap/v3-core (v1.0.0)

## Testing
All the test case are referred from [account-abstractions](https://github.com/eth-infinitism/account-abstraction) code from eth-infinitism. We have replicated most of the test cases in foundry, few cases could not be done as it includes RPC calls

## Deployment Scripts
Under the `deployment-testing/scrip` folder, there are scripts to deploy basic ERC-4337 contracts. 
These scripts however require a few environment variables to be set before they can be run.
- Create a `.env` file containing the following variables
```
RPC_URL=<rpc_url>
PRIVATE_KEY=<enter_your_private_key>
FOUNDRY_CHAIN_ID=<blockchain_id>
ENTRYPOINT_SALT=1
SIMPLE_ACCOUNT_FACTORY_SALT=2
VERIFYING_PAYMASTER_SALT=3
ENRTYPOINT_ADDRESS=<entrypoint_address>
```
- Start the local node. 
- Run `source <path_to_.env>` to use the env variables
- The scripts can run using the following command
`forge script <path_to_the_script>:ContractScript --broadcast --verify`

For further reading about the `script` command and its flags, you can refer to the [foundry docs](https://book.getfoundry.sh/reference/forge/forge-script)