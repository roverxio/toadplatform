// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/test/TestWarmColdAccount.sol";
import "../src/test/TestPaymasterAcceptAll.sol";
import "../src/test/TestRevertAccount.sol";
import "../src/test/TestExpiryAccount.sol";
import "../src/test/TestExpirePaymaster.sol";
import "../src/test/TestAggregatedAccount.sol";
import "../src/test/TestAggregatedAccountFactory.sol";
import "../src/test/TestSignatureAggregator.sol";
import "../src/test/TestCounter.sol";

//Utils
import {Utilities} from "./Utilities.sol";

struct ReturnInfo {
    uint256 preOpGas;
    uint256 prefund;
    bool sigFailed;
    uint48 validAfter;
    uint48 validUntil;
    bytes paymasterContext;
}

struct StakeInfo {
    uint256 stake;
    uint256 unstakeDelaySec;
}

struct AggregatorStakeInfo {
    address aggregator;
    StakeInfo stakeInfo;
}

contract EntryPointTest is TestHelper {
    UserOperation[] internal ops;
    Utilities internal utils;

    event UserOperationEvent(
        bytes32 indexed userOpHash,
        address indexed sender,
        address indexed paymaster,
        uint256 nonce,
        bool success,
        uint256 actualGasCost,
        uint256 actualGasUsed
    );

    event SignatureAggregatorChanged(address indexed aggregator);

    event AccountDeployed(bytes32 indexed userOpHash, address indexed sender, address factory, address paymaster);

    function setUp() public {
        utils = new Utilities();
        accountOwner = utils.createAddress("entrypoint_Owner");
        deployEntryPoint(1101);
        createAccount(1102, 1103);
        vm.deal(accountAddress, 1 ether);
    }

    // Stake Management testing
    // Should deposit for transfer into EntryPoint
    function test_Deposit() public {
        address signerAddress = utils.createAddress("deposit_address").addr;
        vm.deal(signerAddress, 1 ether);
        vm.startPrank(signerAddress);
        (bool success,) = payable(entryPointAddress).call{value: 1 ether}("");
        assert(success);

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

    // Should report "staked" state
    function test_StakedState() public {
        vm.prank(accountOwner.addr);
        _withStakeOf2EthSetup();

        assertEq(entryPoint.getDepositInfo(accountOwner.addr).deposit, 0);
        assertEq(entryPoint.getDepositInfo(accountOwner.addr).staked, true);
        assertEq(entryPoint.getDepositInfo(accountOwner.addr).stake, 2 ether);
        assertEq(entryPoint.getDepositInfo(accountOwner.addr).unstakeDelaySec, 2);
        assertEq(entryPoint.getDepositInfo(accountOwner.addr).withdrawTime, 0);
    }

    // should succeed to stake again
    function test_SucceedToStakeAgain() public {
        vm.startPrank(accountOwner.addr);
        _withStakeOf2EthSetup();

        uint112 stake = entryPoint.getDepositInfo(accountOwner.addr).stake;
        entryPoint.addStake{value: 1 ether}(2);
        uint112 stakeAfter = entryPoint.getDepositInfo(accountOwner.addr).stake;
        assertEq(stakeAfter, stake + 1 ether);

        vm.stopPrank();
    }

    // should fail to withdraw before unlock
    function test_FailToWithdrawBeforeUnlock() public {
        vm.startPrank(accountOwner.addr);
        _withStakeOf2EthSetup();

        vm.expectRevert("must call unlockStake() first");
        entryPoint.withdrawStake(payable(address(0)));

        vm.stopPrank();
    }

    // should report as "not staked"
    function test_ReportAsNotStaked() public {
        vm.startPrank(accountOwner.addr);
        _withUnlockedStakeSetup();

        assertEq(entryPoint.getDepositInfo(accountOwner.addr).staked, false);

        vm.stopPrank();
    }

    // should report unstake state
    function test_ReportUnstakeState() public {
        vm.startPrank(accountOwner.addr);
        _withUnlockedStakeSetup();

        uint48 withdrawTime1 = uint48(block.timestamp + globalUnstakeDelaySec);
        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(accountOwner.addr);
        /*
        The corresponding hardhat test case is dependent on running the previous test cases for
        the stake to be 3 ether. However, on running this test case alone, the stake is 2 ether.
        */
        assertEq(info.stake, 2 ether);
        assertEq(info.staked, false);
        assertEq(info.unstakeDelaySec, 2);
        assertEq(info.withdrawTime, withdrawTime1);

        vm.stopPrank();
    }

    // should fail to withdraw before unlock timeout
    function test_FailToWithdrawBeforeUnlockTimeout() public {
        vm.startPrank(accountOwner.addr);
        _withUnlockedStakeSetup();

        vm.expectRevert("Stake withdrawal is not due");
        entryPoint.withdrawStake(payable(address(0)));

        vm.stopPrank();
    }

    // should fail to unlock again
    function test_FailToUnlockAgain() public {
        vm.startPrank(accountOwner.addr);
        _withUnlockedStakeSetup();

        vm.expectRevert("already unstaking");
        entryPoint.unlockStake();

        vm.stopPrank();
    }

    // adding stake should reset "unlockStake"
    function test_ResetUnlockStakeOnAddingStake() public {
        vm.startPrank(accountOwner.addr);
        _afterUnstakeDelaySetup();

        uint256 snap = vm.snapshot();

        payable(accountOwner.addr).transfer(0);
        entryPoint.addStake{value: 1 ether}(2);
        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(accountOwner.addr);
        /*
        The corresponding hardhat test case is dependent on running the previous test cases for
        the stake to be 4 ether. However, on running this test case alone, the stake is 3 ether.
        */
        assertEq(info.stake, 3 ether);
        assertEq(info.staked, true);
        assertEq(info.unstakeDelaySec, 2);
        assertEq(info.withdrawTime, 0);

        vm.revertTo(snap);

        vm.stopPrank();
    }

    // should fail to unlock again
    function test_FailToUnlockAgainAfterUnstakeDelay() public {
        vm.startPrank(accountOwner.addr);
        _afterUnstakeDelaySetup();

        vm.expectRevert("already unstaking");
        entryPoint.unlockStake();

        vm.stopPrank();
    }

    // should succeed to withdraw
    function test_SucceedToWithdraw() public {
        vm.startPrank(accountOwner.addr);
        _afterUnstakeDelaySetup();

        uint112 stake = entryPoint.getDepositInfo(accountOwner.addr).stake;
        address payable addr1 = payable(utils.createAddress("addr1").addr);
        entryPoint.withdrawStake(addr1);
        assertEq(addr1.balance, stake);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(accountOwner.addr);
        assertEq(info.stake, 0);
        assertEq(info.unstakeDelaySec, 0);
        assertEq(info.withdrawTime, 0);

        vm.stopPrank();
    }

    // With deposit
    // Should be able to withdraw
    function test_WithdrawDeposit() public {
        (SimpleAccount account1, address accountAddress1) = createAccountWithFactory(1104);
        account1.addDeposit{value: 1 ether}();

        assertEq(utils.getBalance(accountAddress1), 0);
        assertEq(account1.getDeposit(), 1 ether);

        uint256 depositBefore = account1.getDeposit();
        vm.prank(accountOwner.addr);
        account1.withdrawDepositTo(payable(accountAddress1), 1 ether);

        assertEq(utils.getBalance(accountAddress1), 1 ether);
        assertEq(account1.getDeposit(), depositBefore - 1 ether);
    }

    //simulationValidation
    /// @notice 1. Should fail if validateUserOp fails
    function test_FailureOnValidateOpFailure() public {
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.nonce = 1234;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA25 invalid account nonce"));
        entryPoint.simulateValidation(op1);
    }

    /// @notice 2. Should report signature failure without revert
    function test_reportSignatureFailureWithoutRevert() public {
        IEntryPoint.ReturnInfo memory returnInfo;
        address account1;
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        (, account1) = createAccountWithFactory(1105, accountOwner1.addr);

        UserOperation memory op = defaultOp;
        op.sender = account1;
        op.maxFeePerGas = 0;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);

        try entryPoint.simulateValidation(op1) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
        }

        assertEq(returnInfo.sigFailed, true);
    }

    /// @notice  3. Should revert if wallet not deployed (and no initCode)
    function test_shouldRevertIfWalletNotDeployed() public {
        UserOperation memory op = defaultOp;
        op.sender = utils.createAddress("randomAccount").addr;
        op.callGasLimit = 1000;
        op.verificationGasLimit = 1000;
        op.maxFeePerGas = 0;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA20 account not deployed"));
        entryPoint.simulateValidation(op1);
    }

    /// @notice  4. Should revert on OOG if not enough verificationGas
    function test_shouldRevertIfNotEnoughGas() public {
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callGasLimit = 1000;
        op.verificationGasLimit = 1000;
        op.maxFeePerGas = 0;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA23 reverted (or OOG)"));
        entryPoint.simulateValidation(op1);
    }

    /// @notice 5. Should succeed if validUserOp succeeds: TBD
    function test_shouldSucceedIfUserOpSucceeds() public {
        IEntryPoint.ReturnInfo memory returnInfo;
        address account1;
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        (, account1) = createAccountWithFactory(1106);

        UserOperation memory op = defaultOp;
        op.sender = account1;
        op.callGasLimit = 0;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 1381937087;
        op.maxPriorityFeePerGas = 1000000000;

        vm.deal(account1, 1 ether);
        UserOperation memory op1 = utils.signUserOp(op, accountOwner1.key, entryPointAddress, chainId);
        try entryPoint.simulateValidation(op1) {}
        catch (bytes memory revertReason) {
            (bytes4 sig, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(sig, utils.validationResultEvent());
        }
    }

    /// @notice 6. Should return empty context if no Paymaster
    function test_shouldReturnEmptyContextIfNoPaymaster() public {
        IEntryPoint.ReturnInfo memory returnInfo;
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callGasLimit = 0;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 0;
        op.maxPriorityFeePerGas = 1000000000;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        try entryPoint.simulateValidation(op1) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
        }
        assertEq(returnInfo.paymasterContext, defaultBytes);
    }

    /// @notice 7. Should return stake of sender
    function test_shouldReturnSendersStake() public {
        IStakeManager.StakeInfo memory senderInfo;
        uint256 stakeValue = 123;
        uint32 unstakeDelay = 3;
        SimpleAccount account2;
        address accountAddress2;
        Account memory accountOwner2 = utils.createAddress("accountOwner2");
        (account2, accountAddress2) = createAccountWithFactory(1107, accountOwner2.addr);
        vm.deal(accountAddress2, 1 ether);
        vm.prank(accountOwner2.addr);
        account2.execute(entryPointAddress, stakeValue, abi.encodeWithSignature("addStake(uint32)", unstakeDelay));

        UserOperation memory op = defaultOp;
        op.sender = accountAddress2;
        op.maxFeePerGas = 0;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner2.key, entryPointAddress, chainId);
        try entryPoint.simulateValidation(op1) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (, senderInfo,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
        }

        assertEq(senderInfo.stake, stakeValue);
        assertEq(senderInfo.unstakeDelaySec, unstakeDelay);
    }

    /// @notice 8. Should prevent overflows: fail if any numeric value is more than 120 bits
    function test_shouldPreventOverflows() public {
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callGasLimit = 0;
        op.verificationGasLimit = 150000;
        op.preVerificationGas = 2 ** 130;
        op.maxFeePerGas = 0;
        op.maxPriorityFeePerGas = 1000000000;

        vm.expectRevert("AA94 gas values overflow");
        entryPoint.simulateValidation(op);
    }

    /// @notice 9. Should fail creation for wrong sender
    function test_shouldFailCreationOnWrongSender() public {
        UserOperation memory op = defaultOp;
        op.sender = 0x1111111111111111111111111111111111111111;
        op.initCode = utils.getAccountInitCode(accountOwner.addr, simpleAccountFactory, 0);
        op.callGasLimit = 0;
        op.verificationGasLimit = 3000000;
        op.maxFeePerGas = 1381937087;
        op.maxPriorityFeePerGas = 1000000000;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA14 initCode must return sender"));
        entryPoint.simulateValidation(op1);
    }

    /// @notice 10. Should report failure on insufficient verificationGas for creation
    function test_shouldReportFailureOnInsufficientVerificationGas() public {
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        address addr;
        bytes memory initCode = utils.getAccountInitCode(accountOwner1.addr, simpleAccountFactory, 0);

        try entryPoint.getSenderAddress(initCode) {}
        catch (bytes memory reason) {
            require(reason.length >= 4);
            (, bytes memory data) = utils.getDataFromEncoding(reason);
            addr = abi.decode(data, (address));
        }
        UserOperation memory op = defaultOp;
        op.sender = addr;
        op.nonce = 0;
        op.initCode = initCode;
        op.callGasLimit = 0;
        op.verificationGasLimit = 500000;
        op.preVerificationGas = 0;
        op.maxFeePerGas = 0;
        op.maxPriorityFeePerGas = 1000000000;

        UserOperation memory op1 = utils.signUserOp(op, accountOwner1.key, entryPointAddress, chainId);
        try entryPoint.simulateValidation{gas: 1e6}(op1) {}
        catch (bytes memory errorReason) {
            bytes4 reason;
            assembly {
                reason := mload(add(errorReason, 32))
            }
            assertEq(reason, utils.validationResultEvent());
        }

        op1.verificationGasLimit = 1e5;
        UserOperation memory op2 = utils.signUserOp(op1, accountOwner1.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA13 initCode failed or OOG"));
        entryPoint.simulateValidation(op2);
    }

    /// @notice 11. Should succeed for creating an account
    function test_shouldSucceedCreatingAccount() public {
        IEntryPoint.ReturnInfo memory returnInfo;
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        address sender = utils.getAccountAddress(accountOwner1.addr, simpleAccountFactory, 0);

        UserOperation memory op = defaultOp;
        op.sender = sender;
        op.initCode = utils.getAccountInitCode(accountOwner1.addr, simpleAccountFactory, 0);
        op.callGasLimit = 0;
        op.verificationGasLimit = 3000000;
        op.maxFeePerGas = 1381937087;
        op.maxPriorityFeePerGas = 1000000000;
        UserOperation memory op1 = utils.signUserOp(op, accountOwner1.key, entryPointAddress, chainId);

        vm.deal(op1.sender, 1 ether);
        try entryPoint.simulateValidation(op1) {}
        catch (bytes memory revertReason) {
            (bytes4 sig, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
            assertEq(sig, utils.validationResultEvent());
        }
    }

    //    12. Should not call initCode from EntryPoint
    function test_shouldNotCallInitCodeFromEntryPoint() public {
        address account1;
        Account memory sender = utils.createAddress("accountOwner1");
        (, account1) = createAccountWithFactory(1108);
        bytes memory initCode = utils.hexConcat(
            abi.encodePacked(account1), abi.encodeWithSignature("execute(address,uint,bytes)", sender, 0, "0x")
        );
        UserOperation memory op = defaultOp;
        op.sender = sender.addr;
        op.initCode = initCode;
        op.callGasLimit = 0;
        op.verificationGasLimit = 3000000;
        op.maxFeePerGas = 1381937087;
        op.maxPriorityFeePerGas = 1000000000;
        UserOperation memory op1 = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        vm.expectRevert(utils.failedOp(0, "AA13 initCode failed or OOG"));
        entryPoint.simulateValidation(op1);
    }

    //    13. Should not use banned ops during simulateValidation
    function test_shouldNotUseBannedOps() public {}

    // #simulateHandleOp
    // Should simulate execution
    function test_ExecutionSimulation() public {
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        (, address accountAddress1) = createAccountWithFactory(1109);
        vm.deal(accountAddress1, 1 ether);
        TestCounter counter = new TestCounter{salt: bytes32(uint256(1110))}();
        bytes memory countData = abi.encodeWithSignature("count()");
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, countData);

        UserOperation memory op = defaultOp;
        op.sender = accountAddress1;
        op.callData = callData;

        op = utils.signUserOp(op, accountOwner1.key, entryPointAddress, chainId);

        vm.recordLogs();
        try entryPoint.simulateHandleOp(
            op, address(counter), abi.encodeWithSignature("counters(address)", accountAddress1)
        ) {} catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (,,,, bool success, bytes memory result) = abi.decode(data, (uint256, uint256, uint48, uint48, bool, bytes));
            assertEq(success, true);
            assertEq(result, abi.encode(1));
        }
        assertEq(counter.counters(accountAddress1), 0);
    }

    // Flickering account validation
    // Should prevent leakage of base fee
    function test_BaseFeeLeakage() public {
        /**
         * Note: Not completing this test cases as it includes RPC calls
         * Create a malicious account
         * Take snapshot
         * RPC call 'evm_mine'
         * Get latest block
         * RPC call 'evm_revert'
         * Validate block baseFeePerGas and expect failure
         * Generate UserOp
         * Trigger Simulate validation
         * Handle revert
         * RPC call 'evm_mine'
         * Trigger Simulate validation
         * Handle revert
         * Expect failures with error messages
         */
    }

    // Should limit revert reason length before emitting it
    function test_RevertReasonLength() public {
        (uint256 revertLength, uint256 REVERT_REASON_MAX_LENGTH) = (1e5, 2048);
        vm.deal(entryPointAddress, 1 ether);
        TestRevertAccount testAccount = new TestRevertAccount(entryPoint);
        bytes memory revertCallData = abi.encodeWithSignature("revertLong(uint256)", revertLength + 1);
        UserOperation memory badOp = defaultOp;
        badOp.sender = address(testAccount);
        badOp.callGasLimit = 1e5;
        badOp.maxFeePerGas = 1;
        badOp.nonce = entryPoint.getNonce(address(testAccount), 0);
        badOp.verificationGasLimit = 1e5;
        badOp.callData = revertCallData;
        badOp.maxPriorityFeePerGas = 1e9;

        vm.deal(address(testAccount), 0.01 ether);
        Account memory beneficiary = utils.createAddress("beneficiary");
        try entryPoint.simulateValidation{gas: 3e5}(badOp) {}
        catch (bytes memory errorReason) {
            (bytes4 sig,) = utils.getDataFromEncoding(errorReason);
            assertEq(sig, utils.validationResultEvent());
        }
        ops.push(badOp);
        vm.recordLogs();
        entryPoint.handleOps(ops, payable(beneficiary.addr));
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs[2].topics[0], keccak256("UserOperationRevertReason(bytes32,address,uint256,bytes)"));
        (, bytes memory revertReason) = abi.decode(logs[2].data, (uint256, bytes));
        assertEq(revertReason.length, REVERT_REASON_MAX_LENGTH);
    }

    // Warm/cold storage detection in simulation vs execution
    // Should prevent detection through getAggregator()
    function test_DetectionThroughGetAggregator() public {
        uint256 TOUCH_GET_AGGREGATOR = 1;
        TestWarmColdAccount testAccount = new TestWarmColdAccount(entryPoint);
        UserOperation memory badOp = defaultOp;
        badOp.nonce = TOUCH_GET_AGGREGATOR;
        badOp.sender = address(testAccount);

        Account memory beneficiary = utils.createAddress("beneficiary");

        try entryPoint.simulateValidation{gas: 1e6}(badOp) {}
        catch (bytes memory revertReason) {
            (bytes4 sig,) = utils.getDataFromEncoding(revertReason);
            if (sig == utils.validationResultEvent()) {
                ops.push(badOp);
                entryPoint.handleOps{gas: 1e6}(ops, payable(beneficiary.addr));
            } else {
                assertEq(revertReason, utils.failedOp(0, "AA23 reverted (or OOG)"));
            }
        }
    }

    // Should prevent detection through paymaster.code.length
    function test_DetectionThroughPaymasterCodeLength() public {
        uint256 TOUCH_PAYMASTER = 2;
        TestWarmColdAccount testAccount = new TestWarmColdAccount(entryPoint);
        TestPaymasterAcceptAll paymaster = new TestPaymasterAcceptAll(entryPoint);
        paymaster.deposit{value: 1 ether}();

        UserOperation memory badOp = defaultOp;
        badOp.nonce = TOUCH_PAYMASTER;
        badOp.sender = address(testAccount);
        badOp.paymasterAndData = abi.encodePacked(address(paymaster));
        badOp.verificationGasLimit = 1000;

        Account memory beneficiary = utils.createAddress("beneficiary");

        try entryPoint.simulateValidation{gas: 1e6}(badOp) {}
        catch (bytes memory revertReason) {
            (bytes4 sig,) = utils.getDataFromEncoding(revertReason);
            if (sig == utils.validationResultEvent()) {
                ops.push(badOp);
                entryPoint.handleOps{gas: 1e6}(ops, payable(beneficiary.addr));
            } else {
                assertEq(revertReason, utils.failedOp(0, "AA23 reverted (or OOG)"));
            }
        }
    }

    // 2d nonces
    // Should fail nonce with new key and seq!=0
    function test_FailNonce() public {
        (Account memory beneficiary,, uint256 keyShifted, address _accountAddress) = _2dNonceSetup(false);

        UserOperation memory op = defaultOp;
        op.sender = _accountAddress;
        op.nonce = keyShifted + 1;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(utils.failedOp(0, "AA25 invalid account nonce"));
        entryPoint.handleOps(ops, payable(beneficiary.addr));
    }

    // With key=1, seq=1
    // should get next nonce value by getNonce
    function test_GetNonce() public {
        (, uint256 key, uint256 keyShifted, address _accountAddress) = _2dNonceSetup(true);

        uint256 nonce = entryPoint.getNonce(_accountAddress, uint192(key));
        assertEq(nonce, keyShifted + 1);
    }

    // Should allow to increment nonce of different key
    function test_IncrementNonce() public {
        (Account memory beneficiary, uint256 key,, address _accountAddress) = _2dNonceSetup(true);

        UserOperation memory op2 = defaultOp;
        op2.sender = _accountAddress;
        op2.nonce = entryPoint.getNonce(_accountAddress, uint192(key));
        op2 = utils.signUserOp(op2, accountOwner.key, entryPointAddress, chainId);
        ops[0] = op2;

        entryPoint.handleOps(ops, payable(beneficiary.addr));
    }

    // should allow manual nonce increment
    function test_ManualNonceIncrement() public {
        (Account memory beneficiary, uint256 key,, address _accountAddress) = _2dNonceSetup(true);

        uint192 incNonceKey = 5;
        bytes memory increment = abi.encodeWithSignature("incrementNonce(uint192)", incNonceKey);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", entryPointAddress, 0, increment);

        UserOperation memory op2 = defaultOp;
        op2.sender = _accountAddress;
        op2.callData = callData;
        op2.nonce = entryPoint.getNonce(_accountAddress, uint192(key));
        op2 = utils.signUserOp(op2, accountOwner.key, entryPointAddress, chainId);
        ops[0] = op2;

        entryPoint.handleOps(ops, payable(beneficiary.addr));

        uint256 nonce = entryPoint.getNonce(_accountAddress, incNonceKey);
        assertEq(nonce, (incNonceKey * 2 ** 64) + 1);
    }

    // Should fail with nonSequential seq
    function test_NonSequentialNonce() public {
        (Account memory beneficiary,, uint256 keyShifted, address _accountAddress) = _2dNonceSetup(true);

        UserOperation memory op2 = defaultOp;
        op2.sender = _accountAddress;
        op2.nonce = keyShifted + 3;
        op2 = utils.signUserOp(op2, accountOwner.key, entryPointAddress, chainId);
        ops[0] = op2;

        vm.expectRevert(utils.failedOp(0, "AA25 invalid account nonce"));
        entryPoint.handleOps(ops, payable(beneficiary.addr));
    }

    // Without paymaster (account pays in eth)
    // #handleOps
    // Should revert on signature failure
    function test_RevertOnSignatureFailure() public {
        Account memory wrong_owner = utils.createAddress("wrong_owner");
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op = utils.signUserOp(op, wrong_owner.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(makeAddr("beneficiary"));

        vm.expectRevert(utils.failedOp(0, "AA24 signature error"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //account should pay for transaction
    function test_PayForTransaction() public {
        (TestCounter counter, bytes memory accountExecFromEntryPoint) = _handleOpsSetUp();

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = accountExecFromEntryPoint;
        op.verificationGasLimit = 1e6;
        op.callGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        uint256 countBefore = counter.counters(accountAddress);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);

        uint256 countAfter = counter.counters(accountAddress);
        assertEq(countAfter, countBefore + 1);

        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[2].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //account should pay for high gas usage tx
    function test_PayForHighGasUse() public {
        (TestCounter counter,) = _handleOpsSetUp();
        uint256 iterations = 45;
        bytes memory countData = abi.encodeWithSignature("gasWaster(uint256,string)", iterations, "");
        bytes memory accountExec =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, countData);

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = accountExec;
        op.verificationGasLimit = 1e5;
        op.callGasLimit = 11e5;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        uint256 offsetBefore = counter.offset();

        vm.recordLogs();
        entryPoint.handleOps{gas: 13e6}(ops, beneficiary);

        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[2].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);

        assertEq(counter.offset(), offsetBefore + iterations);
    }

    //account should not pay if too low gas limit was set
    function test_DontPayForLowGasLimit() public {
        (TestCounter counter,) = _handleOpsSetUp();
        uint256 iterations = 45;
        bytes memory countData = abi.encodeWithSignature("gasWaster(uint256,string)", iterations, "");
        bytes memory accountExec =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, countData);

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = accountExec;
        op.verificationGasLimit = 1e5;
        op.callGasLimit = 11e5;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        uint256 initialAccountBalance = accountAddress.balance;
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        vm.expectRevert(utils.failedOp(0, "AA95 out of gas"));
        entryPoint.handleOps{gas: 12e5}(ops, beneficiary);

        assertEq(accountAddress.balance, initialAccountBalance);
    }

    //if account has a deposit, it should use it to pay
    function test_PayFromDeposit() public {
        (TestCounter counter, bytes memory accountExecFromEntryPoint) = _handleOpsSetUp();
        account.addDeposit{value: 1 ether}();

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = accountExecFromEntryPoint;
        op.verificationGasLimit = 1e6;
        op.callGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        uint256 countBefore = counter.counters(op.sender);
        uint256 balBefore = op.sender.balance;
        uint256 depositBefore = entryPoint.balanceOf(accountAddress);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 countAfter = counter.counters(op.sender);
        assertEq(countAfter, countBefore + 1);

        uint256 balAfter = op.sender.balance;
        uint256 depositAfter = entryPoint.balanceOf(accountAddress);
        assertEq(balAfter, balBefore, "should pay from stake, not balance");
        uint256 depositUsed = depositBefore - depositAfter;
        assertEq(beneficiary.balance, depositUsed);

        (,, uint256 actualGasCost,) = abi.decode(entries[1].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //should pay for reverted tx
    function test_PayForRevertedTx() public {
        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = "0xdeadface";
        op.verificationGasLimit = 1e6;
        op.callGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();

        (, bool success,,) = abi.decode(entries[2].data, (uint256, bool, uint256, uint256));
        assertFalse(success);
        assertGe(beneficiary.balance, 1);
    }

    //#handleOp (single)
    function test_SingleOp() public {
        (TestCounter counter, bytes memory accountExecFromEntryPoint) = _handleOpsSetUp();
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = accountExecFromEntryPoint;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);
        uint256 countBefore = counter.counters(accountAddress);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);

        uint256 countAfter = counter.counters(accountAddress);
        assertEq(countAfter, countBefore + 1);

        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[2].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //should fail to call recursively into handleOps
    function test_RecursiveCallToHandleOps() public {
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        UserOperation[] memory _ops;
        bytes memory callHandleOps = abi.encodeWithSignature(
            "handleOps((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes)[],address)",
            _ops,
            beneficiary
        );
        bytes memory execHandlePost =
            abi.encodeWithSignature("execute(address,uint256,bytes)", entryPointAddress, 0, callHandleOps);

        UserOperation memory op = defaultOp;
        op.sender = accountAddress;
        op.callData = execHandlePost;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (, bytes memory revertReason) = abi.decode(entries[2].data, (uint256, bytes));
        assertEq(
            revertReason,
            abi.encodeWithSignature("Error(string)", "ReentrancyGuard: reentrant call"),
            "execution of handleOps inside a UserOp should revert"
        );
    }

    //should report failure on insufficient verificationGas after creation
    function test_InsufficientVerificationGas() public {
        UserOperation memory op0 = defaultOp;
        op0.sender = accountAddress;
        op0.verificationGasLimit = 5e5;
        op0 = utils.signUserOp(op0, accountOwner.key, entryPointAddress, chainId);

        try entryPoint.simulateValidation(op0) {}
        catch (bytes memory revertReason) {
            (bytes4 reason,) = utils.getDataFromEncoding(revertReason);
            assertEq(reason, utils.validationResultEvent());
        }

        UserOperation memory op1 = defaultOp;
        op1.sender = accountAddress;
        op1.verificationGasLimit = 10000;
        op1 = utils.signUserOp(op1, accountOwner.key, entryPointAddress, chainId);

        vm.expectRevert(utils.failedOp(0, "AA23 reverted (or OOG)"));
        entryPoint.simulateValidation(op1);
    }

    //create account
    //should reject create if sender address is wrong
    function test_RejectCreateIfWrongSender() public {
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        UserOperation memory op = defaultOp;
        op.initCode = utils.getAccountInitCode(accountOwner.addr, simpleAccountFactory, 0);
        op.verificationGasLimit = 2e6;
        op.sender = 0x1111111111111111111111111111111111111111;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(utils.failedOp(0, "AA14 initCode must return sender"));
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
    }

    //should reject create if account not funded
    function test_RejectCreateIfNotFunded() public {
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        uint256 salt = 100;

        UserOperation memory op = defaultOp;
        op.sender = simpleAccountFactory.getAddress(accountOwner.addr, salt);
        op.initCode = utils.getAccountInitCode(accountOwner.addr, simpleAccountFactory, salt);
        op.verificationGasLimit = 2e6;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);

        assertEq(op.sender.balance, 0);
        vm.expectRevert(utils.failedOp(0, "AA21 didn't pay prefund"));
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
    }

    //should succeed to create account after prefund
    function test_CreateIfFunded() public {
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        uint256 salt = 20;
        address preAddr = simpleAccountFactory.getAddress(accountOwner.addr, salt);
        vm.deal(preAddr, 1 ether);

        UserOperation memory createOp = defaultOp;
        createOp.sender = preAddr;
        createOp.initCode = utils.getAccountInitCode(accountOwner.addr, simpleAccountFactory, salt);
        createOp.callGasLimit = 1e6;
        createOp.verificationGasLimit = 2e6;
        createOp = utils.signUserOp(createOp, accountOwner.key, entryPointAddress, chainId);
        ops.push(createOp);

        assertEq(utils.isContract(preAddr), false, "account exists before creation");

        bytes32 hash = entryPoint.getUserOpHash(createOp);
        vm.expectEmit(true, true, true, true);
        //createOp.initCode.toString().slice(0, 42) gets the address of simpleAccountFactory
        emit AccountDeployed(hash, createOp.sender, address(simpleAccountFactory), address(0));
        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiary);

        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[6].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
    }

    //should reject if account already created
    function test_AccountAlreadyCreated() public {
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        // `salt = 0` corresponds to default `account` created during setup
        uint256 salt = 0;
        address preAddr = simpleAccountFactory.getAddress(accountOwner.addr, salt);

        if (!utils.isContract(preAddr)) {
            UserOperation memory createOp = defaultOp;
            createOp.sender = accountAddress;
            createOp.initCode = utils.getAccountInitCode(accountOwner.addr, simpleAccountFactory, salt);
            createOp = utils.signUserOp(createOp, accountOwner.key, entryPointAddress, chainId);
            ops.push(createOp);

            vm.expectRevert(utils.failedOp(0, "AA10 sender already constructed"));
            entryPoint.handleOps{gas: 1e7}(ops, beneficiary);
        }
    }

    // batch multiple requests
    // Should execute
    function test_BatchMultipleRequestsShouldExecute() public {
        //timeout feature is not implemented in these test cases
        uint256 salt = 123;
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        Account memory accountOwner2 = utils.createAddress("accountOwner2");

        TestCounter counter = new TestCounter();
        bytes memory count = abi.encodeWithSignature("count()");
        bytes memory accountExecFromEntryPoint =
                            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, count);
        address account1 = simpleAccountFactory.getAddress(accountOwner1.addr, salt);
        (SimpleAccount account2,) = createAccountWithFactory(1112, accountOwner2.addr);
        vm.deal(account1, 1 ether);
        vm.deal(address(account2), 1 ether);

        UserOperation memory op1 = defaultOp;
        op1.sender = account1;
        op1.initCode = utils.getAccountInitCode(accountOwner1.addr, simpleAccountFactory, salt);
        op1.callData = accountExecFromEntryPoint;
        op1.callGasLimit = 2e6;
        op1.verificationGasLimit = 2e6;
        op1 = utils.signUserOp(op1, accountOwner1.key, entryPointAddress, chainId);
        ops.push(op1);

        UserOperation memory op2 = defaultOp;
        op2.callData = accountExecFromEntryPoint;
        op2.sender = address(account2);
        op2.callGasLimit = 2e6;
        op2.verificationGasLimit = 76000;
        op2 = utils.signUserOp(op2, accountOwner2.key, entryPointAddress, chainId);
        ops.push(op2);

        vm.expectRevert();
        entryPoint.simulateValidation{gas: 1e9}(op2);

        vm.deal(op1.sender, 1 ether);
        vm.deal(address(account2), 1 ether);

        entryPoint.handleOps(ops, beneficiary);
        assertEq(counter.counters(account1), 1);
        assertEq(counter.counters(address(account2)), 1);
    }

    // Aggregation Tests
    //should fail to execute aggregated account without an aggregator
    function test_FailToExecAggregateAccountWithoutAggregator() public {
        (address payable beneficiary,, TestAggregatedAccount aggAccount,) = _aggregationTestsSetUp();

        UserOperation memory userOp = defaultOp;
        userOp.sender = address(aggAccount);
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);
        ops.push(userOp);

        vm.expectRevert(utils.failedOp(0, "AA24 signature error"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //should fail to execute aggregated account with wrong aggregator
    function test_FailAggregateAccountWithWrongAggregator() public {
        (address payable beneficiary,, TestAggregatedAccount aggAccount,) = _aggregationTestsSetUp();

        UserOperation memory userOp = defaultOp;
        userOp.sender = address(aggAccount);
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);
        ops.push(userOp);

        TestSignatureAggregator wrongAggregator = new TestSignatureAggregator();
        bytes memory sig = abi.encodePacked(bytes32(0));

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(ops, wrongAggregator, sig);

        vm.expectRevert(utils.failedOp(0, "AA24 signature error"));
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should reject non-contract (address(1)) aggregator
    function test_RejectNonContractAggregator() public {
        (address payable beneficiary,,,) = _aggregationTestsSetUp();
        address address1 = address(1);
        TestAggregatedAccount aggAccount1 = new TestAggregatedAccount(entryPoint, address1);

        UserOperation memory userOp = defaultOp;
        userOp.sender = address(aggAccount1);
        userOp.maxFeePerGas = 0;
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);
        ops.push(userOp);

        bytes memory sig = abi.encodePacked(bytes32(0));

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(ops, IAggregator(address1), sig);

        vm.expectRevert("AA96 invalid aggregator");
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should fail to execute aggregated account with wrong agg. signature
    function test_FailToExecuteAggregateAccountWithWrongAggregateSig() public {
        (address payable beneficiary, TestSignatureAggregator aggregator, TestAggregatedAccount aggAccount,) =
            _aggregationTestsSetUp();

        UserOperation memory userOp = defaultOp;
        userOp.sender = address(aggAccount);
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);
        ops.push(userOp);

        bytes memory wrongSig = abi.encode(uint256(0x123456));
        address aggAddress = address(aggregator);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(ops, aggregator, wrongSig);

        vm.expectRevert(abi.encodeWithSignature("SignatureValidationFailed(address)", aggAddress));
        entryPoint.handleAggregatedOps(opsPerAggregator, beneficiary);
    }

    //should run with multiple aggregators (and non-aggregated-accounts)
    function test_MultipleAggregators() public {
        (, TestSignatureAggregator aggregator, TestAggregatedAccount aggAccount, TestAggregatedAccount aggAccount2) =
            _aggregationTestsSetUp();

        // UserOps initialization
        UserOperation[] memory userOpArr = new UserOperation[](2);
        UserOperation[] memory userOp_agg3Arr = new UserOperation[](1);
        UserOperation[] memory userOp_noAggArr = new UserOperation[](1);

        TestSignatureAggregator aggregator3 = new TestSignatureAggregator();
        TestAggregatedAccount aggAccount3 = new TestAggregatedAccount(entryPoint, address(aggregator3));
        vm.deal(address(aggAccount3), 0.1 ether);

        //did not sign the userOps as the signature is overwritten below
        UserOperation memory userOp1 = defaultOp;
        userOp1.sender = address(aggAccount);

        UserOperation memory userOp2 = defaultOp;
        userOp2.sender = address(aggAccount2);

        UserOperation memory userOp_agg3 = defaultOp;
        userOp_agg3.sender = address(aggAccount3);
        userOp_agg3Arr[0] = utils.signUserOp(userOp_agg3, accountOwner.key, entryPointAddress, chainId);

        UserOperation memory userOp_noAgg = defaultOp;
        userOp_noAgg.sender = accountAddress;
        userOp_noAggArr[0] = utils.signUserOp(userOp_noAgg, accountOwner.key, entryPointAddress, chainId);

        userOp1.signature = aggregator.validateUserOpSignature(userOp1);
        userOp2.signature = aggregator.validateUserOpSignature(userOp2);
        userOpArr[0] = userOp1;
        userOpArr[1] = userOp2;

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](3);
        opsPerAggregator[0] =
            IEntryPoint.UserOpsPerAggregator(userOpArr, aggregator, aggregator.aggregateSignatures(userOpArr));
        opsPerAggregator[1] =
            IEntryPoint.UserOpsPerAggregator(userOp_agg3Arr, aggregator3, abi.encodePacked(bytes32(0)));
        opsPerAggregator[2] = IEntryPoint.UserOpsPerAggregator(userOp_noAggArr, IAggregator(address(0)), defaultBytes);

        vm.recordLogs();
        entryPoint.handleAggregatedOps{gas: 3e6}(opsPerAggregator, payable(utils.createAddress("beneficiary").addr));
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(address(uint160(uint256(logs[5].topics[1]))), address(aggregator));
        assertEq(address(uint160(uint256(logs[6].topics[2]))), userOp1.sender);
        assertEq(address(uint160(uint256(logs[7].topics[2]))), userOp2.sender);
        assertEq(address(uint160(uint256(logs[8].topics[1]))), address(aggregator3));
        assertEq(address(uint160(uint256(logs[9].topics[2]))), userOp_agg3.sender);
        assertEq(address(uint160(uint256(logs[10].topics[1]))), address(0));
        assertEq(address(uint160(uint256(logs[11].topics[2]))), userOp_noAgg.sender);
        assertEq(address(uint160(uint256(logs[12].topics[1]))), address(0));
    }

    // execution ordering
    //simulateValidation should return aggregator and its stake
    function test_AggregatorAndStakeReturned() public {
        (TestSignatureAggregator aggregator, UserOperation memory userOp,) = _executionOrderingSetup();

        aggregator.addStake{value: 2 ether}(entryPoint, 3);

        try entryPoint.simulateValidation(userOp) {}
        catch (bytes memory reason) {
            (bytes4 sig, bytes memory data) = utils.getDataFromEncoding(reason);
            (,,,, AggregatorStakeInfo memory aggStakeInfo) =
                abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo, AggregatorStakeInfo));
            assertEq(
                sig,
                bytes4(
                    keccak256(
                        "ValidationResultWithAggregation((uint256,uint256,bool,uint48,uint48,bytes),(uint256,uint256),(uint256,uint256),(uint256,uint256),(address,(uint256,uint256)))"
                    )
                )
            );
            assertEq(aggStakeInfo.aggregator, address(aggregator));
            assertEq(aggStakeInfo.stakeInfo.stake, 2 ether);
            assertEq(aggStakeInfo.stakeInfo.unstakeDelaySec, 3);
        }
    }

    //should create account in handleOps
    function test_AggregatorCreateAccount() public {
        (TestSignatureAggregator aggregator, UserOperation memory userOp, address payable beneficiary) =
            _executionOrderingSetup();

        // returns default bytes, but not used
        aggregator.validateUserOpSignature(userOp);
        ops.push(userOp);
        bytes memory sig = aggregator.aggregateSignatures(ops);

        IEntryPoint.UserOpsPerAggregator[] memory opsPerAggregator = new IEntryPoint.UserOpsPerAggregator[](1);
        ops[0].signature = defaultBytes;
        opsPerAggregator[0] = IEntryPoint.UserOpsPerAggregator(ops, aggregator, sig);

        entryPoint.handleAggregatedOps{gas: 3e6}(opsPerAggregator, beneficiary);
    }

    // with paymaster (account with no eth)
    //should fail with nonexistent paymaster
    function test_NonExistentPaymaster() public {
        (Account memory accountOwner2,, bytes memory accountExecFromEntryPoint) = _withPaymasterSetUp();
        uint256 salt = 123;
        address pm = utils.createAddress("paymaster").addr;

        UserOperation memory op = defaultOp;
        op.sender = simpleAccountFactory.getAddress(accountOwner2.addr, salt);
        op.paymasterAndData = abi.encodePacked(pm);
        op.callData = accountExecFromEntryPoint;
        op.initCode = utils.getAccountInitCode(accountOwner2.addr, simpleAccountFactory, salt);
        op.verificationGasLimit = 3e6;
        op.callGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner2.key, entryPointAddress, chainId);

        vm.expectRevert(utils.failedOp(0, "AA30 paymaster not deployed"));
        entryPoint.simulateValidation(op);
    }

    //should fail if paymaster has no deposit
    function test_PaymasterWithNoDeposit() public {
        (Account memory accountOwner2, TestPaymasterAcceptAll paymaster, bytes memory accountExecFromEntryPoint) =
            _withPaymasterSetUp();
        uint256 salt = 123;

        UserOperation memory op = defaultOp;
        op.sender = simpleAccountFactory.getAddress(accountOwner2.addr, salt);
        op.paymasterAndData = abi.encodePacked(address(paymaster));
        op.callData = accountExecFromEntryPoint;
        op.initCode = utils.getAccountInitCode(accountOwner2.addr, simpleAccountFactory, salt);
        op.verificationGasLimit = 3e6;
        op.callGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner2.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        vm.expectRevert(utils.failedOp(0, "AA31 paymaster deposit too low"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //paymaster should pay for tx
    function test_PaymasterPaysForTransaction() public {
        (Account memory accountOwner2, TestPaymasterAcceptAll paymaster, bytes memory accountExecFromEntryPoint) =
            _withPaymasterSetUp();
        uint256 salt = 123;
        paymaster.deposit{value: 1 ether}();

        UserOperation memory op = defaultOp;
        op.sender = simpleAccountFactory.getAddress(accountOwner2.addr, salt);
        op.paymasterAndData = abi.encodePacked(address(paymaster));
        op.callData = accountExecFromEntryPoint;
        op.initCode = utils.getAccountInitCode(accountOwner2.addr, simpleAccountFactory, salt);
        op.verificationGasLimit = 1e6;
        op = utils.signUserOp(op, accountOwner2.key, entryPointAddress, chainId);
        ops.push(op);
        address payable beneficiary = payable(utils.createAddress("beneficiary").addr);

        vm.recordLogs();
        entryPoint.handleOps(ops, beneficiary);

        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, uint256 actualGasCost,) = abi.decode(entries[5].data, (uint256, bool, uint256, uint256));
        assertEq(beneficiary.balance, actualGasCost);
        uint256 paymasterPaid = 1 ether - entryPoint.balanceOf(address(paymaster));
        assertEq(paymasterPaid, actualGasCost);
    }

    // simulateValidation should return paymaster stake and delay
    function test_ReturnPaymasterStakeInfo() public {
        (, TestPaymasterAcceptAll paymaster, bytes memory accountExecFromEntryPoint) = _withPaymasterSetUp();
        uint256 salt = 123;
        paymaster.deposit{value: 1 ether}();
        Account memory anOwner = utils.createAddress("anOwner");

        UserOperation memory op = defaultOp;
        op.sender = simpleAccountFactory.getAddress(anOwner.addr, salt);
        op.paymasterAndData = abi.encodePacked(address(paymaster));
        op.callData = accountExecFromEntryPoint;
        op.initCode = utils.getAccountInitCode(anOwner.addr, simpleAccountFactory, salt);
        op.verificationGasLimit = 1e6;
        op = utils.signUserOp(op, anOwner.key, entryPointAddress, chainId);

        StakeInfo memory paymasterInfo;
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (,,, paymasterInfo) = abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));
        }

        uint256 simRetStake = paymasterInfo.stake;
        uint256 simRetDelay = paymasterInfo.unstakeDelaySec;

        assertEq(simRetStake, paymasterStake);
        assertEq(simRetDelay, globalUnstakeDelaySec);
    }

    //validateUserOp time-range
    //should accept non-expired owner
    function test_AcceptNonExpiredOwner() public {
        (uint256 _now,, TestExpiryAccount expAccount, Account memory sessionOwner) = _validationTimeRangeSetUp();

        UserOperation memory op = defaultOp;
        op.sender = address(expAccount);
        op = utils.signUserOp(op, sessionOwner.key, entryPointAddress, chainId);

        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfoFromRevert,,,) =
                abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));

            assertEq(returnInfoFromRevert.validUntil, _now + 60);
            assertEq(returnInfoFromRevert.validAfter, 100);
        }
    }

    //should not reject expired owner
    function test_ShouldNotRejectExpiredOwner() public {
        (uint256 _now,, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();

        Account memory expiredOwner = utils.createAddress("expiredOwner");
        vm.prank(accountOwner.addr);
        expAccount.addTemporaryOwner(expiredOwner.addr, 123, uint48(_now - 60));

        UserOperation memory op = defaultOp;
        op.sender = address(expAccount);
        op = utils.signUserOp(op, expiredOwner.key, entryPointAddress, chainId);

        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            require(revertReason.length >= 4);
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfoFromRevert,,,) =
                abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));

            assertEq(returnInfoFromRevert.validUntil, _now - 60);
            assertEq(returnInfoFromRevert.validAfter, 123);
        }
    }

    //should accept non-expired paymaster request
    function test_AcceptNonExpiredPaymasterRequest() public {
        (uint256 _now,, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();
        (TestExpirePaymaster paymaster) = _validatePaymasterSetUp();

        //timeRange directly sent to the helper function
        UserOperation memory op =
            createOpWithPaymasterParams(address(expAccount), address(paymaster), 123, uint48(_now + 60), accountOwner);

        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfoFromRevert,,,) =
                abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));

            assertEq(returnInfoFromRevert.validUntil, _now + 60);
            assertEq(returnInfoFromRevert.validAfter, 123);
        }
    }

    //should not reject expired paymaster request
    function test_DontRejectExpiredPaymasterRequest() public {
        (uint256 _now,, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();
        (TestExpirePaymaster paymaster) = _validatePaymasterSetUp();

        //timeRange directly sent to the helper function
        UserOperation memory op =
            createOpWithPaymasterParams(address(expAccount), address(paymaster), 321, uint48(_now - 60), accountOwner);

        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfoFromRevert,,,) =
                abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));
            assertEq(returnInfoFromRevert.validUntil, _now - 60);
            assertEq(returnInfoFromRevert.validAfter, 321);
        }
    }

    //should use lower "after" value of paymaster
    function test_UseAfterOfPaymaster() public {
        (ReturnInfo memory ret) = simulateWithPaymasterParams(10, 1000);
        assertEq(ret.validAfter, 100);
    }

    //should use lower "after" value of account
    function test_UseAfterOfAccount() public {
        (ReturnInfo memory ret) = simulateWithPaymasterParams(200, 1000);
        assertEq(ret.validAfter, 200);
    }

    //should use higher "until" value of paymaster
    function test_UseUntilOfPaymaster() public {
        (ReturnInfo memory ret) = simulateWithPaymasterParams(10, 400);
        assertEq(ret.validUntil, 400);
    }

    //should use higher "until" value of account
    function test_UseUntilOfAccount() public {
        (ReturnInfo memory ret) = simulateWithPaymasterParams(200, 600);
        assertEq(ret.validUntil, 500);
    }

    //handleOps should revert on expired paymaster request
    function test_RevertExpiredPaymasterRequest() public {
        (uint256 _now, address payable beneficiary, TestExpiryAccount expAccount, Account memory sessionOwner) =
            _validationTimeRangeSetUp();
        (TestExpirePaymaster paymaster) = _validatePaymasterSetUp();

        UserOperation memory op = createOpWithPaymasterParams(
            address(expAccount), address(paymaster), uint48(_now + 100), uint48(_now + 200), sessionOwner
        );
        ops.push(op);

        vm.expectRevert(utils.failedOp(0, "AA32 paymaster expired or not due"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //handleOps should abort on time-range
    //should revert on expired account
    function test_RevertOnExpiredAccount() public {
        (, address payable beneficiary, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();

        Account memory expiredOwner = utils.createAddress("expiredOwner");
        vm.prank(accountOwner.addr);
        expAccount.addTemporaryOwner(expiredOwner.addr, 1, 2);

        UserOperation memory op = defaultOp;
        op.sender = address(expAccount);
        op = utils.signUserOp(op, expiredOwner.key, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(utils.failedOp(0, "AA22 expired or not due"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //should revert on date owner
    function test_RevertDateOwner() public {
        (uint256 _now, address payable beneficiary, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();

        Account memory futureOwner = utils.createAddress("futureOwner");
        vm.prank(accountOwner.addr);
        expAccount.addTemporaryOwner(futureOwner.addr, uint48(_now + 100), uint48(_now + 200));

        UserOperation memory op = defaultOp;
        op.sender = address(expAccount);
        op = utils.signUserOp(op, futureOwner.key, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(utils.failedOp(0, "AA22 expired or not due"));
        entryPoint.handleOps(ops, beneficiary);
    }

    function createOpWithPaymasterParams(
        address _accountAddr,
        address _paymasterAddr,
        uint48 _after,
        uint48 _until,
        Account memory owner
    ) public view returns (UserOperation memory op) {
        bytes memory timeRange = abi.encode(_after, _until);

        op = defaultOp;
        op.sender = _accountAddr;
        op.paymasterAndData = abi.encodePacked(_paymasterAddr, timeRange);
        op = utils.signUserOp(op, owner.key, entryPointAddress, chainId);
    }

    // Stake Management
    // With stake of 2 eth
    function _withStakeOf2EthSetup() private {
        // accountOwner address is used in place ethers.signer address
        vm.deal(accountOwner.addr, 10 ether);
        entryPoint.addStake{value: 2 ether}(2);
    }

    // with unlocked stake
    function _withUnlockedStakeSetup() private {
        _withStakeOf2EthSetup();
        entryPoint.unlockStake();
    }

    // after unstake delay
    function _afterUnstakeDelaySetup() private {
        _withUnlockedStakeSetup();
        vm.warp(uint48(block.timestamp + 2));
        payable(accountOwner.addr).transfer(0);
    }

    // Set up
    // 2d nonce
    function _2dNonceSetup(bool triggerHandelOps) internal returns (Account memory, uint256, uint256, address) {
        Account memory beneficiary = utils.createAddress("beneficiary");
        uint256 key = 1;
        uint256 keyShifted = key * 2 ** 64;

        (, address _accountAddress) = createAccountWithFactory(123422);
        vm.deal(_accountAddress, 1 ether);

        if (!triggerHandelOps) {
            return (beneficiary, key, keyShifted, _accountAddress);
        }
        UserOperation memory op = defaultOp;
        op.sender = _accountAddress;
        op.nonce = keyShifted;
        op = utils.signUserOp(op, accountOwner.key, entryPointAddress, chainId);
        ops.push(op);

        entryPoint.handleOps(ops, payable(beneficiary.addr));
        return (beneficiary, key, keyShifted, _accountAddress);
    }

    //without paymaster (account pays in eth)
    //#handleOps
    function _handleOpsSetUp() public returns (TestCounter counter, bytes memory accountExecFromEntryPoint) {
        counter = new TestCounter{salt: bytes32(uint256(123))}();
        bytes memory count = abi.encodeWithSignature("count()");
        accountExecFromEntryPoint =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, count);
    }

    //aggregation tests
    function _aggregationTestsSetUp()
        public
        returns (
            address payable beneficiary,
            TestSignatureAggregator aggregator,
            TestAggregatedAccount aggAccount,
            TestAggregatedAccount aggAccount2
        )
    {
        beneficiary = payable(utils.createAddress("beneficiary").addr);
        aggregator = new TestSignatureAggregator();
        aggAccount = new TestAggregatedAccount(entryPoint, address(aggregator));
        aggAccount2 = new TestAggregatedAccount(entryPoint, address(aggregator));
        vm.deal(address(aggAccount), 0.1 ether);
        vm.deal(address(aggAccount2), 0.1 ether);
    }

    //execution ordering
    function _executionOrderingSetup()
        public
        returns (TestSignatureAggregator aggregator, UserOperation memory userOp, address payable beneficiary)
    {
        (beneficiary, aggregator,,) = _aggregationTestsSetUp();

        // this setup sarts from context create account
        TestAggregatedAccountFactory factory = new TestAggregatedAccountFactory(entryPoint, address(aggregator));
        bytes memory _initCallData = abi.encodeWithSignature("createAccount(address,uint256)", address(0), 0);
        bytes memory initCode = abi.encodePacked(address(factory), _initCallData);
        address addr;
        try entryPoint.getSenderAddress(initCode) {}
        catch (bytes memory reason) {
            (, bytes memory data) = utils.getDataFromEncoding(reason);
            addr = abi.decode(data, (address));
        }
        vm.deal(addr, 0.1 ether);

        userOp = defaultOp;
        userOp.sender = addr;
        userOp.verificationGasLimit = 1e6;
        userOp.initCode = initCode;
        userOp = utils.signUserOp(userOp, accountOwner.key, entryPointAddress, chainId);
    }

    // With paymaster (account with no eth)
    function _withPaymasterSetUp()
        public
        returns (Account memory accountOwner2, TestPaymasterAcceptAll paymaster, bytes memory accountExecFromEntryPoint)
    {
        accountOwner2 = utils.createAddress("accountOwner2");
        vm.deal(accountOwner.addr, 10 ether);
        vm.startPrank(accountOwner.addr, accountOwner.addr);
        paymaster = new TestPaymasterAcceptAll(entryPoint);
        paymaster.addStake{value: paymasterStake}(uint32(globalUnstakeDelaySec));
        vm.stopPrank();
        TestCounter counter = new TestCounter();
        bytes memory count = abi.encodeWithSignature("count()");
        accountExecFromEntryPoint =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(counter), 0, count);
    }

    //Validation time-range
    function _validationTimeRangeSetUp()
        public
        returns (uint256 _now, address payable beneficiary, TestExpiryAccount expAccount, Account memory sessionOwner)
    {
        beneficiary = payable(makeAddr("beneficiary"));
        vm.deal(accountOwner.addr, 1000 ether);
        vm.startPrank(accountOwner.addr);
        // the account variable in hardhat tests is renamed as expAccount, so as to not confuse with the default `account` creates in TestHelper
        expAccount = new TestExpiryAccount(entryPoint);
        expAccount.initialize(accountOwner.addr);
        vm.deal(address(expAccount), 0.1 ether);
        vm.warp(1641070800);
        _now = block.timestamp;
        sessionOwner = utils.createAddress("sessionOwner");
        expAccount.addTemporaryOwner(sessionOwner.addr, 100, uint48(_now + 60));
        vm.stopPrank();
    }

    //validatePaymasterUserOp with deadline
    function _validatePaymasterSetUp() public returns (TestExpirePaymaster paymaster) {
        // not implementing the timeout feature
        paymaster = new TestExpirePaymaster(entryPoint);
        paymaster.addStake{value: paymasterStake}(1);
        paymaster.deposit{value: 0.1 ether}();
        // using _now created in the validation time range setup
    }

    //time-range overlap of paymaster and account should intersect
    //this function contains the setup and helper functions from hardhat tests
    function simulateWithPaymasterParams(uint48 _after, uint48 _until) public returns (ReturnInfo memory ret) {
        (,, TestExpiryAccount expAccount,) = _validationTimeRangeSetUp();
        (TestExpirePaymaster paymaster) = _validatePaymasterSetUp();

        // before
        Account memory owner = utils.createAddress("owner");
        vm.prank(accountOwner.addr);
        expAccount.addTemporaryOwner(owner.addr, 100, 500);

        // createOpWithPaymasterParams
        UserOperation memory op =
            createOpWithPaymasterParams(address(expAccount), address(paymaster), _after, _until, owner);

        // simulateWithPaymasterParams
        try entryPoint.simulateValidation(op) {}
        catch (bytes memory revertReason) {
            (, bytes memory data) = utils.getDataFromEncoding(revertReason);
            (ret,,,) = abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));
        }
    }
}
