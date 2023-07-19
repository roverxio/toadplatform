// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";

contract SimpleAccountTest is TestHelper {
    uint256 internal constant gasPrice = 1000000000;

    function setUp() public {
        createAddress("owner");
        deployEntryPoint(123456);
        createAccount(123457, 123458);
    }

    // Owner should be able to call transfer
    function testTransferByOwner(address receiver) public {
        // add balance to scw
        vm.deal(accountAddress, 3 ether);
        // set msg.sender to owner address
        vm.prank(owner.addr);
        account.execute(receiver, 1 ether, defaultBytes);
        assertEq(accountAddress.balance, 2 ether);
    }

    // Other account should not be able to call transfer
    function testTransferByNonOwner(address receiver) public {
        // add balance to scw
        vm.deal(accountAddress, 3 ether);
        vm.expectRevert(bytes('account: not Owner or EntryPoint'));
        account.execute(receiver, 1 ether, defaultBytes);
    }

    // #validateUserOp
    // Should pay
    function testPayment() public {
        vm.deal(accountAddress, 0.2 ether);

        UserOperation memory userOp = fillAndSign(chainId, 0);
        uint256 expectedPay = gasPrice * (userOp.callGasLimit + userOp.verificationGasLimit);
        bytes32 userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);
        uint256 preBalance = accountAddress.balance;

        // set msg.sender to entry point address
        vm.prank(entryPointAddress);
        account.validateUserOp{gas: gasPrice}(userOp, userOpHash, expectedPay);

        uint256 postBalance = accountAddress.balance;
        assertEq(preBalance - postBalance, expectedPay);
    }

    // Should return NO_SIG_VALIDATION on wrong signature
    function testWrongSignature() public {
        bytes32 zeroHash = 0x0000000000000000000000000000000000000000000000000000000000000000;
        UserOperation memory op = fillAndSign(chainId, 1);

        // set msg.sender to entry point address
        vm.prank(entryPointAddress);
        uint256 deadline = account.validateUserOp(op, zeroHash, 0);

        assertEq(deadline, 1);
    }
}
