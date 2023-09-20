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

    uint256 internal entryPointSalt;
    uint256 internal factorySalt;
    uint256 internal erc20Salt;
    uint256 internal paymasterSalt;

    function setUp() public {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        owner = address(vm.addr(deployerPrivateKey));
        console.log("=Signer addr==", owner);

        entryPointSalt = vm.envOr("ENTRYPOINT_SALT", uint256(1));
        factorySalt = vm.envOr("SIMPLE_ACCOUNT_FACTORY_SALT", uint256(2));
        erc20Salt = vm.envOr("TEST_ERC20_SALT", uint256(3));
        paymasterSalt = vm.envOr("VERIFYING_PAYMASTER_SALT", uint256(4));
    }

    function run() public {
        vm.startBroadcast(deployerPrivateKey);
        
        entryPoint = new EntryPoint{salt: bytes32(entryPointSalt)}();
        console.log("=EntryPoint addr==", address(entryPoint));

        factory = new SimpleAccountFactory{salt: bytes32(factorySalt)}(entryPoint);
        console.log("=SimpleAccountFactory addr==", address(factory));

        erc20 = new TestERC20{salt: bytes32(erc20Salt)}(18);
        console.log("=TestErC20 addr==", address(erc20));

        paymaster = new VerifyingPaymaster{salt: bytes32(paymasterSalt)}(entryPoint, owner);
        console.log("=VerifyingPaymaster addr==", address(paymaster));

        vm.stopBroadcast();
    }
}
