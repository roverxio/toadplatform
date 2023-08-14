// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/VerifyingPaymaster.sol";
import "../src/tests/TestERC20.sol";

contract ToadContractsScript is Script {
    uint256 internal deployerPrivateKey;
    address internal owner;

    uint256 internal epSalt;
    uint256 internal factorySalt;
    uint256 internal paymasterSalt;
    uint256 internal erc20Salt;

    EntryPoint internal entryPoint;
    SimpleAccountFactory internal factory;
    VerifyingPaymaster internal paymaster;
    TestERC20 internal erc20;

    function setUp() public {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        owner = address(vm.addr(deployerPrivateKey));
        console.log("=SignerAddress==", owner);

        epSalt = vm.envUint("ENTRYPOINT_SALT");
        factorySalt = vm.envUint("SIMPLE_ACCOUNT_FACTORY_SALT");
        paymasterSalt = vm.envUint("VERIFYING_PAYMASTER_SALT");
        erc20Salt = vm.envUint("TEST_ERC20_SALT");
    }

    function run() public {
        vm.startBroadcast(deployerPrivateKey);
        
        entryPoint = new EntryPoint{salt: bytes32(epSalt)}();
        console.log("=EntryPoint addr==", address(entryPoint));

        factory = new SimpleAccountFactory{salt: bytes32(uint256(factorySalt))}(entryPoint);
        console.log("=SimpleAccountFactory addr==", address(factory));

        erc20 = new TestERC20{salt: bytes32(erc20Salt)}(18);
        console.log("=TestErC20 addr==", address(erc20));

        paymaster = new VerifyingPaymaster{salt: bytes32(uint256(paymasterSalt))}(entryPoint, owner);
        console.log("=VerifyingPaymaster addr==", address(paymaster));

        vm.stopBroadcast();
    }
}
