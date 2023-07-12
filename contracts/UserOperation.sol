// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.18.0;

import {calldataKeccak} from "../Helpers.sol";

/**
 * User Operation struct
 * @param sender the sender account of this request.
 * @param nonce unique value the sender uses to verify it is not a replay.
 * @param initCode if set, the account contract will be created by this constructor/
 * @param callData the method call to execute on this account.
 * @param callGasLimit the gas limit passed to the callData method call.
 * @param verificationGasLimit gas used for validateUserOp and validatePaymasterUserOp.
 * @param preVerificationGas gas not calculated by the handleOps method, but added to the gas paid. Covers batch overhead.
 * @param maxFeePerGas same as EIP-1559 gas parameter.
 * @param maxPriorityFeePerGas same as EIP-1559 gas parameter.
 * @param paymasterAndData if set, this field holds the paymaster address and paymaster-specific data. the paymaster will pay for the transaction instead of the sender.
 * @param signature sender-verified signature over the entire request, the EntryPoint address and the chain ID.
*/

    struct UserOperation {

        address sender;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        uint256 callGasLimit;
        uint256 verificationGasLimit;
        uint256 preVerificationGas;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        bytes paymasterAndData;
        bytes signature;
    }

// as of now, this is here temporarily.
library UserOperationLib {

    function pack(UserOperation calldata userOp) internal pure returns (bytes memory ret) {
        bytes calldata sig = userOp.signature;
        // copy directly the userOp from calldata up to (but not including) the signature.
        // this encoding depends on the ABI encoding of calldata, but is much lighter to cop)
        // than referencing each field separately.
        assembly {
            let ofs := userOp
            let len := sub(sub(sig.offset, ofs), 32)
            ret := mload(0x40)
            mstore(0x40, add(ret, add(len, 32)))
            mstore(ret, len)
            calldatacopy(add(ret, 32), ofs, len)
        }
    }

    function hash(UserOperation calldata userOp) internal pure returns (bytes32) {
        return keccak256(pack(userOp));
    }
}
