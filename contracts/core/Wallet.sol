// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.18.0;

import "./interface/UserOperation.sol";
import "./interface/IAccount.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract Wallet2 is IAccount {

    using ECDSA for bytes32;

    // This is an Entry Point function. It's here temporarily for testing "validateUserOp" purposes.
    using UserOperationLib for UserOperation;

    address private _entryPoint;
    address private _owner;
    uint private constant SIG_VALIDATION_FAILED = 1;

    constructor(address entryPoint, address owner) {
        _entryPoint = entryPoint;
        _owner = owner;
    }

    modifier onlyOwner() {
        require(msg.sender == _owner, "only owner");
        _;
    }

    // This is an Entry Point function. It's here temporarily for testing "validateUserOp" purposes.
    function getUserOpHash(UserOperation calldata userOp) public view returns (bytes32) {
        return keccak256(abi.encode(userOp.hash(), address(this), block.chainid));
    }


    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds) external override returns (uint256 validationData) {
        _requireFromEntryPoint();
        validationData = _validateSignature(userOp, userOpHash);
        _payPrefund(missingAccountFunds);
    }

    function _requireFromEntryPoint() internal view {
        require(msg.sender == address(_entryPoint), "account: not from EntryPoint");
    }

    function _validateSignature(UserOperation calldata userOp, bytes32 userOpHash) internal view returns (uint256 validationData) {
        if (userOp.signature.length == 0) {
            revert("account: no signature");
        }
        if (userOp.signature.length != 65) {
            revert("account: invalid signature length");
        }
        if (uint8(userOp.signature[64]) != 27 && uint8(userOp.signature[64]) != 28) {
            revert("account: invalid signature v");
        }
        bytes32 hash = userOpHash.toEthSignedMessageHash();
        address signer = hash.recover(userOp.signature);
        if (signer != _owner) {
            return SIG_VALIDATION_FAILED;
        }
        return 0;
    }

    function _payPrefund(uint256 missingAccountFunds) internal {
        if (missingAccountFunds > 0) {
            payable(_entryPoint).transfer(missingAccountFunds);
        }
    }

    function execute(address destination, uint value, bytes calldata data) external {
        _requireFromEntryPoint();
        (bool success, bytes memory result) = destination.call{value: value}(data);
        if (!success) {
            revert(string(result));
        }
    }

    receive() external payable {
    }
}
