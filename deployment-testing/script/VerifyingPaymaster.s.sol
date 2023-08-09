// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/VerifyingPaymaster.sol";
import "../src/EntryPoint.sol";

contract VerifyingPaymasterScript is Script {
    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address from = address(vm.addr(deployerPrivateKey));
        vm.startBroadcast(deployerPrivateKey);

        // entrypoint address needs to payable
        address payable entrypointAddress = payable(vm.envAddress("ENTRYPOINT_ADDRESS"));
        EntryPoint entrypoint = EntryPoint(entrypointAddress);

        VerifyingPaymaster verifyingpaymaster = new VerifyingPaymaster{salt: bytes32(uint256(1))}(entrypoint, from);
        console.log("VerifyingPaymaster addr", address(verifyingpaymaster));

        vm.stopBroadcast();
    }
}
