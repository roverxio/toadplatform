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

