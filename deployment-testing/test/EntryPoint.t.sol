// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract EntryPointTest is TestHelper {
    UserOperation[] internal ops;

    function setUp() public {
        owner = createAddress("owner_entrypoint");
        deployEntryPoint(123441);
        createAccount(123442, 123443);
    }

    // Stake Management testing
    // Should deposit for transfer into EntryPoint
    function testDeposit(address signerAddress) public {
        entryPoint.depositTo{value: 1 ether}(signerAddress);

        assertEq(entryPoint.balanceOf(signerAddress), 1 ether);

        assertEq(entryPoint.getDepositInfo(signerAddress).deposit, 1 ether);
        assertEq(entryPoint.getDepositInfo(signerAddress).staked, false);
        assertEq(entryPoint.getDepositInfo(signerAddress).stake, 0);
        assertEq(entryPoint.getDepositInfo(signerAddress).unstakeDelaySec, 0);
        assertEq(entryPoint.getDepositInfo(signerAddress).withdrawTime, 0);
    }

    // Without stake
    // Should fail to stake without value
    function testNoStakeSpecified(uint32 unstakeDelaySec) public {
        if (unstakeDelaySec > 0) {
            vm.expectRevert(bytes("no stake specified"));
            entryPoint.addStake(unstakeDelaySec);
        }
    }

    // Should fail to stake without delay
    function testNoDelaySpecified() public {
        vm.expectRevert(bytes("must specify unstake delay"));
        entryPoint.addStake{value: 1 ether}(0);
    }

    // Should fail to unlock
    function testNoStakeUnlock() public {
        vm.expectRevert(bytes("not staked"));
        entryPoint.unlockStake();
    }

    // With stake of 2 eth
    // Should report "staked" state
    function testStakedState(address signerAddress) public {
        // add balance to temp address
        vm.deal(signerAddress, 3 ether);
        // set msg.sender to specific address
        vm.prank(signerAddress);
        entryPoint.addStake{value: 2 ether}(2);

        assertEq(entryPoint.getDepositInfo(signerAddress).deposit, 0);
        assertEq(entryPoint.getDepositInfo(signerAddress).staked, true);
        assertEq(entryPoint.getDepositInfo(signerAddress).stake, 2 ether);
        assertEq(entryPoint.getDepositInfo(signerAddress).unstakeDelaySec, 2);
        assertEq(entryPoint.getDepositInfo(signerAddress).withdrawTime, 0);
    }

    // With deposit
    // Should be able to withdraw
    function testWithdrawDeposit() public {
        account.addDeposit{value: 1 ether}();

        assertEq(getAccountBalance(), 0);
        assertEq(account.getDeposit(), 1 ether);

        vm.prank(owner.addr);
        account.withdrawDepositTo(payable(accountAddress), 1 ether);

        assertEq(getAccountBalance(), 1 ether);
        assertEq(account.getDeposit(), 0);
    }

    // 2d nonces
    // Should fail nonce with new key and seq!=0
    function test_FailNonce() public {
        Account memory beneficiary = createAddress("beneficiary");
        uint256 key = 1;
        uint256 keyShifed = key * 2 ** 64;

        (, address _accountAddress) = createAccountWithFactory(123422);
        vm.deal(_accountAddress, 1 ether);

        UserOperation memory op = _defaultOp;
        op.sender = _accountAddress;
        op.nonce = keyShifed + 1;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(abi.encodeWithSignature("FailedOp(uint256,string)", 0, "AA25 invalid account nonce"));
        entryPoint.handleOps(ops, payable(beneficiary.addr));
    }

    // With key=1, seq=1
    // should get next nonce value by getNonce
    function test_GetNonce() public {
        /**
         * Create beneficiary address
         * initialize key with 1
         * initialize key shifted
         * Create a SCW
         * Fund SCw
         * Fill and sign userop
         * Trigger handle ops
         * Get nonce
         * Validate nonce
         */
    }
}