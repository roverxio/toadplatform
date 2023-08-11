# Introduction

This is a simple server that is used to sign UserOperation and return the "signature" to the calling service. It uses node version 16.20.1.

## How to run
1. Clone the repo
2. Run `npm ci`
3. Navigate to `.env.example` and set the environment variables mentioned there
4. Run `node index.js`

It exposes just one endpoint `/app/v1/sign_message` which accepts a POST request with the following body:
```json
{
    "user_operation": {
        "sender": "0x086d3d5e54c03fc5722edde88d3d6a8cf1c26f24",
        "nonce": 0,
        "init_code": "0x",
        "calldata": "0xb61d27f60000000000000",
        "call_gas_limit": 10000000,
        "verification_gas_limit": 10000000,
        "pre_verification_gas": 1000000000000000,
        "max_fee_per_gas": 5,
        "max_priority_fee_per_gas": 1000000000,
        "paymaster_and_data": "0x589c378e85c",
        "signature": "0xa0a343caa9a884c25ba8"
   },
    "entrypoint_address": "0x53D5E11475f4158dA8f0f3B46C69C717EE1b57b4",
    "chain_id": 31337
}
```

It assumes that the data is in a valid format and does no checks.