// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/VerifyingPaymaster.sol";
import "../src/tests/TestERC20.sol";

contract DeployOnChain is Script {
    uint256 internal deployerPrivateKey;

    address payable internal entryPoint;
    address internal verifyingSigner;

    SimpleAccountFactory internal factory;
    VerifyingPaymaster internal paymaster;

    uint256 internal factorySalt;
    uint256 internal paymasterSalt;

    function setUp() public {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address owner = address(vm.addr(deployerPrivateKey));
        console.log("=SignerAddress==", owner);

        entryPoint = payable(vm.envAddress("ENTRYPOINT_ADDRESS"));
        console.log("=EntryPointAddress==", entryPoint);

        verifyingSigner = vm.envAddress("VERIFYING_SIGNER_ADDRESS");
        console.log("=VerifyingSigner==", verifyingSigner);

        factorySalt = vm.envOr("SIMPLE_ACCOUNT_FACTORY_SALT", uint256(2));
        paymasterSalt = vm.envOr("VERIFYING_PAYMASTER_SALT", uint256(4));
    }

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        factory = new SimpleAccountFactory{salt: bytes32(factorySalt)}(IEntryPoint(entryPoint));
        console.log("=SimpleAccountFactory addr==", address(factory));

        paymaster = new VerifyingPaymaster{salt: bytes32(paymasterSalt)}(IEntryPoint(entryPoint), verifyingSigner);
        console.log("=VerifyingPaymaster addr==", address(paymaster));

        vm.stopBroadcast();
    }
}
