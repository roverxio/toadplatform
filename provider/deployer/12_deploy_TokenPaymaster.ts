import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import { ethers } from 'hardhat'
import { BigNumber } from 'ethers'
import {
  OracleHelper as OracleHelperNamespace, TokenPaymaster,
  UniswapHelper as UniswapHelperNamespace
} from '../typechain/contracts/samples/TokenPaymaster'

const deployTokenPaymaster: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const provider = ethers.provider
  const from = await provider.getSigner().getAddress()
  const network = await provider.getNetwork()
  // only deploy on local test network.
  if (network.chainId !== 31337 && network.chainId !== 1337) {
    return
  }

  const entrypoint = await hre.deployments.get('EntryPoint')
  const erc20 = await hre.deployments.get('TestERC20')
  const uniswap = await hre.deployments.get('TestUniswap')
  const testOracle = await hre.deployments.get('TestOracle2')
  const tokenPaymasterConfig: TokenPaymaster.TokenPaymasterConfigStruct = {
    priceMaxAge: 86400,
    refundPostopCost: 4000,
    minEntryPointBalance: 1e17.toString(),
    priceMarkup: BigNumber.from(10).pow(26).mul(15).div(10)
  }
  const oracleHelperConfig: OracleHelperNamespace.OracleHelperConfigStruct = {
    cacheTimeToLive: 0,
    nativeOracle: testOracle.address,
    nativeOracleReverse: false,
    priceUpdateThreshold: 200_000, // +20%
    tokenOracle: testOracle.address,
    tokenOracleReverse: false,
    tokenToNativeOracle: false
  }
  const uniswapHelperConfig: UniswapHelperNamespace.UniswapHelperConfigStruct = {
    minSwapAmount: 1,
    slippage: 5,
    uniswapPoolFee: 3
  }
  const ret = await hre.deployments.deploy(
    'TokenPaymaster', {
      from,
      args: [erc20.address, entrypoint.address, erc20.address, uniswap.address, tokenPaymasterConfig, oracleHelperConfig, uniswapHelperConfig, from],
      gasLimit: 6e6,
      log: true,
      deterministicDeployment: true
    })
  console.log('==TokenPaymaster addr=', ret.address)
}

export default deployTokenPaymaster
