// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/TokenPaymaster.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/tests/TestErc20.sol";
import "../src/tests/TestUniswap.sol";
import "../src/tests/TestOracle2.sol";
import "../src/tests/TestWrappedNativeToken.sol";

contract TokenPaymasterTest is TestHelper {
    TestERC20 private token;
    TestUniswap private uniswap;
    TestOracle2 private tokenOracle;
    TokenPaymaster private paymaster;
    TestWrappedNativeToken private weth;
    TestOracle2 private nativeAssetOracle;

    address internal paymasterAddress;
    address private tokenAddress;

    int256 private initialPriceEther = 500000000;
    int256 private initialPriceToken = 100000000;
    bytes private callData;
    address payable private beneficiaryAddress = payable(0x1111111111111111111111111111111111111111);
    UserOperation[] public ops;
    uint256 private priceDenominator = 1e26;
    uint256 private minEntryPointBalance = 1e17;
    uint256 private blockTime = 1680509051;

    event TokenPriceUpdated(uint256 currentPrice, uint256 previousPrice, uint256 cachedPriceTimestamp);
    event UserOperationSponsored(
        address indexed user, uint256 actualTokenCharge, uint256 actualGasCost, uint256 actualTokenPrice
    );

    function setUp() public {
        createAddress("owner_paymaster");
        deployEntryPoint(123461);
        createAccount(123462, 123463);

        weth = new TestWrappedNativeToken();
        uniswap = new TestUniswap(weth);

        vm.deal(accountAddress, 1 ether);
        vm.deal(owner.addr, 1003 ether);
        // Check for geth

        vm.startPrank(owner.addr);

        token = new TestERC20(6);
        tokenAddress = address(token);
        nativeAssetOracle = new TestOracle2(initialPriceEther, 8);
        tokenOracle = new TestOracle2(initialPriceToken, 8);

        weth.deposit{value: 1 ether}();
        weth.transfer(address(uniswap), 1 ether);
        vm.stopPrank();

        TokenPaymaster.TokenPaymasterConfig memory paymasterConfig = TokenPaymaster.TokenPaymasterConfig({
            priceMarkup: priceDenominator * 15 / 10,
            minEntryPointBalance: minEntryPointBalance,
            refundPostopCost: 40000,
            priceMaxAge: 86400
        });
        OracleHelper.OracleHelperConfig memory oracleConfig = OracleHelper.OracleHelperConfig({
            tokenOracle: tokenOracle,
            nativeOracle: nativeAssetOracle,
            tokenToNativeOracle: false,
            tokenOracleReverse: false,
            nativeOracleReverse: false,
            priceUpdateThreshold: 200_000,
            cacheTimeToLive: 0
        });
        UniswapHelper.UniswapHelperConfig memory uniswapConfig =
            UniswapHelper.UniswapHelperConfig({minSwapAmount: 1, uniswapPoolFee: 3, slippage: 5});

        paymaster = new TokenPaymaster(
            token,
            entryPoint,
            weth,
            uniswap,
            paymasterConfig,
            oracleConfig,
            uniswapConfig,
            owner.addr);
        paymasterAddress = address(paymaster);

        vm.startPrank(owner.addr);
        token.transfer(paymasterAddress, 100);
        vm.warp(blockTime);
        paymaster.updateCachedPrice(true);
        entryPoint.depositTo{value: 1000 ether}(paymasterAddress);
        paymaster.addStake{value: 2 ether}(1);
        vm.stopPrank();
        callData = abi.encodeWithSignature("execute(address,uint256,bytes)", owner.addr, 0, defaultBytes);
    }

    function testNoTokensOrAllowance() public {
        uint256 snapShotId = vm.snapshot();
        bytes memory paymasterData = _generatePaymasterData(paymasterAddress, 0);
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = paymasterData;
        op.callData = callData;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        vm.expectRevert(
            abi.encodeWithSignature("FailedOp(uint256,string)", 0, "AA33 reverted: ERC20: insufficient allowance")
        );
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);

        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);
        vm.expectRevert(
            abi.encodeWithSignature(
                "FailedOp(uint256,string)", 0, "AA33 reverted: ERC20: transfer amount exceeds balance"
            )
        );
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);
        vm.revertTo(snapShotId);
    }

    function test_UpdateCachedTokenPrice() public {
        uint256 snapShotId = vm.snapshot();
        vm.startPrank(owner.addr);
        vm.warp(blockTime + 10);
        token.transfer(accountAddress, 1 ether);
        token.sudoApprove(accountAddress, address(paymaster), type(uint256).max);
        tokenOracle.setPrice(initialPriceToken * 5);
        nativeAssetOracle.setPrice(initialPriceEther * 10);

        bytes memory paymasterAndData = _generatePaymasterData(paymasterAddress, 0);
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = paymasterAndData;
        op.callData = callData;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        uint256 oldExpectedPrice = uint256(int256(priceDenominator) * initialPriceToken / initialPriceEther);
        uint256 newExpectedPrice = uint256(oldExpectedPrice / 2);

        vm.expectEmit(false, false, false, true);
        emit TokenPriceUpdated(newExpectedPrice, oldExpectedPrice, block.timestamp);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);

        Vm.Log[] memory logs = vm.getRecordedLogs();
        (,, uint256 actualTokenPrice) = abi.decode(logs[4].data, (uint256, uint256, uint256)); //Expected event is UserOperationSponsored
        assertEq(actualTokenPrice, newExpectedPrice);
        vm.stopPrank();
        vm.revertTo(snapShotId);
    }

    // Should be able to sponsor the UserOp while charging correct amount of ERC-20 tokens
    function test_SponsorErc20() public {
        uint256 snapShotId = vm.snapshot();
        vm.startPrank(owner.addr);

        token.transfer(accountAddress, 1 ether);
        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);
        bytes memory paymasterData = _generatePaymasterData(paymasterAddress, 0);
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.callGasLimit = 30754;
        op.verificationGasLimit = 150000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 1000000000;
        op.maxPriorityFeePerGas = 1000000000;
        op.paymasterAndData = paymasterData;
        op.callData = callData;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        // Gas price calculation
        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, payable(beneficiaryAddress));
        Vm.Log[] memory logs = vm.getRecordedLogs();
        uint256 actualTokenChargeEvents = abi.decode(logs[0].data, (uint256)) - abi.decode(logs[2].data, (uint256));
        (uint256 actualTokenCharge, uint256 actualGasCostPaymaster, uint256 actualTokenPrice) =
            abi.decode(logs[3].data, (uint256, uint256, uint256));
        (, bool status, uint256 actualGasCostEntryPoint,) = abi.decode(logs[4].data, (uint256, bool, uint256, uint256));
        int256 expectedTokenPriceWithMarkup =
            (((int256(priceDenominator) * initialPriceToken) / initialPriceEther) * 10) / 15;
        uint256 expectedTokenCharge = ((actualGasCostPaymaster + (op.maxFeePerGas * 40000)) * priceDenominator)
            / uint256(expectedTokenPriceWithMarkup);
        uint256 postOpGasCost = actualGasCostEntryPoint - actualGasCostPaymaster;
        console.logUint(postOpGasCost);

        assertEq(logs.length, 5);
        assertEq(status, true);
        assertEq(actualTokenChargeEvents, actualTokenCharge);
        assertEq(actualTokenChargeEvents, expectedTokenCharge);
        assertEq((int256(actualTokenPrice) / 1e26), (initialPriceToken / initialPriceEther));
        // TODO: Assert (postOpGasCost/tx.effectiveGasPrice) close to 40000 with 20000 delta

        vm.stopPrank();
        vm.revertTo(snapShotId);
    }

    function test_UseSuppliedPriceIfItsBetter() public {
        uint256 snapshotId = vm.snapshot();
        vm.startPrank(owner.addr);
        token.transfer(accountAddress, 1 ether);
        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);

        uint256 currentCachedPrice = paymaster.cachedPrice();
        assertEq((currentCachedPrice * 10) / priceDenominator, 2);
        uint256 overrideTokenPrice = (priceDenominator * 132) / 1000;
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = _generatePaymasterData(paymasterAddress, overrideTokenPrice);
        op.callData = callData;

        op.callGasLimit = 30754;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 1000000007;
        op.maxPriorityFeePerGas = 1000000000;

        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        // TODO: figure out the syntax to set base fee per gas for the next block

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        uint256 preChargeTokens = abi.decode(logs[0].data, (uint256)); // log[0] is a transfer event
        uint256 requiredGas = op.callGasLimit + (op.verificationGasLimit * 3) + op.preVerificationGas + 40000;
        uint256 requeiredPrefund = requiredGas * op.maxFeePerGas;
        uint256 preChargeTokenPrice = requeiredPrefund * priceDenominator / preChargeTokens;

        assertEq(preChargeTokenPrice / 1e10, overrideTokenPrice / 1e10);
        vm.stopPrank();
        vm.revertTo(snapshotId);
    }

    function test_UseCachedPriceIfItsBetter() public {
        uint256 snapshotId = vm.snapshot();
        vm.startPrank(owner.addr);
        token.transfer(accountAddress, 1 ether);
        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);

        uint256 currentCachedPrice = paymaster.cachedPrice();
        assertEq((currentCachedPrice * 10) / priceDenominator, 2);
        uint256 overrideTokenPrice = (priceDenominator * 50);
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = _generatePaymasterData(paymasterAddress, overrideTokenPrice);
        op.callData = callData;

        op.callGasLimit = 30754;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 1000000007;
        op.maxPriorityFeePerGas = 1000000000;

        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        // TODO: figure out the syntax to set base fee per gas for the next block

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);
        Vm.Log[] memory logs = vm.getRecordedLogs();

        uint256 preChargeTokens = abi.decode(logs[0].data, (uint256)); // log[0] is a transfer event
        uint256 requiredGas = op.callGasLimit + (op.verificationGasLimit * 3) + op.preVerificationGas + 40000;
        uint256 requeiredPrefund = requiredGas * op.maxFeePerGas;
        uint256 preChargeTokenPrice = requeiredPrefund * priceDenominator / preChargeTokens;

        assertEq(preChargeTokenPrice, (currentCachedPrice * 10) / 15);
        vm.stopPrank();
        vm.revertTo(snapshotId);
    }

    // should swap tokens for ether if it falls below configured value and deposit it
    function test_SwapEtherTokens() public {
        vm.startPrank(owner.addr);
        vm.deal(owner.addr, 1 ether);

        token.transfer(accountAddress, token.balanceOf(owner.addr));
        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);

        (uint112 deposit,,,,) = entryPoint.deposits(paymasterAddress);
        paymaster.withdrawTo(payable(accountAddress), deposit);
        entryPoint.depositTo{value: minEntryPointBalance}(paymasterAddress);

        bytes memory paymasterData = _generatePaymasterData(paymasterAddress, 0);
        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = paymasterData;
        op.callData = callData;
        op.callGasLimit = 30754;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 1000000007;
        op.maxPriorityFeePerGas = 1000000000;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, payable(beneficiaryAddress));
        Vm.Log[] memory logs = vm.getRecordedLogs();

        assertEq(logs[4].topics[0], keccak256("StubUniswapExchangeEvent(uint256,uint256,address,address)"));
        assertEq(logs[8].topics[0], keccak256("Received(address,uint256)"));
        assertEq(logs[9].topics[0], keccak256("Deposited(address,uint256)"));
        // TODO: validate deFactoExchangeRate with expectedPrice

        vm.stopPrank();
    }

    function test_chargeOverdraftIfPrechargeIsLowerThanTxnCost() public {
        uint256 snapshotId = vm.snapshot();
        vm.startPrank(owner.addr);
        token.transfer(accountAddress, token.balanceOf(owner.addr));
        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);

        tokenOracle.setPrice(initialPriceToken);
        nativeAssetOracle.setPrice(initialPriceEther * 100);

        vm.warp(blockTime + 200);

        UserOperation memory op = _defaultOp;
        op.sender = accountAddress;
        op.paymasterAndData = _generatePaymasterData(paymasterAddress, 0);
        op.callData = callData;
        op.callGasLimit = 30754;
        op.verificationGasLimit = 150000;
        op.maxFeePerGas = 1000000007;
        op.maxPriorityFeePerGas = 1000000000;
        op = signUserOp(op, entryPointAddress, chainId);
        ops.push(op);

        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, beneficiaryAddress);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        uint256 preChargeTokens = abi.decode(logs[0].data, (uint256)); // 0 is transfer event
        uint256 overdraftTokens = abi.decode(logs[3].data, (uint256)); // 3 is transfer event
        (uint256 actualTokenCharge,,) = abi.decode(logs[4].data, (uint256, uint256, uint256)); // 4 is UserOperationSponsored event
        (, bool success,,) = abi.decode(logs[5].data, (uint256, bool, uint256, uint256)); // 5 is UserOperationEvent event

        assertEq(logs[0].topics[1], logs[3].topics[1]);
        assertEq(logs[0].topics[2], logs[3].topics[2]);

        assertEq(preChargeTokens + overdraftTokens, actualTokenCharge);
        assertEq(success, true);

        vm.stopPrank();
        vm.revertTo(snapshotId);
    }

    function _generatePaymasterData(address _pmAddress, uint256 tokenPrice) internal pure returns (bytes memory) {
        if (tokenPrice == 0) {
            return abi.encodePacked(_pmAddress);
        } else {
            return abi.encodePacked(_pmAddress, tokenPrice);
        }
    }
}
