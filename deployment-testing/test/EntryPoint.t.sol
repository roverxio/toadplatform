// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";

contract EntryPointTest is TestHelper {
    uint256 internal _accountSalt;

    function setUp() public {
        _accountSalt = 123433;
        owner = createAddress("owner_entrypoint");
        deployEntryPoint(123441);
        createAccount(123442, _accountSalt);
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
        owner = createAddress("new_owner");
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

    //if account has a deposit, it should use it to pay
    function testPayFromDeposit() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));
        UserOperation memory op = fillAndSign(chainId, 0);
        userOps.push(op);

        entryPoint.depositTo{value: 10 ether}(op.sender);
        (bool _sent,) = op.sender.call{value: 10 ether}("");
        require(_sent, "Could not pay op.sender");

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
        assertEq(op.sender.balance, 10 ether);
        assertEq(entryPoint.getDepositInfo(op.sender).deposit + actualGasCost, 10 ether);
    }

    //should pay for reverted tx
    function testPayForRevertedTx() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        UserOperation memory op = fillOp(0);
        op.callData = "0xdeadface";
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);
        entryPoint.depositTo{value: 10 ether}(op.sender);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (, bool success, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertFalse(success);
        assertEq(beneficiary.balance, actualGasCost);
    }

    //#handleOp (single)
    function testSingleOp() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));
        UserOperation[] memory userOperations = new UserOperation[](1);

        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 10 ether}(op.sender);
        userOperations[0] = op;

        vm.recordLogs();
        entryPoint.handleOps(userOperations, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //should fail to call recursively into handleOps
    function testRecursiveCallToHandleOps() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        UserOperation memory dummyOp = fillAndSign(chainId, 0);
        UserOperation[] memory dummyUserOperations = new UserOperation[](1);
        dummyUserOperations[0] = dummyOp;

        UserOperation memory op = fillOp(0);
        bytes memory handleOpsCalldata = abi.encodeWithSignature(
            "handleOps((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes)[],address)",
            dummyUserOperations,
            beneficiary
        );
        bytes memory executeCalldata =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(entryPoint), 0, handleOpsCalldata);

        op.callData = executeCalldata;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);
        entryPoint.depositTo{value: 10 ether}(op.sender);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (, bytes memory revertReason) = abi.decode(entries[1].data, (uint256, bytes));
        assertEq(
            revertReason, abi.encodeWithSelector(bytes4(keccak256("Error(string)")), "ReentrancyGuard: reentrant call")
        );
    }

    //should report failure on insufficient verificationGas after creation
    function testInsufficientVerificationGas() public {
        UserOperation memory op0 = fillOp(0);
        op0.verificationGasLimit = 100e5;
        op0 = signUserOp(op0, entryPointAddress, chainId);
        entryPoint.depositTo{value: 1 ether}(op0.sender);

        vm.expectRevert();
        try entryPoint.simulateValidation(op0) {}
        catch (bytes memory revertReason) {
            bytes4 reason;
            assembly {
                reason := mload(add(revertReason, 32))
            }
            assertEq(
                bytes4(
                    keccak256(
                        "ValidationResult((uint256,uint256,bool,uint48,uint48,bytes),(uint256,uint256),(uint256,uint256),(uint256,uint256))"
                    )
                ),
                reason
            );
        }

        UserOperation memory op1 = fillOp(0);
        op1.verificationGasLimit = 10000;
        op1 = signUserOp(op1, entryPointAddress, chainId);
        entryPoint.depositTo{value: 1 ether}(op1.sender);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA23 reverted (or OOG)")
        );
        entryPoint.simulateValidation(op1);
    }

    //create account
    //should reject create if sender address is wrong
    function testCreateWrongSenderAddress() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, _accountSalt));
        op.sender = 0x1111111111111111111111111111111111111111;
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA14 initCode must return sender")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //should reject create if account not funded
    function testRejectCreateIfNotFunded() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));
        uint256 salt = 123;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.verificationGasLimit = 100000000;
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        assertEq(entryPoint.getDepositInfo(op.sender).deposit, 0);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA21 didn't pay prefund")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //should succeed to create account after prefund
    function testCreateIfFunded() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));
        uint256 salt = 123;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.verificationGasLimit = 100000000;
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        bool isFunded = entryPoint.getDepositInfo(op.sender).deposit > 0;
        assertEq(isFunded, true);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries[3].topics[0], keccak256("AccountDeployed(bytes32,address,address,address)"));
    }

    //should reject if account already created
    function testSenderAlreadyCreated() public {
        address payable beneficiary = payable(makeAddr("beneficiary"));

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, _accountSalt));
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA10 sender already constructed")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }
}
