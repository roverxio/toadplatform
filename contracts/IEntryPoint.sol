// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.20.0;

import "./UserOperation.sol";

interface IEntryPoint {

    function handleOp(UserOperation memory op) external returns (uint256);
}