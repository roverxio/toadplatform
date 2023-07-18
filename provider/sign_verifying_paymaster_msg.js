const {ethers, utils} = require("ethers");
const {arrayify, keccak256, hexConcat, defaultAbiCoder} = require("ethers/lib/utils");

const privateKey = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
const providerUrl = 'http://localhost:8545';
const provider = new ethers.providers.JsonRpcProvider(providerUrl);
const wallet = new ethers.Wallet(privateKey, provider);

const userOpType = ['address', 'uint256', 'bytes32', 'bytes32',
    'uint256', 'uint256', 'uint256', 'uint256', 'uint256',
    'bytes32']

wallet.signMessage(arrayify("<msg_hash>")).then((sig) => {
    const paymaster = hexConcat(["<verify_paymaster>", defaultAbiCoder.encode(['uint48', 'uint48'], ['0x00000000deadbeef', '0x0000000000001234']), sig])

    let userOperation = {
        //user operation payload
    }
    const message = arrayify(getUserOpHash(userOperation, '<entry_point_address>', 31337))

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
