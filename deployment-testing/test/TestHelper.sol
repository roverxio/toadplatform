// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract TestHelper is Test {
    Account internal accountOwner;
    EntryPoint internal entryPoint;
    SimpleAccount internal account;
    SimpleAccount internal implementation;
    SimpleAccountFactory internal simpleAccountFactory;

    address internal accountAddress;
    address internal entryPointAddress;

    uint256 internal chainId = vm.envOr("FOUNDRY_CHAIN_ID", uint256(31337));
    uint256 internal constant globalUnstakeDelaySec = 2;
    uint256 internal constant paymasterStake = 2 ether;
    bytes internal constant defaultBytes = bytes("");

    UserOperation internal _defaultOp = UserOperation({
        sender: accountAddress,
        nonce: 0,
        initCode: defaultBytes,
        callData: defaultBytes,
        callGasLimit: 200000,
        verificationGasLimit: 100000,
        preVerificationGas: 21000,
        maxFeePerGas: 3000000000,
        maxPriorityFeePerGas: 1,
        paymasterAndData: defaultBytes,
        signature: defaultBytes
    });

    function createAddress(string memory _name) internal returns (Account memory) {
        return makeAccount(_name);
    }

    function deployEntryPoint(uint256 _salt) internal {
        entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        entryPointAddress = address(entryPoint);
    }

    function createAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        simpleAccountFactory = new SimpleAccountFactory{salt: bytes32(_factorySalt)}(entryPoint);
        implementation = simpleAccountFactory.accountImplementation();
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        accountAddress = simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt);
        account = SimpleAccount(payable(accountAddress));
    }

    function createAccountWithFactory(uint256 _accountSalt) internal returns (SimpleAccount, address) {
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        address _accountAddress = simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt);
        return (SimpleAccount(payable(_accountAddress)), _accountAddress);
    }

    function fillAndSign(uint256 _chainId, uint256 _nonce) internal view returns (UserOperation memory) {
        UserOperation memory op;
        op.sender = accountAddress;
        op.nonce = _nonce;
        op.callData = defaultBytes;
        op.initCode = defaultBytes;
        op.callGasLimit = 200000;
        op.verificationGasLimit = 100000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 3000000000;
        op.paymasterAndData = defaultBytes;
        op.signature = defaultBytes;

        return signUserOp(op, entryPointAddress, _chainId);
    }

    function signUserOp(UserOperation memory op, address _entryPoint, uint256 _chainId)
        internal
        view
        returns (UserOperation memory)
    {
        bytes32 message = getUserOpHash(op, _entryPoint, _chainId);
        op.signature = signMessage(message);
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
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(accountOwner.key, digest);
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
}
