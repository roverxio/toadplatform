// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.20.0;

struct UserOperation {
    address sender;
    address receiver;
    uint256 nonce;
    uint256 amount;
}
