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
      [op.sender, op.nonce, keccak256(op.init_code), keccak256(op.calldata),
        op.call_gas_limit, op.verification_gas_limit, op.pre_verification_gas, op.max_fee_per_gas, op.max_priority_fee_per_gas,
        keccak256(op.paymaster_and_data)])
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
