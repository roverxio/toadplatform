// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";

contract SimpleAccountTest is TestHelper {
    uint256 internal constant _GAS_PRICE = 1000000000;

    function setUp() public {
        _createAddress("owner");
        _deployEntryPoint(123456);
        _createAccount(123457, 123458);
    }

    // Owner should be able to call transfer
    function testTransferByOwner(address receiver) public {
        // add balance to scw
        vm.deal(_accountAddress, 3 ether);
        // set msg.sender to owner address
        vm.prank(_owner.addr);
        _account.execute(receiver, 1 ether, _DEFAULT_BYTES);
        assertEq(_getAccountBalance(), 2 ether);
    }

    // Other account should not be able to call transfer
    function testTransferByNonOwner(address receiver) public {
        // add balance to scw
        vm.deal(_accountAddress, 3 ether);
        vm.expectRevert(bytes('account: not Owner or EntryPoint'));
        _account.execute(receiver, 1 ether, _DEFAULT_BYTES);
    }

    // #validateUserOp
    // Should pay
    function testPayment() public {
        vm.deal(_accountAddress, 0.2 ether);

        UserOperation memory userOp = _fillAndSign(_chainId, 0);
        uint256 expectedPay = _GAS_PRICE * (userOp.callGasLimit + userOp.verificationGasLimit);
        bytes32 userOpHash = _getUserOpHash(userOp, _entryPointAddress, _chainId);
        uint256 preBalance = _getAccountBalance();

        // set msg.sender to entry point address
        vm.prank(_entryPointAddress);
        _account.validateUserOp{gas: _GAS_PRICE}(userOp, userOpHash, expectedPay);

        uint256 postBalance = _getAccountBalance();
        assertEq(preBalance - postBalance, expectedPay);
    }

    // Should return NO_SIG_VALIDATION on wrong signature
    function testWrongSignature() public {
        bytes32 zeroHash = 0x0000000000000000000000000000000000000000000000000000000000000000;
        UserOperation memory op = _fillAndSign(_chainId, 1);

        // set msg.sender to entry point address
        vm.prank(_entryPointAddress);
        uint256 deadline = _account.validateUserOp(op, zeroHash, 0);

        assertEq(deadline, 1);
    }
}
