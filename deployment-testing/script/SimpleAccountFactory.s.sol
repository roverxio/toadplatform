// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/EntryPoint.sol";
import "./Utilities.sol";

contract SimpleAccountFactoryScript is Script {
    address internal entryPointAddress;
    Utilities utils = new Utilities();

    function setUp() public {
        entryPointAddress = utils.entryPointSetUp();
    }

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        uint256 salt = vm.envUint("SALT");
        vm.startBroadcast(deployerPrivateKey);

        // entrypoint address needs to payable
        EntryPoint entryPoint = EntryPoint(payable(entryPointAddress));

        SimpleAccountFactory factory = new SimpleAccountFactory{salt: bytes32(uint256(salt))}(entryPoint);
        console.log("SimpleAccountFactory addr", address(factory));

        vm.stopBroadcast();
    }
}
