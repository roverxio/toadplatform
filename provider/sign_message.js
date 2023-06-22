const {ethers} = require("ethers");
const {arrayify, keccak256} = require("ethers/lib/utils");


const privateKey = '<your private key>';
const providerUrl = 'http://localhost:8545';
const provider = new ethers.providers.JsonRpcProvider(providerUrl);
const wallet = new ethers.Wallet(privateKey, provider);

const userOpType = ['address', 'uint256', 'bytes32', 'bytes32',
    'uint256', 'uint256', 'uint256', 'uint256', 'uint256',
    'bytes32']

let userOperation = {
    sender: '<SCW address>',
    nonce: '0',
    initCode: '0x',
    callData: '<generated calldata>',
    callGasLimit: 10000000,
    verificationGasLimit: 10000000,
    preVerificationGas: 1000000000000000,
    maxFeePerGas: 5,
    maxPriorityFeePerGas: 1000000000,
    paymasterAndData: '0x',
    signature: '0x'
}
const message = arrayify(getUserOpHash(userOperation, '<entry point>', 31337))

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

