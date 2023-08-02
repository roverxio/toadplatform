// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/test/TestCounter.sol";
import "../src/test/TestPaymasterAcceptAll.sol";
import "../src/test/TestExpiryAccount.sol";
import "../src/test/TestExpirePaymaster.sol";

contract EntryPointTest is TestHelper {
    uint256 internal _accountSalt;
    BasePaymaster internal paymasterAcceptAll;
    TestExpirePaymaster internal expirePaymaster;
    TestCounter internal counter;
    address payable internal beneficiary;
    UserOperation[] internal userOps;

    event SignatureAggregatorChanged(address indexed aggregator);
    event UserOperationEvent(
        bytes32 indexed userOpHash,
        address indexed sender,
        address indexed paymaster,
        uint256 nonce,
        bool success,
        uint256 actualGasCost,
        uint256 actualGasUsed
    );
    event AccountDeployed(bytes32 indexed userOpHash, address indexed sender, address factory, address paymaster);

    function setUp() public {
        _accountSalt = 123433;
        owner = createAddress("owner_entrypoint");
        deployEntryPoint(123441);
        createAccount(123442, _accountSalt);
        createAggregatedAccount(123456, 123456);

        paymasterAcceptAll = new TestPaymasterAcceptAll(entryPoint);
        expirePaymaster = new TestExpirePaymaster(entryPoint);
        beneficiary = payable(makeAddr("beneficiary"));
        counter = new TestCounter();
    }

    // Stake Management testing
    // Should deposit for transfer into EntryPoint
    function test_Deposit(address signerAddress) public {
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
    function test_NoStakeSpecified(uint32 unstakeDelaySec) public {
        if (unstakeDelaySec > 0) {
            vm.expectRevert(bytes("no stake specified"));
            entryPoint.addStake(unstakeDelaySec);
        }
    }

    // Should fail to stake without delay
    function test_NoDelaySpecified() public {
        vm.expectRevert(bytes("must specify unstake delay"));
        entryPoint.addStake{value: 1 ether}(0);
    }

    // Should fail to unlock
    function test_NoStakeUnlock() public {
        vm.expectRevert(bytes("not staked"));
        entryPoint.unlockStake();
    }

    // With stake of 2 eth
    // Should report "staked" state
    function test_StakedState(address signerAddress) public {
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
    function test_WithdrawDeposit() public {
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
    //should revert on signature failure
    function test_RevertOnSignatureFailure() public {
        // assign a new owner to sign the User Op
        Account memory new_owner = createAddress("new_owner");
        UserOperation memory op = fillOp(0);
        op = signUserOp(op, entryPointAddress, chainId, new_owner.key);
        entryPoint.depositTo{value: 1 ether}(op.sender);
        userOps.push(op);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA24 signature error")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //account should pay for transaction
    function test_PayForTransaction() public {
        UserOperation memory op = fillOp(0);
        bytes memory counterCallData = abi.encodeWithSignature("count()");
        op.callData = abi.encodeCall(account.execute, (address(counter), 0, counterCallData));
        op = signUserOp(op, entryPointAddress, chainId);
        entryPoint.depositTo{value: 1 ether}(op.sender);
        userOps.push(op);

        counter.count();
        uint256 countBefore = counter.counters(op.sender);
        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
        assertEq(counter.counters(op.sender), countBefore + 1);
    }

    //account should pay for high gas usage tx
    function test_PayForGasUse() public {
        UserOperation memory op = fillOp(0);
        uint256 iterations = 45;
        bytes memory counterCallData = abi.encodeWithSignature("gasWaster(uint256,string)", iterations, '');
        op.callData = abi.encodeCall(account.execute, (address(counter), 0, counterCallData));
        op.verificationGasLimit = 1e5;
        op.callGasLimit = 11e5;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);
        entryPoint.depositTo{value: 1 ether}(op.sender);

        uint256 offsetBefore = counter.offset();
        vm.recordLogs();
        entryPoint.handleOps{gas: 13e5}(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
        assertEq(counter.offset(), offsetBefore + iterations);
    }

    //account should not pay if too low gas limit was set
    function test_DontPayForLowGasLimit() public {
        UserOperation memory op = fillOp(0);
        uint256 iterations = 45;
        bytes memory counterCallData = abi.encodeWithSignature("gasWaster(uint256,string)", iterations, '');
        op.callData = abi.encodeCall(account.execute, (address(counter), 0, counterCallData));
        op.verificationGasLimit = 1e5;
        op.callGasLimit = 11e5;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA95 out of gas"));
        entryPoint.handleOps{gas: 12e5}(userOps, beneficiary);
        assertEq(entryPoint.getDepositInfo(op.sender).deposit, 1 ether);
    }

    //if account has a deposit, it should use it to pay
    function test_PayFromDeposit() public {
        UserOperation memory op = fillOp(0);
        bytes memory counterCallData = abi.encodeWithSignature("count()");
        op.callData = abi.encodeCall(account.execute, (address(counter), 0, counterCallData));
        op.verificationGasLimit = 1e6;
        op.callGasLimit = 1e6;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);
        entryPoint.depositTo{value: 1 ether}(op.sender);

        (bool _sent,) = op.sender.call{value: 1 ether}("");
        require(_sent, "Could not pay op.sender");

        uint256 balanceBefore = op.sender.balance;
        uint256 depositBefore = entryPoint.getDepositInfo(accountAddress).deposit;
        uint256 countBefore = counter.counters(op.sender);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();

        assertEq(counter.counters(op.sender), countBefore + 1);
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
        assertEq(op.sender.balance, balanceBefore);
        assertEq(depositBefore - entryPoint.getDepositInfo(op.sender).deposit, beneficiary.balance);
    }

    //should pay for reverted tx
    function test_PayForRevertedTx() public {
        UserOperation memory op = fillOp(0);
        op.callData = "0xdeadface";
        op.verificationGasLimit = 1e6;
        op.callGasLimit = 1e6;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);
        entryPoint.depositTo{value: 1 ether}(op.sender);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (, bool success,,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertFalse(success);
        bool balanceGt1 = (beneficiary.balance > 1);
        assertEq(balanceGt1, true);
    }

    //#handleOp (single)
    function test_SingleOp() public {
        UserOperation[] memory userOperations = new UserOperation[](1);

        UserOperation memory op = fillAndSign(chainId, 0);
        entryPoint.depositTo{value: 1 ether}(op.sender);
        userOperations[0] = op;

        vm.recordLogs();
        entryPoint.handleOps(userOperations, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //should fail to call recursively into handleOps
    function test_RecursiveCallToHandleOps() public {
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
        entryPoint.depositTo{value: 1 ether}(op.sender);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (, bytes memory revertReason) = abi.decode(entries[1].data, (uint256, bytes));
        assertEq(
            revertReason, abi.encodeWithSelector(bytes4(keccak256("Error(string)")), "ReentrancyGuard: reentrant call")
        );
    }

    //should report failure on insufficient verificationGas after creation
    function test_InsufficientVerificationGas() public {
        UserOperation memory op0 = fillOp(0);
        op0.verificationGasLimit = 100e5;
        op0 = signUserOp(op0, entryPointAddress, chainId);
        entryPoint.depositTo{value: 1 ether}(op0.sender);

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
    function test_CreateWrongSenderAddress() public {
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
    function test_RejectCreateIfNotFunded() public {
        uint256 salt = 123;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
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
    function test_CreateIfFunded() public {
        uint256 salt = 123;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
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
    function test_SenderAlreadyCreated() public {
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
    
    //batch multiple requests
    function batchMultipleRequests() public {
        UserOperation memory op1 = fillOp(0);
        SimpleAccount account1 = accountFactory.createAccount(owner.addr, 1);
        op1.sender = address(account1);
        op1 = signUserOp(op1, entryPointAddress, chainId);
        userOps.push(op1);
        entryPoint.depositTo{value: 1 ether}(op1.sender);

        UserOperation memory op2 = fillOp(0);
        SimpleAccount account2 = accountFactory.createAccount(owner.addr, 2);
        op2.sender = address(account2);
        op2 = signUserOp(op2, entryPointAddress, chainId);
        userOps.push(op2);
        entryPoint.depositTo{value: 1 ether}(op2.sender);
    }

    //should execute
    function test_BatchMultipleRequestsExec() public {
        batchMultipleRequests();
        vm.expectEmit(false, true, false, false);
        emit UserOperationEvent(bytes32(0), userOps[0].sender, address(0), 0, true, 0, 0);
        vm.expectEmit(false, true, false, false);
        emit UserOperationEvent(bytes32(0), userOps[1].sender, address(0), 0, true, 0, 0);
        entryPoint.handleOps(userOps, beneficiary);
    }

    //should pay for tx
    function test_BatchMultipleRequestsPay() public {
        batchMultipleRequests();
        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost1,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        (,, uint256 actualGasCost2,) = abi.decode(entries[2].data, (uint256, bool, uint256, uint256));
        assertEq(actualGasCost1 + entryPoint.getDepositInfo(userOps[0].sender).deposit, 1 ether);
        assertEq(actualGasCost2 + entryPoint.getDepositInfo(userOps[1].sender).deposit, 1 ether);
        assertEq(beneficiary.balance, actualGasCost1 + actualGasCost2);
    }

    //aggregation tests
    //should fail to execute aggregated account without an aggregator
    function test_AggrAccountWithoutAggregator() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountAddress;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA24 signature error")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //should fail to execute aggregated account with wrong aggregator
    function test_AggrAccountWithWrongAggregator() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountAddress;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        TestSignatureAggregator wrongAggregator = new TestSignatureAggregator();
        opsPerAggregator[0] = fillAggregatedOp(userOps, wrongAggregator);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA24 signature error")
        );
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should reject non-contract (address(1)) aggregator
    function test_NonExistentAggregator() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountAddress;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        bytes memory nullSignature = abi.encodePacked(bytes32(0));
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(userOps, IAggregator(address(1)), nullSignature);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectRevert("AA96 invalid aggregator");
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should fail to execute aggregated account with wrong agg. signature
    function test_WrongAggregateSig() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountAddress;
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        bytes memory wrongSignature = abi.encodePacked(uint256(0x123456));
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(userOps, aggregator, wrongSignature);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectRevert(abi.encodeWithSignature("SignatureValidationFailed(address)", aggregator));
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should run with multiple aggregators (and non-aggregated-accounts)
    function test_MultipleAggregators() public {
        UserOperation[] memory userOpsArr = new UserOperation[](2);
        UserOperation[] memory userOpsAggrArr = new UserOperation[](1);
        UserOperation[] memory userOpsNoAggrArr = new UserOperation[](1);
        TestSignatureAggregator aggregator3 = new TestSignatureAggregator();

        UserOperation memory op1 = fillOp(0);
        op1.sender = aggrAccountAddress;
        op1.signature = aggregator.validateUserOpSignature(op1);
        userOpsArr[0] = op1;

        UserOperation memory op2 = fillOp(0);
        aggrAccountFactory.createAccount(owner.addr, 2);
        address aggrAccount2 = aggrAccountFactory.getAddress(owner.addr, 2);
        op2.sender = aggrAccount2;
        op2.signature = aggregator.validateUserOpSignature(op2);
        userOpsArr[1] = op2;

        UserOperation memory op3 = fillOp(0);
        TestAggregatedAccountFactory tempAggrAccountFactory =
            new TestAggregatedAccountFactory(entryPoint, address(aggregator3));
        tempAggrAccountFactory.createAccount(owner.addr, 3);
        op3.sender = tempAggrAccountFactory.getAddress(owner.addr, 3);
        userOpsAggrArr[0] = signUserOp(op3, entryPointAddress, chainId);

        userOpsNoAggrArr[0] = fillAndSign(chainId, 0);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](3);
        opsPerAggregator[0] = fillAggregatedOp(userOpsArr, aggregator);
        opsPerAggregator[1] =
            IEntryPoint.UserOpsPerAggregator(userOpsAggrArr, aggregator3, abi.encodePacked(bytes32(0)));
        opsPerAggregator[2] =
            IEntryPoint.UserOpsPerAggregator(userOpsNoAggrArr, IAggregator(nullAddress), abi.encodePacked(bytes32(0)));

        entryPoint.depositTo{value: 1 ether}(op1.sender);
        entryPoint.depositTo{value: 1 ether}(op2.sender);
        entryPoint.depositTo{value: 1 ether}(op3.sender);
        entryPoint.depositTo{value: 1 ether}(accountAddress);

        for (uint256 i = 0; i < opsPerAggregator.length; i++) {
            vm.expectEmit(true, false, false, false);
            emit SignatureAggregatorChanged(address(opsPerAggregator[i].aggregator));
            for (uint256 j = 0; j < opsPerAggregator[i].userOps.length; j++) {
                vm.expectEmit(false, true, false, false);
                emit UserOperationEvent(bytes32(0), opsPerAggregator[i].userOps[j].sender, address(0), 0, false, 0, 0);
            }
        }
        vm.expectEmit(true, false, false, false);
        emit SignatureAggregatorChanged(address(0));
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //execution ordering
    //simulateValidation should return aggregator and its stake
    function test_AggregatorAndStakeReturned() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountFactory.getAddress(owner.addr, 1);
        bytes memory _initCallData = abi.encodeWithSignature("createAccount(address,uint256)", owner.addr, 1);
        op.initCode = abi.encodePacked(address(aggrAccountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);

        uint256 _stake = 1 ether;
        uint32 _delay = 100;
        aggregator.addStake{value: _stake}(entryPoint, _delay);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (bytes4 sig, bytes memory data) = getDataFromEncoding(revertReason);
            (,,,, IEntryPoint.AggregatorStakeInfo memory aggrStakeInfo) = abi.decode(
                data,
                (
                    IEntryPoint.ReturnInfo,
                    IStakeManager.StakeInfo,
                    IStakeManager.StakeInfo,
                    IStakeManager.StakeInfo,
                    IEntryPoint.AggregatorStakeInfo
                )
            );
            assertEq(
                sig,
                bytes4(
                    keccak256(
                        "ValidationResultWithAggregation((uint256,uint256,bool,uint48,uint48,bytes),(uint256,uint256),(uint256,uint256),(uint256,uint256),(address,(uint256,uint256)))"
                    )
                )
            );
            assertEq(aggrStakeInfo.aggregator, address(aggregator));
            assertEq(aggrStakeInfo.stakeInfo.stake, _stake);
            assertEq(aggrStakeInfo.stakeInfo.unstakeDelaySec, _delay);
        }
    }

    //should create account in handleOps
    function test_AggrCreateAccount() public {
        UserOperation memory op = fillOp(0);
        op.sender = aggrAccountFactory.getAddress(owner.addr, 1);
        bytes memory _initCallData = abi.encodeWithSignature("createAccount(address,uint256)", owner.addr, 1);
        op.initCode = abi.encodePacked(address(aggrAccountFactory), _initCallData);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        opsPerAggregator[0] = fillAggregatedOp(userOps, aggregator);

        entryPoint.depositTo{value: 1 ether}(op.sender);
        vm.expectEmit(false, true, false, false);
        emit AccountDeployed(bytes32(0), op.sender, address(0), address(0));
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //with paymaster (account with no eth)
    //should fail with nonexistent paymaster
    function test_NonExistenetPaymaster() public {
        uint256 salt = 123;
        address pm = createAddress("paymaster").addr;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op.paymasterAndData = abi.encodePacked(pm);
        op = signUserOp(op, entryPointAddress, chainId);

        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA30 paymaster not deployed")
        );
        entryPoint.simulateValidation(op);
    }

    //should fail if paymaster has no deposit
    function test_PaymasterWithNoDeposit() public {
        uint256 salt = 123;

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op.paymasterAndData = abi.encodePacked(address(paymasterAcceptAll));
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA31 paymaster deposit too low")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //paymaster should pay for tx
    function test_PaymasterPaysForTransaction() public {
        address paymaster = address(paymasterAcceptAll);
        uint256 salt = 123;

        entryPoint.depositTo{value: 1 ether}(paymaster);

        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op.paymasterAndData = abi.encodePacked(paymaster);
        op = signUserOp(op, entryPointAddress, chainId);
        userOps.push(op);

        vm.recordLogs();
        entryPoint.handleOps(userOps, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        uint256 i;
        for (i = 0; i < entries.length; i++) {
            if (
                entries[i].topics[0]
                    == keccak256("UserOperationEvent(bytes32,address,address,uint256,bool,uint256,uint256)")
            ) {
                break;
            }
        }
        (,, uint256 actualGasCost,) = abi.decode(entries[i].data, (uint256, bool, uint256, uint256));
        assertEq(entryPoint.getDepositInfo(paymaster).deposit + actualGasCost, 1 ether);
    }

    // simulateValidation should return paymaster stake and delay
    function test_ReturnPaymasterStakeInfo() public {
        address paymaster = address(paymasterAcceptAll);
        uint256 salt = 123;

        entryPoint.depositTo{value: 1 ether}(paymaster);
        UserOperation memory op = fillOp(0);
        bytes memory _initCallData = abi.encodeCall(SimpleAccountFactory.createAccount, (owner.addr, salt));
        op.sender = accountFactory.getAddress(owner.addr, salt);
        op.initCode = abi.encodePacked(address(accountFactory), _initCallData);
        op.paymasterAndData = abi.encodePacked(paymaster);
        op = signUserOp(op, entryPointAddress, chainId);

        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (,,, IStakeManager.StakeInfo memory stakeInfoFromRevert) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            IStakeManager.DepositInfo memory stakeInfo = entryPoint.getDepositInfo(paymaster);
            assertEq(stakeInfoFromRevert.stake, stakeInfo.stake);
            assertEq(stakeInfoFromRevert.unstakeDelaySec, stakeInfo.unstakeDelaySec);
        }
    }

    //Validation time-range
    //validateUserOp time-range
    //should accept non-expired owner
    function test_NonExpiredOwner() public {
        TestExpiryAccount testAccount = new TestExpiryAccount(entryPoint);
        Account memory sessionOwner = createAddress("session_owner");
        uint48 _after = uint48(block.timestamp);
        uint48 _until = uint48(block.timestamp) + 10000;
        vm.prank(nullAddress);
        testAccount.addTemporaryOwner(sessionOwner.addr, uint48(_after), uint48(_until));

        UserOperation memory op = fillOp(0);
        op.sender = address(testAccount);
        op = signUserOp(op, entryPointAddress, chainId, sessionOwner.key);

        entryPoint.depositTo{value: 1 ether}(address(testAccount));
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (IEntryPoint.ReturnInfo memory returnInfoFromRevert,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(returnInfoFromRevert.validAfter, _after);
            assertEq(returnInfoFromRevert.validUntil, _until);
        }
    }

    //should not reject expired owner
    function test_ExpiredOwner() public {
        TestExpiryAccount testAccount = new TestExpiryAccount(entryPoint);
        Account memory sessionOwner = createAddress("session_owner");
        vm.warp(100);
        uint48 _after = 0;
        uint48 _until = 99;
        vm.prank(nullAddress);
        testAccount.addTemporaryOwner(sessionOwner.addr, uint48(_after), uint48(_until));

        UserOperation memory op = fillOp(0);
        op.sender = address(testAccount);
        op = signUserOp(op, entryPointAddress, chainId, sessionOwner.key);

        entryPoint.depositTo{value: 1 ether}(address(testAccount));
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (IEntryPoint.ReturnInfo memory returnInfoFromRevert,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(returnInfoFromRevert.validAfter, _after);
            assertEq(returnInfoFromRevert.validUntil, _until);
        }
    }

    //validatePaymasterUserOp with deadline
    //should accept non-expired paymaster request
    function test_NonExpiredPaymasterRequest() public {
        address paymaster = address(expirePaymaster);
        uint48 _after = 1;
        uint48 _until = 100;
        UserOperation memory op = fillOp(0);
        op.paymasterAndData = abi.encodePacked(paymaster, _after, _until);
        op = signUserOp(op, entryPointAddress, chainId, owner.key);

        entryPoint.depositTo{value: 1 ether}(paymaster);
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (IEntryPoint.ReturnInfo memory returnInfoFromRevert,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(returnInfoFromRevert.validAfter, _after);
            assertEq(returnInfoFromRevert.validUntil, _until);
        }
    }

    //should not reject expired paymaster request
    function test_ExpiredPaymasterRequest() public {
        address paymaster = address(expirePaymaster);
        uint48 _after = 10;
        uint48 _until = 20;
        vm.warp(100);
        UserOperation memory op = fillOp(0);
        op.paymasterAndData = abi.encodePacked(paymaster, _after, _until);
        op = signUserOp(op, entryPointAddress, chainId, owner.key);

        entryPoint.depositTo{value: 1 ether}(paymaster);
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (IEntryPoint.ReturnInfo memory returnInfoFromRevert,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(returnInfoFromRevert.validAfter, _after);
            assertEq(returnInfoFromRevert.validUntil, _until);
        }
    }

    //time-range overlap of paymaster and account should intersect
    function simulateWithValidityParams(
        uint48 accountValidAfter,
        uint48 accountValidUntil,
        uint48 paymasterValidAfter,
        uint48 paymasterValidUntil
    ) public returns (uint48 validAfter, uint48 validUntil) {
        address paymaster = address(expirePaymaster);

        TestExpiryAccount testAccount = new TestExpiryAccount(entryPoint);
        Account memory sessionOwner = createAddress("session_owner");
        vm.prank(nullAddress);
        testAccount.addTemporaryOwner(sessionOwner.addr, accountValidAfter, accountValidUntil);

        UserOperation memory op = fillOp(0);
        op.sender = address(testAccount);
        op.paymasterAndData = abi.encodePacked(paymaster, paymasterValidAfter, paymasterValidUntil);
        op = signUserOp(op, entryPointAddress, chainId, sessionOwner.key);

        entryPoint.depositTo{value: 1 ether}(paymaster);
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = getDataFromEncoding(revertReason);

            (IEntryPoint.ReturnInfo memory returnInfoFromRevert,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            return (returnInfoFromRevert.validAfter, returnInfoFromRevert.validUntil);
        }
    }

    //should use lower "after" value of account
    function test_AfterOfAccount() public {
        uint48 _after = 10;
        (uint48 validAfter,) = simulateWithValidityParams(_after, 100, _after - 10, 100);
        assertEq(validAfter, _after);
    }

    //should use lower "after" value of paymaster
    function test_AfterOfPaymaster() public {
        uint48 _after = 10;
        (uint48 validAfter,) = simulateWithValidityParams(_after - 10, 100, _after, 100);
        assertEq(validAfter, _after);
    }

    //should use higher "until" value of account
    function test_UntilOfAccount() public {
        uint48 _until = 100;
        (, uint48 validUntil) = simulateWithValidityParams(10, _until, 10, _until + 10);
        assertEq(validUntil, _until);
    }

    //should use higher "until" value of paymaster
    function test_UntilOfPaymaster() public {
        uint48 _until = 100;
        (, uint48 validUntil) = simulateWithValidityParams(10, _until + 10, 10, _until);
        assertEq(validUntil, _until);
    }

    //handleOps should revert on expired paymaster request
    function test_RevertExpiredPaymasterRequest() public {
        address paymaster = address(expirePaymaster);
        uint48 _after = 10;
        uint48 _until = 20;
        vm.warp(100);
        UserOperation memory op = fillOp(0);
        op.paymasterAndData = abi.encodePacked(paymaster, _after, _until);
        op = signUserOp(op, entryPointAddress, chainId, owner.key);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(paymaster);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA32 paymaster expired or not due"
            )
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //handleOps should abort on time-range
    //should revert on expired account
    function test_RevertExpiredOwner() public {
        TestExpiryAccount testAccount = new TestExpiryAccount(entryPoint);
        Account memory sessionOwner = createAddress("session_owner");
        vm.warp(100);
        uint48 _after = 0;
        uint48 _until = 90;
        vm.prank(nullAddress);
        testAccount.addTemporaryOwner(sessionOwner.addr, uint48(_after), uint48(_until));

        UserOperation memory op = fillOp(0);
        op.sender = address(testAccount);
        op = signUserOp(op, entryPointAddress, chainId, sessionOwner.key);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(address(testAccount));
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA22 expired or not due")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }

    //should revert on date owner
    function test_RevertDateOwner() public {
        TestExpiryAccount testAccount = new TestExpiryAccount(entryPoint);
        Account memory sessionOwner = createAddress("session_owner");
        vm.warp(1);
        uint48 _after = 10;
        uint48 _until = 100;
        vm.prank(nullAddress);
        testAccount.addTemporaryOwner(sessionOwner.addr, uint48(_after), uint48(_until));

        UserOperation memory op = fillOp(0);
        op.sender = address(testAccount);
        op = signUserOp(op, entryPointAddress, chainId, sessionOwner.key);
        userOps.push(op);

        entryPoint.depositTo{value: 1 ether}(address(testAccount));
        vm.expectRevert(
            abi.encodeWithSelector(bytes4(keccak256("FailedOp(uint256,string)")), 0, "AA22 expired or not due")
        );
        entryPoint.handleOps(userOps, beneficiary);
    }
}
