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
        sender: 0x0000000000000000000000000000000000000000,
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

    function deployEntryPoint(uint256 _salt) internal {
        entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        entryPointAddress = address(entryPoint);
    }

    function createAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        simpleAccountFactory = new SimpleAccountFactory{salt: bytes32(_factorySalt)}(entryPoint);
        implementation = simpleAccountFactory.accountImplementation();
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        accountAddress = payable(simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt));
        account = SimpleAccount(payable(accountAddress));
    }

    function createAccountWithFactory(uint256 _accountSalt) internal returns (SimpleAccount, address) {
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        address _accountAddress = simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt);
        return (SimpleAccount(payable(_accountAddress)), _accountAddress);
    }

    function createAccountWithFactory(uint256 _accountSalt, address _ownerAddress)
        internal
        returns (SimpleAccount, address)
    {
        simpleAccountFactory.createAccount(_ownerAddress, _accountSalt);
        address _accountAddress = simpleAccountFactory.getAddress(_ownerAddress, _accountSalt);
        return (SimpleAccount(payable(_accountAddress)), _accountAddress);
    }

    function getDataFromEncoding(bytes memory encoding) public pure returns (bytes4 sig, bytes memory data) {
        assembly {
            let totalLength := mload(encoding)
            let targetLength := sub(totalLength, 4)
            sig := mload(add(encoding, 0x20))
            data := mload(0x40)

            mstore(data, targetLength)
            mstore(0x40, add(data, add(0x20, targetLength)))
            mstore(add(data, 0x20), shl(0x20, mload(add(encoding, 0x20))))

            for { let i := 0x1C } lt(i, targetLength) { i := add(i, 0x20) } {
                mstore(add(add(data, 0x20), i), mload(add(add(encoding, 0x20), add(i, 0x04))))
            }
        }
    }

    function fillAggregatedOp(UserOperation[] memory _userOps, IAggregator _aggregator)
        public
        view
        returns (IEntryPoint.UserOpsPerAggregator memory ops)
    {
        ops.userOps = _userOps;
        ops.aggregator = _aggregator;
        ops.signature = _aggregator.aggregateSignatures(_userOps);
    }

    function getAccountInitCode(address owner, uint256 salt) public view returns (bytes memory initCode) {
        bytes memory initCallData = abi.encodeWithSignature("createAccount(address,uint256)", owner, salt);
        initCode = abi.encodePacked(address(simpleAccountFactory), initCallData);
    }

    function createOpWithPaymasterParams(
        address _accountAddr,
        address _paymasterAddr,
        uint48 _after,
        uint48 _until,
        Account memory owner
    ) public view returns (UserOperation memory op) {
        bytes memory timeRange = abi.encode(_after, _until);

        op = _defaultOp;
        op.sender = _accountAddr;
        op.paymasterAndData = abi.encodePacked(_paymasterAddr, timeRange);
        op = signUserOp(op, entryPointAddress, chainId, owner.key);
    }
}
