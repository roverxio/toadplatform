// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract EntryPointWithoutPaymasterTest is TestHelper {
    UserOperation[] internal userOps;
    address payable internal beneficiary;

    function setUp() public {
        createAddress("owner_entrypoint");
        deployEntryPoint(123441);
        createAccount(123442, 123443);
        beneficiary = payable(makeAddr("beneficiary"));
    }

    //#handleOps
    //should revert on signature failure
    function testRevertOnSignatureFailure() public {
        // assign a new owner to sign the User Op
        createAddress("new_owner");
        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 1 ether}(op.sender);
        userOps.push(op);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256('FailedOp(uint256,string)')), 
                0, 
                'AA24 signature error'));
        entryPoint.handleOps(userOps, beneficiary);
    }

    //account should pay for transaction
    function testPayForTransaction() public {
        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 10 ether}(op.sender);
        userOps.push(op);

        entryPoint.handleOps(userOps, beneficiary);
        assertEq(beneficiary.balance + entryPoint.getDepositInfo(op.sender).deposit, 10 ether);
    }

    //account should not pay if too low gas was set
    function testDonotPayForLowGas() public {
        UserOperation memory op = fillOp(0);
        op.callGasLimit = 9000000;
        op.verificationGasLimit = 9000000;
        op = signUserOp(op, address(entryPoint), chainId);
        
        entryPoint.depositTo{value: 10 ether}(op.sender);
        userOps.push(op);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256('FailedOp(uint256,string)')), 
                0, 
                'AA95 out of gas'));
        entryPoint.handleOps(userOps, beneficiary);
        assertEq(entryPoint.getDepositInfo(op.sender).deposit, 10 ether);
    }
}