// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/TokenPaymaster.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/tests/TestErc20.sol";
import "../src/tests/TestUniswap.sol";
import "../src/tests/TestOracle2.sol";
import "../src/tests/TestWrappedNativeToken.sol";


contract TokenPaymasterTest is Test {
    Account private owner = makeAccount("owner");
    TestERC20 private token;
    EntryPoint private entryPoint;
    TestUniswap private uniswap;
    TestOracle2 private tokenOracle;
    TestOracle2 private nativeAssetOracle;
    SimpleAccount private wallet;
    TokenPaymaster private paymaster;
    SimpleAccountFactory private factory;
    TestWrappedNativeToken private weth;

    uint256 private chainId = vm.envUint('FOUNDRY_CHAIN_ID');
    int256 private initialPriceEther = 500000000;
    int256 private initialPriceToken = 100000000;
    address payable private walletAddress;
    address private tokenAddress;
    address private epAddress;

    function setUp() public {
        entryPoint = new EntryPoint();
        epAddress = payable(entryPoint);

        weth = new TestWrappedNativeToken();
        uniswap = new TestUniswap(weth);

        factory = new SimpleAccountFactory(entryPoint);
        wallet = factory.createAccount(owner.addr, 1);
        walletAddress = payable(wallet);

        vm.deal(walletAddress, 1 ether);
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
        UniswapHelper.UniswapHelperConfig memory uniswapConfig = UniswapHelper.UniswapHelperConfig({
            minSwapAmount: 1,
            uniswapPoolFee: 3,
            slippage: 5
        });

        paymaster = new TokenPaymaster(
            token,
            entryPoint,
            weth,
            uniswap,
            paymasterConfig,
            oracleConfig,
            uniswapConfig,
            owner.addr);

        vm.startPrank(owner.addr);
        token.transfer(address(paymaster), 100);
        vm.warp(1680509051);
        paymaster.updateCachedPrice(true);
        entryPoint.depositTo{value: 1000 ether}(address(paymaster));
        paymaster.addStake{value: 2 ether}(1);
        vm.stopPrank();
    }
}
