#!/usr/bin/env bash
cd $(dirname "$0")
forge install OpenZeppelin/openzeppelin-contracts@v4.9.3 --no-git
forge install Uniswap/v3-periphery@v1.3.0 --no-git
forge install Uniswap/v3-core@v1.0.0 --no-git
