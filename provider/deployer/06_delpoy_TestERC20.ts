import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import { ethers } from 'hardhat'

const deployTestERC20: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const provider = ethers.provider
  const from = await provider.getSigner().getAddress()
  const network = await provider.getNetwork()
  // only deploy on local test network.
  if (network.chainId !== 31337 && network.chainId !== 1337) {
    return
  }

  const ret = await hre.deployments.deploy(
    'TestERC20', {
      from,
      args: [18],
      gasLimit: 6e6,
      log: true,
      deterministicDeployment: true
    })
  console.log('==TestERC20 addr=', ret.address)
}

export default deployTestERC20
