// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/VerifyingPaymaster.sol";
import "../src/EntryPoint.sol";
import "./Utilities.sol";

contract VerifyingPaymasterScript is Script {
    address internal entryPointAddress;
    Utilities utils = new Utilities();

    function setUp() public {
        entryPointAddress = utils.entryPointSetUp();
    }

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address from = address(vm.addr(deployerPrivateKey));
        uint256 salt = vm.envUint("SALT");
        vm.startBroadcast(deployerPrivateKey);

        // entrypoint address needs to payable
        EntryPoint entrypoint = EntryPoint(payable(entryPointAddress));

        VerifyingPaymaster verifyingPaymaster = new VerifyingPaymaster{salt: bytes32(uint256(salt))}(entrypoint, from);
        console.log("VerifyingPaymaster addr", address(verifyingPaymaster));

        vm.stopBroadcast();
    }
}
