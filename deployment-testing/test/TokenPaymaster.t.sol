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
    TestERC20 private _token;
    TestUniswap private _uniswap;
    TestOracle2 private _tokenOracle;
    TokenPaymaster private _paymaster;
    TestWrappedNativeToken private _weth;
    TestOracle2 private _nativeAssetOracle;

    address internal _paymasterAddress;

    int256 private _initialPriceEther = 500000000;
    int256 private _initialPriceToken = 100000000;
    address private _tokenAddress;

    function setUp() public {
        _createAddress("owner_paymaster");
        _deployEntryPoint(123461);
        _createAccount(123462, 123463);

        _weth = new TestWrappedNativeToken();
        _uniswap = new TestUniswap(_weth);

        vm.deal(_accountAddress, 1 ether);
        vm.deal(_owner.addr, 1003 ether);
        // Check for geth

        vm.startPrank(_owner.addr);

        _token = new TestERC20(6);
        _tokenAddress = address(_token);
        _nativeAssetOracle = new TestOracle2(_initialPriceEther, 8);
        _tokenOracle = new TestOracle2(_initialPriceToken, 8);

        _weth.deposit{value: 1 ether}();
        _weth.transfer(address(_uniswap), 1 ether);
        vm.stopPrank();

        TokenPaymaster.TokenPaymasterConfig memory paymasterConfig = TokenPaymaster.TokenPaymasterConfig({
            priceMarkup: 1e26 * 15 / 10,
            minEntryPointBalance: 0.1 ether,
            refundPostopCost: 40000,
            priceMaxAge: 86400
        });
        OracleHelper.OracleHelperConfig memory oracleConfig = OracleHelper.OracleHelperConfig({
            tokenOracle: _tokenOracle,
            nativeOracle: _nativeAssetOracle,
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

        _paymaster = new TokenPaymaster(
            _token,
            _entryPoint,
            _weth,
            _uniswap,
            paymasterConfig,
            oracleConfig,
            uniswapConfig,
            _owner.addr);
        _paymasterAddress = address(_paymaster);

        vm.startPrank(_owner.addr);
        _token.transfer(_paymasterAddress, 100);
        vm.warp(1680509051);
        _paymaster.updateCachedPrice(true);
        _entryPoint.depositTo{value: 1000 ether}(_paymasterAddress);
        _paymaster.addStake{value: 2 ether}(1);
        vm.stopPrank();
    }

    function testNoTokensOrAllowance() public {
        uint256 snapShotId = vm.snapshot();
        /*
        - Generate Paymaster Data
        - Generate user op
        - Sign user op
        - expect revert on handle op with insufficient allowance
        - Approve
        - Expect revert on handle op with no balance
        */
        vm.revertTo(snapShotId);
    }
}
