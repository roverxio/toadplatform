// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/EntryPoint.sol";
import "./Utilities.sol";

contract SimpleAccountFactoryScript is Script {
    address internal entryPointAddress;
    EntryPoint internal entryPoint;
    uint256 internal deployerPrivateKey;
    Utilities internal utils = new Utilities();

    function setUp() public {
        utils = new Utilities();

        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        entryPointAddress = vm.envOr("ENTRYPOINT_ADDRESS", address(0));
        if (!utils.isContract(entryPointAddress) || (entryPointAddress == address(0))) {
            uint256 epSalt = vm.envOr("ENTRYPOINT_SALT", uint256(123));
            entryPoint = new EntryPoint{salt: bytes32(uint256(epSalt))}();
            entryPointAddress = address(entryPoint);
        } else {
            // entrypoint address needs to payable
            entryPoint = EntryPoint(payable(entryPointAddress));
        }
    }

    function run() public {
        uint256 salt = vm.envUint("SIMPLE_ACCOUNT_FACTORY_SALT");
        vm.startBroadcast(deployerPrivateKey);

        SimpleAccountFactory factory = new SimpleAccountFactory{salt: bytes32(uint256(salt))}(entryPoint);
        console.log("SimpleAccountFactory addr", address(factory));

        vm.stopBroadcast();
    }
}
