// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/EntryPoint.sol";

contract SimpleAccountFactoryScript is Script {
    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // entrypoint address needs to payable
        address payable entrypointAddress = payable(vm.envAddress("ENTRYPOINT_ADDRESS"));
        EntryPoint entrypoint = EntryPoint(entrypointAddress);

        SimpleAccountFactory factory = new SimpleAccountFactory{salt: bytes32(uint256(1))}(entrypoint);
        console.log("SimpleAccountFactory addr", address(factory));

        vm.stopBroadcast();
    }
}
