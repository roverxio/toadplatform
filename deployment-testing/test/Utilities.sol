// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract Utilities is Test {
    function createAccountOwner(string memory _name) public returns (Account memory) {
        return makeAccount(_name);
    }

    function createAccountWithEntryPoint(
        address accountOwner,
        EntryPoint entryPoint,
        SimpleAccountFactory _simpleAccountFactory
    ) public returns (SimpleAccount, SimpleAccountFactory) {
        SimpleAccountFactory simpleAccountFactory;

        if (!isContract(address(_simpleAccountFactory))) {
            simpleAccountFactory = new SimpleAccountFactory{salt: bytes32(0)}(entryPoint);
        } else {
            simpleAccountFactory = _simpleAccountFactory;
        }
        simpleAccountFactory.createAccount(accountOwner, 0);
        address accountAddress = simpleAccountFactory.getAddress(accountOwner, 0);
        SimpleAccount proxy = SimpleAccount(payable(accountAddress));

        return (proxy, simpleAccountFactory);
    }

    function createAccount(address accountOwner, SimpleAccountFactory _simpleAccountFactory)
        public
        view
        returns (SimpleAccount)
    {
        SimpleAccountFactory simpleAccountFactory;
        if (!isContract(address(_simpleAccountFactory))) {
            simpleAccountFactory = _simpleAccountFactory;
        }
        address accountAddress = simpleAccountFactory.getAddress(accountOwner, 0);
        SimpleAccount proxy = SimpleAccount(payable(accountAddress));

        return proxy;
    }

    function deployEntryPoint(uint256 _salt) public returns (EntryPoint) {
        EntryPoint entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        return entryPoint;
    }

    function fillAndSign(UserOperation memory op, Account memory accountOwner, EntryPoint entryPoint, uint256 chainId)
        public
        pure
        returns (UserOperation memory)
    {
        bytes32 userOpHash = keccak256(
            abi.encode(
                op.sender,
                op.nonce,
                op.initCode,
                op.callData,
                op.callGasLimit,
                op.verificationGasLimit,
                op.preVerificationGas,
                op.maxFeePerGas,
                op.maxPriorityFeePerGas,
                op.paymasterAndData
            )
        );

        bytes memory encoded = abi.encode(userOpHash, entryPoint, chainId);
        bytes32 message = bytes32(keccak256(encoded));
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(accountOwner.key, digest);
        op.signature = abi.encodePacked(r, s, v);

        return op;
    }

    function getAccountInitCode(address accountOwner, SimpleAccountFactory simpleAccountFactory, uint256 salt)
        public
        pure
        returns (bytes memory)
    {
        return hexConcat(
            abi.encodePacked(address(simpleAccountFactory)),
            abi.encodeWithSignature("createAccount(address,uint256)", accountOwner, salt)
        );
    }

    function getAccountAddress(address accountOwner, SimpleAccountFactory simpleAccountFactory, uint256 salt)
        public
        view
        returns (address)
    {
        return simpleAccountFactory.getAddress(accountOwner, salt);
    }

    function isContract(address _addr) internal view returns (bool) {
        uint256 size;
        assembly {
            size := extcodesize(_addr)
        }
        return size > 0;
    }

    function hexConcat(bytes memory _a, bytes memory _b) public pure returns (bytes memory) {
        bytes memory combined = new bytes(_a.length + _b.length);
        uint256 i;
        uint256 j;

        for (i = 0; i < _a.length; i++) {
            combined[j++] = _a[i];
        }

        for (i = 0; i < _b.length; i++) {
            combined[j++] = _b[i];
        }

        return combined;
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
