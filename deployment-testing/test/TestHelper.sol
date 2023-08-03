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

    function deployEntryPoint(uint256 _salt) internal {
        entryPoint = new EntryPoint{salt: bytes32(_salt)}();
        entryPointAddress = address(entryPoint);
    }

    function createAccount(uint256 _factorySalt, uint256 _accountSalt) internal {
        simpleAccountFactory = new SimpleAccountFactory{salt: bytes32(_factorySalt)}(entryPoint);
        implementation = simpleAccountFactory.accountImplementation();
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        accountAddress = payable(simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt));
        account = SimpleAccount(simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt));
    }

    function createAccountWithFactory(uint256 _accountSalt) internal returns (SimpleAccount, address) {
        simpleAccountFactory.createAccount(accountOwner.addr, _accountSalt);
        address _accountAddress = simpleAccountFactory.getAddress(accountOwner.addr, _accountSalt);
        return (SimpleAccount(payable(_accountAddress)), _accountAddress);
    }
}
