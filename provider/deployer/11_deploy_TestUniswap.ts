import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import { ethers } from 'hardhat'

const deployTestUniswap: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const provider = ethers.provider
  const from = await provider.getSigner().getAddress()
  const network = await provider.getNetwork()
  // only deploy on local test network.
  if (network.chainId !== 31337 && network.chainId !== 1337) {
    return
  }

  const testWrappedNativeToken = await hre.deployments.get('TestWrappedNativeToken')
  const ret = await hre.deployments.deploy(
    'TestUniswap', {
      from,
      args: [testWrappedNativeToken.address],
      gasLimit: 6e6,
      log: true,
      deterministicDeployment: true
    })
  console.log('==TestUniswap addr=', ret.address)
}

export default deployTestUniswap
