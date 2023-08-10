# Introduction
A simple rust project that exposes REST APIs and acts as a relayer to [ERC4337](https://eips.ethereum.org/EIPS/eip-4337#rpc-methods-eth-namespace) (Account Abstraction via Entry Point Contract specification).
It uses actix for the REST APIs and an sqlite database for storing wallet deployment state and the salt. Rust version: `rustc 1.71.0 (8ede3aae2 2023-07-12)`

## How to run
1. Clone the repo
2. Navigate to ".env.example" and set the environment variables mentioned there
3. run `bash db/setup_db.sh`
4. run `cargo run`

By deault, the server uses "Development.toml" as the config file. If you want to use a different config file, set the `RUN_ENV` environment variable to the path of the config file. `RUN_ENV` can be one of:
1. Development
2. Production
3. Staging

The project does not come with a "Production.toml", but you can create one and use it. The config file should be in the same format as "Development.toml".

## Signing
The server also uses a node service for signing called "signing-server". It needs to be running to be able to use this repo. You can find it under "roverx-rpc/signing-server". It is a simple node server that exposes a REST API for signing. You can find the instructions to run it in the README.md of the signing-server repo.
