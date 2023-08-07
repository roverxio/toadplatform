// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccount.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/test/TestWarmColdAccount.sol";
import "../src/test/TestPaymasterAcceptAll.sol";
import "../src/test/TestRevertAccount.sol";
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

contract EntryPointTest is TestHelper {
    UserOperation[] internal ops;
    Utilities internal utils;

    function setUp() public {
        utils = new Utilities();
        accountOwner = utils.createAddress("entrypoint_Owner");
        deployEntryPoint(1101);
        createAccount(1102, 1103);
        vm.deal(accountAddress, 1 ether);
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

    // should succeed to stake again
    function test_SucceedToStakeAgain() public {
        // accountOwner address is used in place ethers.signer address
        vm.deal(accountOwner.addr, 10 ether);
        vm.startPrank(accountOwner.addr);
        // setup
        entryPoint.addStake{value: 2 ether}(2);

        uint112 stake = entryPoint.getDepositInfo(accountOwner.addr).stake;
        entryPoint.addStake{value: 1 ether}(2);
        uint112 stakeAfter = entryPoint.getDepositInfo(accountOwner.addr).stake;
        assertEq(stakeAfter, stake + 1 ether);
        vm.stopPrank();
    }

    // should fail to withdraw before unlock
    function test_FailToWithdrawBeforeUnlock() public {
        vm.deal(accountOwner.addr, 10 ether);
        vm.startPrank(accountOwner.addr);
        // setup
        entryPoint.addStake{value: 2 ether}(2);

        vm.expectRevert("must call unlockStake() first");
        entryPoint.withdrawStake(payable(address(0)));
        vm.stopPrank();
    }

    // with unlocked stake
    // should report as "not staked"
    function test_ReportAsNotStaked() public {
        vm.deal(accountOwner.addr, 10 ether);
        vm.startPrank(accountOwner.addr);
        // setup
        entryPoint.addStake{value: 2 ether}(2);
        entryPoint.unlockStake();

        assertEq(entryPoint.getDepositInfo(accountOwner.addr).staked, false);
        vm.stopPrank();
    }

    // With deposit
    // Should be able to withdraw
    function test_WithdrawDeposit() public {
        (SimpleAccount account1, address accountAddress1) = createAccountWithFactory(1104);
        account1.addDeposit{value: 1 ether}();

        assertEq(utils.getBalance(accountAddress1), 0);
        assertEq(account1.getDeposit(), 1 ether);

        vm.prank(accountOwner.addr);
        account1.withdrawDepositTo(payable(accountAddress1), 1 ether);

        assertEq(utils.getBalance(accountAddress1), 1 ether);
        assertEq(account1.getDeposit(), 0);
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
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
    function test_shouldSucceedifUserOpSucceeds() public {
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (, senderInfo,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
        }

        assertEq(senderInfo.stake, 123);
        assertEq(senderInfo.unstakeDelaySec, 3);
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
        vm.expectRevert(abi.encodeWithSignature("FailedOp(uint256,string)", 0, "AA14 initCode must return sender"));
        entryPoint.simulateValidation(op1);
    }

    /// @notice 10. Should report failure on insufficient verificationGas for creation
    function test_shouldReportFailureOnInsufficentVerificationGas() public {
        Account memory accountOwner1 = utils.createAddress("accountOwner1");
        address addr;
        bytes memory initCode = utils.getAccountInitCode(accountOwner1.addr, simpleAccountFactory, 0);

        try entryPoint.getSenderAddress(initCode) {}
        catch (bytes memory reason) {
            require(reason.length >= 4);
            bytes memory data = utils.getDataFromEncoding(reason);
            assembly {
                addr := mload(add(data, 0x20))
            }
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (returnInfo,,,) = abi.decode(
                data,
                (IEntryPoint.ReturnInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo, IStakeManager.StakeInfo)
            );
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
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (,,,, bool success, bytes memory result) = abi.decode(data, (uint256, uint256, uint48, uint48, bool, bytes));
            assertEq(success, true);
            assertEq(result, abi.encode(1));
        }
        assertEq(counter.counters(accountAddress1), 0);
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

    // Should fail with nonsequential seq
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

    // Flickering account validation
    // Should prevent leakage of basefee
    // Note: Not completing this test cases as it includes RPC calls
    function test_BaseFeeLeakage() public {
        /**
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
            bytes4 reason;
            assembly {
                reason := mload(add(errorReason, 32))
            }
            assertEq(reason, utils.validationResultEvent());
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
            bytes4 reason;
            assembly {
                reason := mload(add(revertReason, 32))
            }
            if (reason == utils.validationResultEvent()) {
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
            bytes4 reason;
            assembly {
                reason := mload(add(revertReason, 32))
            }
            if (reason == utils.validationResultEvent()) {
                ops.push(badOp);
                entryPoint.handleOps{gas: 1e6}(ops, payable(beneficiary.addr));
            } else {
                assertEq(revertReason, utils.failedOp(0, "AA23 reverted (or OOG)"));
            }
        }
    }

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
}
