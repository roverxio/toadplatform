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
    address private beneficiaryAddress = 0x1111111111111111111111111111111111111111;
    UserOperation[] public ops;

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
            priceMarkup: 1e26 * 15 / 10,
            minEntryPointBalance: 0.1 ether,
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
        vm.warp(1680509051);
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
        entryPoint.handleOps{gas: 1e7}(ops, payable(beneficiaryAddress));

        token.sudoApprove(accountAddress, paymasterAddress, type(uint256).max);
        vm.expectRevert(
            abi.encodeWithSignature(
                "FailedOp(uint256,string)", 0, "AA33 reverted: ERC20: transfer amount exceeds balance"
            )
        );
        entryPoint.handleOps{gas: 1e7}(ops, payable(beneficiaryAddress));
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
        vm.txGasPrice(op.maxFeePerGas);
        // Gas price calculation
        vm.recordLogs();
        entryPoint.handleOps{gas: 1e7}(ops, payable(beneficiaryAddress));
        Vm.Log[] memory logs = vm.getRecordedLogs();
        uint256 preChargeTokens = abi.decode(logs[0].data, (uint256));
        uint256 refundTokens = abi.decode(logs[2].data, (uint256));
        uint256 actualTokenChargeEvents = preChargeTokens - refundTokens;
        (uint256 actualTokenCharge, uint256 actualGasCostPaymaster, uint256 actualTokenPrice) =
            abi.decode(logs[3].data, (uint256, uint256, uint256));
        (, bool status, uint256 actualGasCostEntryPoint,) = abi.decode(logs[4].data, (uint256, bool, uint256, uint256));
        int256 expectedTokenPrice = initialPriceToken / initialPriceEther;
        uint256 addedPostOpCost = op.maxFeePerGas * 40000;
        int256 expectedTokenPriceWithMarkup = (((1e26 * initialPriceToken) / initialPriceEther) * 10) / 15;
        uint256 expectedTokenCharge =
            ((actualGasCostPaymaster + addedPostOpCost) * 1e26) / uint256(expectedTokenPriceWithMarkup);
        uint256 postOpGasCost = actualGasCostEntryPoint - actualGasCostPaymaster;
        int256 calculatedTokenPrice = int256(actualTokenPrice) / 1e26;

        assertEq(logs.length, 5);
        assertEq(status, true);
        assertEq(actualTokenChargeEvents, actualTokenCharge);
        assertEq(actualTokenChargeEvents, expectedTokenCharge);
        assertEq(calculatedTokenPrice, expectedTokenPrice);
        // assert.closeTo(postOpGasCost.div(tx.effectiveGasPrice).toNumber(), 40000, 20000)

        vm.stopPrank();
        vm.revertTo(snapShotId);
    }

    function _generatePaymasterData(address _pmAddress, uint256 tokenPrice) internal pure returns (bytes memory) {
        if (tokenPrice == 0) {
            return abi.encodePacked(_pmAddress);
        } else {
            return abi.encodePacked(_pmAddress, tokenPrice);
        }
    }
}

// emit Transfer(from: ERC1967Proxy: [0x8e1A239c2C2238dD277A360f20eE8520F9Ff6e81], to: TokenPaymaster: [0x5991A2dF15A8F6A256D3Ec51E99254Cd3fb576A9], value: 12622500000000000 [1.262e16])
// emit BeforeExecution()
// emit Transfer(from: TokenPaymaster: [0x5991A2dF15A8F6A256D3Ec51E99254Cd3fb576A9], to: ERC1967Proxy: [0x8e1A239c2C2238dD277A360f20eE8520F9Ff6e81], value: 12622499998957133 [1.262e16])
// emit UserOperationSponsored(user: ERC1967Proxy: [0x8e1A239c2C2238dD277A360f20eE8520F9Ff6e81], actualTokenCharge: 1042867 [1.042e6], actualGasCost: 99049 [9.904e4], actualTokenPrice: 20000000000000000000000000 [2e25])
// emit UserOperationEvent(userOpHash: 0xbe0467ddf1355d76635cdaa220bf62b50ef5287b754c1af2992a55495077f442, sender: ERC1967Proxy: [0x8e1A239c2C2238dD277A360f20eE8520F9Ff6e81], paymaster: TokenPaymaster: [0x5991A2dF15A8F6A256D3Ec51E99254Cd3fb576A9], nonce: 0, success: false, actualGasCost: 114473 [1.144e5], actualGasUsed: 114473 [1.144e5])

// event Transfer(address indexed from, address indexed to, uint256 value);
// event BeforeExecution();
// event Transfer(address indexed from, address indexed to, uint256 value);
// event UserOperationSponsored(address indexed user, uint256 actualTokenCharge, uint256 actualGasCost, uint256 actualTokenPrice);
// event UserOperationEvent(bytes32 indexed userOpHash, address indexed sender, address indexed paymaster, uint256 nonce, bool success, uint256 actualGasCost, uint256 actualGasUsed);
