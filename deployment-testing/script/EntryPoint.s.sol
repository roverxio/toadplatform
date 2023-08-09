// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/ImmutableCreate2Factory.sol";

contract EntryPointScript is Script {
    function setUp() public {}

    function run() public {
        // provider is anvil
        // deployerPrivateKey is used instead of hardhat signer address `from`
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);
        // create2 deployer exists on local fork of sepolia

        // gas cannot be used with new
        EntryPoint entryPoint = new EntryPoint{salt: bytes32(uint256(1))}();
        console.log("entrypoint addr", address(entryPoint));

        // the commented out code, used for deploying SimpleAccount and TestCounter is not implemented

        vm.stopBroadcast();
    }
}
