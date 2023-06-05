// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.20.0;

import "./UserOperation.sol";

contract Wallet {

    event Paid(bytes indexed data);

    function executeUserOp(address receiver, uint amount) external returns(uint) {

        uint pg = gasleft();
        (bool success, bytes memory data) = payable(receiver).call{value: amount}("");

        if (!success) {
            revert("pay failed");
        }
        emit Paid(data);

        return pg - gasleft();
    }

    receive() external payable {
    }
}