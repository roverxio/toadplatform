// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EntryPoint.sol";

contract EntryPointTest is Test {
    EntryPoint public ep;

    function setUp() public {
        ep = new EntryPoint();
    }

    // Stake Management testing
    // should deposit for transfer into EntryPoint
    function testDeposit(address signerAddress) public {
        ep.depositTo{value: 1 ether}(signerAddress);

        assertEq(ep.balanceOf(signerAddress), 1 ether);

        assertEq(ep.getDepositInfo(signerAddress).deposit, 1 ether);
        assertEq(ep.getDepositInfo(signerAddress).staked, false);
        assertEq(ep.getDepositInfo(signerAddress).stake, 0);
        assertEq(ep.getDepositInfo(signerAddress).unstakeDelaySec, 0);
        assertEq(ep.getDepositInfo(signerAddress).withdrawTime, 0);
    }
}