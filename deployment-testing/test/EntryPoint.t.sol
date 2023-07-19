// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/Counter.sol";

import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/ImmutableCreate2Factory.sol";
import "../src/interfaces/UserOperation.sol";

contract EntryPointTest is Test {
    EntryPoint internal entryPoint;
    SimpleAccountFactory internal simpleAccountFactory;
    SimpleAccount internal simpleAccount;

    Account accountOwner;
    uint256 constant globalUnstakeDelaySec = 2;
    uint256 constant paymasterStake = 2 ether;

    function deployEntryPoint(uint256 _salt) internal returns (EntryPoint) {
        return new EntryPoint{ salt: bytes32(_salt) }();
    }

    function createAccount() internal returns (SimpleAccount, SimpleAccountFactory, SimpleAccount) {
        vm.startBroadcast();
        SimpleAccountFactory accountFactory = new SimpleAccountFactory(entryPoint); //Tick
        SimpleAccount implementation = accountFactory.accountImplementation();
        accountFactory.createAccount(accountOwner.addr, 0);
        address accountAddress = accountFactory.getAddress(accountOwner.addr, 0);
        vm.stopBroadcast();
        SimpleAccount proxy = SimpleAccount(payable(accountAddress));
        return (implementation, accountFactory, proxy);
    }

    function signUserOp(UserOperation memory op, address ep, uint256 chainId)
        internal
        view
        returns (UserOperation memory)
    {
        bytes32 message = getUserOpHash(op, ep, chainId);
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(accountOwner.key, digest);
        op.signature = abi.encodePacked(r, s, v);
        return op;
    }

    function getUserOpHash(UserOperation memory op, address ep, uint256 chainId) internal pure returns (bytes32) {
        bytes32 userOpHash = keccak256(packUserOp(op, true));
        bytes memory encoded = abi.encode(userOpHash, ep, chainId);
        return bytes32(keccak256(encoded));
    }

    function packUserOp(UserOperation memory op, bool signature) internal pure returns (bytes memory) {
        if (signature) {
            return abi.encode(
                op.sender,
                op.nonce,
                keccak256(op.initCode),
                keccak256(op.callData),
                op.callGasLimit,
                op.verificationGasLimit,
                op.preVerificationGas,
                op.maxFeePerGas,
                op.maxPriorityFeePerGas,
                keccak256(op.paymasterAndData)
            );
        } else {
            return abi.encode(
                op.sender,
                op.nonce,
                op.initCode,
                op.callData,
                op.callGasLimit,
                op.verificationGasLimit,
                op.preVerificationGas,
                op.maxFeePerGas,
                op.maxPriorityFeePerGas,
                op.paymasterAndData,
                op.signature
            );
        }
    }

    //fillAndSign(accountAddress, accountOwner, entryPointAddress)
    function fillAndSign(uint256 chainId) internal view returns (UserOperation memory) {
        UserOperation memory op;
        op.sender = address(simpleAccount);
        op.nonce = 0;
        op.callData = "0x";
        op.initCode = "0x";
        op.callGasLimit = 200000;
        op.verificationGasLimit = 100000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 3000000000;
        op.paymasterAndData = "0x";
        op.signature = "0x";

        return signUserOp(op, address(entryPoint), chainId);
    }

    function setUp() public {
        uint256 chainId = vm.envOr("FOUNDRY_CHAIN_ID", uint256(31337));
        uint256 _entryPointSalt = 123456;
        entryPoint = deployEntryPoint(_entryPointSalt);
        accountOwner = makeAccount("accountOwner");
        (, simpleAccountFactory, simpleAccount) = createAccount();

        vm.prank(address(simpleAccount));
        vm.deal(address(simpleAccount), 1 ether);

        fillAndSign(chainId);
    }

    function test() public view {
        console.log("%s", "hello");
    }
}
