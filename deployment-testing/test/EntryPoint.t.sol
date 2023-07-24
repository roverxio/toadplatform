// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract EntryPointTest is TestHelper {
    function setUp() public {
        createAddress("owner_entrypoint");
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

    //without paymaster (account pays in eth)
    //#handleOps
    UserOperation[] internal userOps;

    //should revert on signature failure
    function testRevertOnSignatureFailure() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        // assign a new owner to sign the User Op
        createAddress("new_owner");
        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 1 ether}(op.sender);
        userOps.push(op);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA24 signature error")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //account should pay for transaction
    function testPayForTransaction() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 10 ether}(op.sender);
        userOps.push(op);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }
}
