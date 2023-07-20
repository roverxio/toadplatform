// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract TestHelper is Test {
    Account internal _owner;
    EntryPoint internal _entryPoint;
    SimpleAccount internal _account;
    SimpleAccount internal _implementation;
    SimpleAccountFactory internal _accountFactory;

    address internal _accountAddress;
    address internal _entryPointAddress;

    uint256 internal _chainId = vm.envOr("FOUNDRY_CHAIN_ID", uint256(31337));
    uint256 constant internal _GLOBAL_UNSTAKE_DELAY_SEC = 2;
    uint256 constant internal _PAYMASTER_STAKE = 2 ether;
    bytes constant internal _DEFAULT_BYTES = bytes("");

    UserOperation internal _defaultOp = UserOperation({
        sender: _accountAddress,
        nonce: 0,
        initCode: _DEFAULT_BYTES,
        callData: _DEFAULT_BYTES,
        callGasLimit: 200000,
        verificationGasLimit: 100000,
        preVerificationGas: 21000,
        maxFeePerGas: 3000000000,
        maxPriorityFeePerGas: 1,
        paymasterAndData: _DEFAULT_BYTES,
        signature: _DEFAULT_BYTES
        });

    function _createAddress(string memory _name) internal {
        _owner = makeAccount(_name);
    }

    function _deployEntryPoint(uint256 _salt) internal returns (EntryPoint) {
        _entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        _entryPointAddress = address(_entryPoint);
        return _entryPoint;
    }

    function _createAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        vm.startBroadcast();
        _accountFactory = new SimpleAccountFactory{salt: bytes32(_factorySalt)}(_entryPoint);
        _implementation = _accountFactory.accountImplementation();
        _accountFactory.createAccount(_owner.addr, _accountSalt);
        _accountAddress = _accountFactory.getAddress(_owner.addr, _accountSalt);
        vm.stopBroadcast();
        _account = SimpleAccount(payable(_accountAddress));
    }

    function _fillAndSign(uint256 _id, uint256 _nonce) internal view returns (UserOperation memory) {
        UserOperation memory op;
        op.sender = _accountAddress;
        op.nonce = _nonce;
        op.callData = _DEFAULT_BYTES;
        op.initCode = _DEFAULT_BYTES;
        op.callGasLimit = 200000;
        op.verificationGasLimit = 100000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 3000000000;
        op.paymasterAndData = _DEFAULT_BYTES;
        op.signature = _DEFAULT_BYTES;

        return _signUserOp(op, _entryPointAddress, _id);
    }

    function _signUserOp(UserOperation memory op, address _epAddress, uint256 _id)
    internal
    view
    returns (UserOperation memory)
    {
        bytes32 message = _getUserOpHash(op, _epAddress, _id);
        op.signature = _signMessage(message);
        return op;
    }

    function _getUserOpHash(UserOperation memory op, address _epAddress, uint256 _id) internal pure returns (bytes32) {
        bytes32 userOpHash = keccak256(_packUserOp(op, true));
        bytes memory encoded = abi.encode(userOpHash, _epAddress, _id);
        return bytes32(keccak256(encoded));
    }

    function _packUserOp(UserOperation memory op, bool signature) internal pure returns (bytes memory) {
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

    function _signMessage(bytes32 message) internal view returns (bytes memory) {
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(_owner.key, digest);
        return abi.encodePacked(r, s, v);
    }

    function _getEntryPointBalance() internal view returns (uint256) {
        return _entryPointAddress.balance;
    }

    function _getAccountBalance() internal view returns (uint256) {
        return _accountAddress.balance;
    }
}
