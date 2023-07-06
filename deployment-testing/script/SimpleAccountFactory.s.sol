// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";

contract SimpleAccountFactoryScript is Script {
    function setUp() public {

    }

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        address payable epAddress = payable(0xB9b7Aa6cE769Ce26867Dac898d1d37737176532E);
        EntryPoint ep = EntryPoint(epAddress);

        new SimpleAccountFactory{salt: bytes32(uint256(3))}(ep);

        vm.stopBroadcast();
    }
}
