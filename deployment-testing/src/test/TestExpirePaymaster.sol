// SPDX-License-Identifier: GPL-3.0-only
pragma solidity ^0.8.12;

import "../core/BasePaymaster.sol";

/**
 * test expiry mechanism: paymasterData encodes the "validUntil" and validAfter" times
 */
contract TestExpirePaymaster is BasePaymaster {
    // solhint-disable no-empty-blocks
    constructor(IEntryPoint _entryPoint) BasePaymaster(_entryPoint) {}

    function _validatePaymasterUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost)
        internal
        view
        virtual
        override
        returns (bytes memory context, uint256 validationData)
    {
        (userOp, userOpHash, maxCost);
        bytes memory pmd = userOp.paymasterAndData[20:];
        bytes6 validAfter;
        bytes6 validUntil;
        assembly {
            validAfter := mload(add(pmd, 0x20))
            validUntil := mload(add(pmd, 0x26))
        }
        validationData = _packValidationData(false, uint48(validUntil), uint48(validAfter));
        context = "";
    }
}
