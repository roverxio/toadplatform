// File: handlers/sign_message.js

const {arrayify, keccak256} = require("ethers/lib/utils");
const {ethers} = require("ethers");

const userOpType = ['address', 'uint256', 'bytes32', 'bytes32',
  'uint256', 'uint256', 'uint256', 'uint256', 'uint256',
  'bytes32']

function getUserOpHash(userOperation, entrypoint_address, chain_id) {
  const userOpHash = keccak256(packUserOp(userOperation))
  const enc = ethers.utils.defaultAbiCoder.encode(
      ['bytes32', 'address', 'uint256'],
      [userOpHash, entrypoint_address, chain_id])
  return keccak256(enc)
}

function packUserOp(op) {
  return ethers.utils.defaultAbiCoder.encode(
      userOpType,
      [op.sender, op.nonce, keccak256(op.initCode), keccak256(op.calldata),
        op.callGasLimit, op.verificationGasLimit, op.preVerificationGas, op.maxFeePerGas, op.maxPriorityFeePerGas,
        keccak256(op.paymasterAndData)])
}

const signMessage = async (req, res, wallet) => {
  const userOperation = req.body.user_operation;
  const entrypoint_address = req.body.entrypoint_address;
  const chain_id = req.body.chain_id;
  let sig = null;

  try {
    const message = arrayify(getUserOpHash(userOperation, entrypoint_address, chain_id));
    sig = await wallet.signMessage(message);
  } catch (err) {
    console.log(err);
  }

  res.json({sign: sig}).status(200);
}

module.exports = signMessage;
