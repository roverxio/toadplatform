// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18.0;

import "./UserOperation.sol";

interface IAccount {

    /**
     * Validate user's signature
     * the entryPoint will make the call to the recipient only if this validation call returns successfully.
     * signature failure should be reported by returning SIG_VALIDATION_FAILED (1).
     *
     * @dev Must validate caller is the entryPoint.
     *      Must validate the signature
     * @param userOp the operation that is about to be executed.
     * @param userOpHash hash of the user's request data, minus the signature. can be used as the basis for signature.
     * @param missingAccountFunds missing funds on the account's deposit in the entrypoint.
     *      This is the minimum amount to transfer to the sender(entryPoint) to be able to make the call.
     *      The excess is left as a deposit in the entrypoint, for future calls.
     */
    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
    external returns (uint256 validationData);
}
