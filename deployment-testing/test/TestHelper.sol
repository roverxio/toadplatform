// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/test/TestAggregatedAccount.sol";
import "../src/test/TestAggregatedAccountFactory.sol";
import "../src/test/TestSignatureAggregator.sol";

contract TestHelper is Test {
    Account internal owner;
    EntryPoint internal entryPoint;
    SimpleAccount internal account;
    SimpleAccount internal implementation;
    SimpleAccountFactory internal accountFactory;
    address internal nullAddress = 0x0000000000000000000000000000000000000000;

    TestSignatureAggregator internal aggregator;
    TestAggregatedAccount internal aggrAccount;
    TestAggregatedAccount internal aggrAccountImplementation;
    TestAggregatedAccountFactory internal aggrAccountFactory;

    address internal accountAddress;
    address internal entryPointAddress;
    address internal aggregatorAddress;
    address internal aggrAccountAddress;

    uint256 internal chainId = vm.envOr("FOUNDRY_CHAIN_ID", uint256(31337));
    uint256 internal constant globalUnstakeDelaySec = 2;
    uint256 internal constant paymasterStake = 2 ether;
    bytes internal constant defaultBytes = bytes("");

    function createAddress(string memory _name) internal returns (Account memory) {
        return makeAccount(_name);
    }

    function deployEntryPoint(uint256 _salt) internal returns (EntryPoint) {
        entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        entryPointAddress = address(entryPoint);
        return entryPoint;
    }

    function createAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        vm.startBroadcast();
        accountFactory = new SimpleAccountFactory{salt: bytes32(_factorySalt)}(entryPoint);
        implementation = accountFactory.accountImplementation();
        accountFactory.createAccount(owner.addr, _accountSalt);
        accountAddress = accountFactory.getAddress(owner.addr, _accountSalt);
        vm.stopBroadcast();
        account = SimpleAccount(payable(accountAddress));
    }

    function createAggregatedAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        vm.startBroadcast();
        aggregator = new TestSignatureAggregator();
        aggregatorAddress = address(aggregator);
        aggrAccountFactory =
            new TestAggregatedAccountFactory{salt: bytes32(_factorySalt)}(entryPoint, aggregatorAddress);
        aggrAccountImplementation = aggrAccountFactory.accountImplementation();
        aggrAccountFactory.createAccount(owner.addr, _accountSalt);
        aggrAccountAddress = aggrAccountFactory.getAddress(owner.addr, _accountSalt);
        vm.stopBroadcast();
        aggrAccount = TestAggregatedAccount(payable(aggrAccountAddress));
    }

    function fillAndSign(uint256 _chainId, uint256 _nonce) internal view returns (UserOperation memory) {
        UserOperation memory op = fillOp(_nonce);
        return signUserOp(op, entryPointAddress, _chainId);
    }

    function fillOp(uint256 _nonce) internal view returns (UserOperation memory) {
        UserOperation memory op;
        op.sender = accountAddress;
        op.nonce = _nonce;
        op.callData = defaultBytes;
        op.initCode = defaultBytes;
        op.callGasLimit = 200000;
        op.verificationGasLimit = 100000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 300000;
        op.maxPriorityFeePerGas = 300000;
        op.paymasterAndData = defaultBytes;
        op.signature = defaultBytes;

        return op;
    }

    function signUserOp(UserOperation memory op, address _entryPoint, uint256 _chainId)
        internal
        view
        returns (UserOperation memory)
    {
        return signUserOp(op, _entryPoint, _chainId, owner.key);
    }

    function signUserOp(UserOperation memory op, address _entryPoint, uint256 _chainId, uint256 key)
        internal
        pure
        returns (UserOperation memory)
    {
        bytes32 message = getUserOpHash(op, _entryPoint, _chainId);
        op.signature = signMessage(message, key);
        return op;
    }

    function getUserOpHash(UserOperation memory op, address _entryPoint, uint256 _chainId)
        internal
        pure
        returns (bytes32)
    {
        bytes32 userOpHash = keccak256(packUserOp(op, true));
        bytes memory encoded = abi.encode(userOpHash, _entryPoint, _chainId);
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

    function signMessage(bytes32 message) internal view returns (bytes memory) {
        return signMessage(message, owner.key);
    }

    function signMessage(bytes32 message, uint256 key) internal pure returns (bytes memory) {
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(key, digest);
        return abi.encodePacked(r, s, v);
    }

    function getEntryPointBalance() internal view returns (uint256) {
        return entryPointAddress.balance;
    }

    function getAccountBalance() internal view returns (uint256) {
        return accountAddress.balance;
    }

    function isDeployed(address addr) public view returns (bool) {
        uint256 size;
        assembly {
            size := extcodesize(addr)
        }
        return size > 0;
    }

    function getDataFromEncoding(bytes memory encoding) public pure returns (bytes memory data) {
        assembly {
            let totalLength := mload(encoding)
            let targetLength := sub(totalLength, 4)
            data := mload(0x40)

            mstore(data, targetLength)
            mstore(0x40, add(data, add(0x20, targetLength)))
            mstore(add(data, 0x20), shl(0x20, mload(add(encoding, 0x20))))

            for { let i := 0x1C } lt(i, targetLength) { i := add(i, 0x20) } {
                mstore(add(add(data, 0x20), i), mload(add(add(encoding, 0x20), add(i, 0x04))))
            }
        }
    }
}
