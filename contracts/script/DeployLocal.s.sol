// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/VerifyingPaymaster.sol";
import "../src/tests/TestERC20.sol";

contract DeployLocal is Script {
    uint256 internal deployerPrivateKey;
    address internal owner;

    EntryPoint internal entryPoint;
    SimpleAccountFactory internal factory;
    VerifyingPaymaster internal paymaster;
    TestERC20 internal erc20;

    function setUp() public {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        owner = address(vm.addr(deployerPrivateKey));
        console.log("=SignerAddress==", owner);
    }

    function run() public {
        vm.startBroadcast(deployerPrivateKey);
        
        entryPoint = new EntryPoint{salt: bytes32(uint256(1))}();
        console.log("=EntryPoint addr==", address(entryPoint));

        factory = new SimpleAccountFactory{salt: bytes32(uint256(2))}(entryPoint);
        console.log("=SimpleAccountFactory addr==", address(factory));

        erc20 = new TestERC20{salt: bytes32(uint256(3))}(18);
        console.log("=TestErC20 addr==", address(erc20));

        paymaster = new VerifyingPaymaster{salt: bytes32(uint256(4))}(entryPoint, owner);
        console.log("=VerifyingPaymaster addr==", address(paymaster));

        vm.stopBroadcast();
    }
}
