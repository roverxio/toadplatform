// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.20.0;

import "./IEntryPoint.sol";
import "./UserOperation.sol";

contract EntryPoint is IEntryPoint {

    function handleOp(UserOperation memory op) public returns(uint) {
        if (!_verifyOp(op)) {
            revert("verification failed");
        }
        return _executeOp(op);
    }

    function _verifyOp(UserOperation memory op) private view returns(bool) {
        return op.sender.balance > op.amount;
    }

    function _executeOp(UserOperation memory op) private returns (uint) {

        bytes memory data = abi.encodeWithSignature("executeUserOp(address,uint256)", op.receiver, op.amount);

        (bool success, bytes memory returnData) = op.sender.call(data);
        require(success, "Wallet transfer function call failed");

        return abi.decode(returnData, (uint));
    }

    function deposit(address payable targetContract) external payable returns(uint) {
        targetContract.transfer(msg.value);
        return msg.value;
    }

    function getBalance(address targetContract) external view returns(uint) {
        return targetContract.balance;
    }
}