// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
//Utils
import {Utilities} from "./Utilities.sol";

contract SimpleAccountTest is TestHelper {
    uint256 internal constant gasPrice = 1000000000;
    Utilities internal utils;

    function setUp() public {
        utils = new Utilities();
        accountOwner = utils.createAddress("simple_account_owner");
        deployEntryPoint(1201);
        createAccount(1202, 1203);
    }

    // Owner should be able to call transfer
    function test_TransferByOwner() public {
        // add balance to scw
        vm.deal(accountAddress, 3 ether);
        Account memory receiver = makeAccount("receiver");
        // set msg.sender to owner address
        vm.prank(accountOwner.addr);
        account.execute(receiver.addr, 1 ether, utils.defaultBytes());
        assertEq(utils.getBalance(accountAddress), 2 ether);
    }

    // Other account should not be able to call transfer
    function test_TransferByNonOwner(address receiver) public {
        // add balance to scw
        vm.deal(accountAddress, 3 ether);
        vm.expectRevert("account: not Owner or EntryPoint");
        account.execute(receiver, 1 ether, utils.defaultBytes());
    }

    // #validateUserOp
    // Should pay
    function test_Payment() public {
        vm.deal(accountAddress, 0.2 ether);

        UserOperation memory userOp = utils.defaultOp();
        userOp.sender = accountAddress;
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);

        uint256 expectedPay = gasPrice * (userOp.callGasLimit + userOp.verificationGasLimit);
        bytes32 userOpHash = utils.getUserOpHash(userOp, entryPointAddress, chainId);
        uint256 preBalance = utils.getBalance(accountAddress);

        // set msg.sender to entry point address
        vm.prank(entryPointAddress);
        account.validateUserOp{gas: gasPrice}(userOp, userOpHash, expectedPay);

        uint256 postBalance = utils.getBalance(accountAddress);
        assertEq(preBalance - postBalance, expectedPay);
    }

    // Should return NO_SIG_VALIDATION on wrong signature
    function test_WrongSignature() public {
        bytes32 zeroHash = 0x0000000000000000000000000000000000000000000000000000000000000000;
        UserOperation memory op = utils.defaultOp();
        op.sender = accountAddress;
        op.nonce = 1;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);

        // set msg.sender to entry point address
        vm.prank(entryPointAddress);
        uint256 deadline = account.validateUserOp(op, zeroHash, 0);

        assertEq(deadline, 1);
    }

    // SimpleAccountFactory
    // Sanity: check deployer
    function test_Deployer() public {
        Account memory newOwner = utils.createAddress("new_owner");
        address testAccount = simpleAccountFactory.getAddress(newOwner.addr, 123471);
        assertEq(utils.isContract(testAccount), false);
        simpleAccountFactory.createAccount(newOwner.addr, 123471);
        assertEq(utils.isContract(testAccount), true);
    }
}
