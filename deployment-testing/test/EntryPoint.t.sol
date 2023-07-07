// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract EntryPointTest is Test {
    EntryPoint public ep;
    SimpleAccountFactory public factory;
    SimpleAccount public wallet;
    address payable public walletAddress;

    function setUp() public {
        ep = new EntryPoint();
        factory = new SimpleAccountFactory(ep);
        wallet = factory.createAccount(0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC, 1);

        walletAddress = payable(wallet);

    }

    // Stake Management testing
    // Should deposit for transfer into EntryPoint
    function testDeposit(address signerAddress) public {
        ep.depositTo{value: 1 ether}(signerAddress);

        assertEq(ep.balanceOf(signerAddress), 1 ether);

        assertEq(ep.getDepositInfo(signerAddress).deposit, 1 ether);
        assertEq(ep.getDepositInfo(signerAddress).staked, false);
        assertEq(ep.getDepositInfo(signerAddress).stake, 0);
        assertEq(ep.getDepositInfo(signerAddress).unstakeDelaySec, 0);
        assertEq(ep.getDepositInfo(signerAddress).withdrawTime, 0);
    }

    // Without stake
    // Should fail to stake without value
    function testNoStakeSpecified(uint32 unstakeDelaySec) public {
        if (unstakeDelaySec > 0) {
            vm.expectRevert(bytes("no stake specified"));
            ep.addStake(unstakeDelaySec);
        }
    }

    // Should fail to stake without delay
    function testNoDelaySpecified() public {
        vm.expectRevert(bytes("must specify unstake delay"));
        ep.addStake{value: 1 ether}(0);
    }

    // Should fail to unlock
    function testNoStakeUnlock() public {
        vm.expectRevert(bytes("not staked"));
        ep.unlockStake();
    }

    // With stake of 2 eth
    // Should report "staked" state
    function testStakedState(address signerAddress) public {
        // set msg.sender to specific address
        vm.prank(signerAddress);
        // add balance to temp address
        vm.deal(signerAddress, 3 ether);
        ep.addStake{value: 2 ether}(2);

        assertEq(ep.getDepositInfo(signerAddress).deposit, 0);
        assertEq(ep.getDepositInfo(signerAddress).staked, true);
        assertEq(ep.getDepositInfo(signerAddress).stake, 2 ether);
        assertEq(ep.getDepositInfo(signerAddress).unstakeDelaySec, 2);
        assertEq(ep.getDepositInfo(signerAddress).withdrawTime, 0);
    }

    // With deposit
    // Should be able to withdraw
    function testWithdrawDeposit() public {
        wallet.addDeposit{value: 1 ether}();

        assertEq(walletAddress.balance, 0);
        assertEq(wallet.getDeposit(), 1 ether);

        vm.prank(0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC);
        wallet.withdrawDepositTo(walletAddress, 1 ether);

        assertEq(address(wallet).balance, 1 ether);
        assertEq(wallet.getDeposit(), 0);
    }
}