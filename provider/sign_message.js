/*
pack the struct using abi-encoder
{
  sender: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
  nonce: '0x0000000000000000000000000000000000000000',
  initCode: '0x',
  callData: '0x',
  callGasLimit: 0,
  verificationGasLimit: 100000,
  preVerificationGas: 21000,
  maxFeePerGas: BigNumber { value: "1541441108" },
  maxPriorityFeePerGas: 1000000000,
  paymasterAndData: '0x',
  signature: '0x'
}
* */

const {ethers, keccak256} = require("ethers");

const privateKey = 'your_private_key';
const providerUrl = 'http://localhost:8545';
const provider = new ethers.JsonRpcProvider(providerUrl);
const wallet = new ethers.Wallet(privateKey, provider);

const userOpType = {
    components: [{type: 'address', name: 'sender'}, {type: 'uint256', name: 'nonce'}, {
        type: 'bytes',
        name: 'initCode'
    }, {type: 'bytes', name: 'callData'}, {type: 'uint256', name: 'callGasLimit'}, {
        type: 'uint256',
        name: 'verificationGasLimit'
    }, {type: 'uint256', name: 'preVerificationGas'}, {type: 'uint256', name: 'maxFeePerGas'}, {
        type: 'uint256',
        name: 'maxPriorityFeePerGas'
    }, {type: 'bytes', name: 'paymasterAndData'}, {type: 'bytes', name: 'signature'}], name: 'userOp', type: 'tuple'
}

let userOperation = {
    sender: '<scw_address>',
    nonce: '0x0000000000000000000000000000000000000000',
    initCode: '0x',
    callData: '0x',
    callGasLimit: 0,
    verificationGasLimit: 100000,
    preVerificationGas: 21000,
    maxFeePerGas: 1541441108,
    maxPriorityFeePerGas: 1000000000,
    paymasterAndData: '0x',
    signature: '0x'
}


let encoded = ethers.AbiCoder.defaultAbiCoder().encode([userOpType], [userOperation])
console.log(encoded)
// remove leading word (total length) and trailing word (zero-length signature)
encoded = "Ox" + encoded.slice(66, encoded.length - 64)
console.log("-------------------")
console.log("encoded -> ", encoded)
console.log("-------------------")
const userOpHash = keccak256(ethers.toUtf8Bytes(encoded))
const enc = ethers.AbiCoder.defaultAbiCoder().encode(["bytes32", "address", "uint256"],
    [userOpHash, "<entry_point_address>", 31337]) // 31337 is the chainId
console.log(enc)

wallet.signMessage(enc).then((sig) => {
    console.log("userOpHash -> ", userOpHash)
    userOperation.signature = sig
    console.log("userOperation -> ", userOperation)
}).catch((err) => {
    console.log(err)
})
