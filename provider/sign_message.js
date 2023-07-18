const {ethers, utils} = require("ethers");
const {arrayify, keccak256} = require("ethers/lib/utils");

const privateKey = 'private_key';
const providerUrl = 'http://localhost:8545';
const provider = new ethers.providers.JsonRpcProvider(providerUrl);
const wallet = new ethers.Wallet(privateKey, provider);

const userOpType = ['address', 'uint256', 'bytes32', 'bytes32',
    'uint256', 'uint256', 'uint256', 'uint256', 'uint256',
    'bytes32']
const MOCK_VALID_UNTIL = '0x00000000deadbeef'
const MOCK_VALID_AFTER = '0x0000000000001234'
async function getVerifyingPaymasterData(paymaster) {
    let now = await provider.getBlock('latest').then((block) => block.timestamp)
    const timeRange = ethers.utils.defaultAbiCoder.encode(['uint48', 'uint48'], [MOCK_VALID_UNTIL, MOCK_VALID_AFTER])
    // return ethers.utils.hexConcat([paymaster, timeRange])
    console.log('timeRange: ' + timeRange.length)
    console.log('timeRange: ' + timeRange)
    return utils.hexConcat([paymaster, timeRange, '0x' + '00'.repeat(65)])
}

getVerifyingPaymasterData('<paymaster_address_or_0x>').then((data) => {

    let userOperation = {
        sender: '0x418fe523f35f6b094ffe3e8ed45db30840a24431',
        nonce: '0',
        initCode: '0x',
        callData: '<calldata>',
        callGasLimit: 10000000,
        verificationGasLimit: 10000000,
        preVerificationGas: 1000000000000000,
        maxFeePerGas: 5,
        maxPriorityFeePerGas: 1000000000,
        paymasterAndData: data,
        signature: '0x'
    }
    const message = arrayify(getUserOpHash(userOperation, '<entrypoint_address>', 31337))

    function getUserOpHash(op, entryPoint, chainId) {
        const userOpHash = keccak256(packUserOp(op))
        const enc = ethers.utils.defaultAbiCoder.encode(
            ['bytes32', 'address', 'uint256'],
            [userOpHash, entryPoint, chainId])
        return keccak256(enc)
    }

    function packUserOp(op) {
        return ethers.utils.defaultAbiCoder.encode(
            userOpType,
            [op.sender, op.nonce, keccak256(op.initCode), keccak256(op.callData),
                op.callGasLimit, op.verificationGasLimit, op.preVerificationGas, op.maxFeePerGas, op.maxPriorityFeePerGas,
                keccak256(op.paymasterAndData)])
    }

    wallet.signMessage(message).then((sig) => {
        userOperation.signature = sig
        console.log(userOperation)

        let values = Object.values(userOperation)
        let str = ""
        for (let i = 0; i < values.length; i++) {
            str += values[i]
            if (i < values.length - 1) {
                str += ","
            }
        }
        console.log(str)

    }).catch((err) => {
        console.log(err)
    })
});
