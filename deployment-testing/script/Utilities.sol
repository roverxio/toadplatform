// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";

contract Utilities is Script {
    function isContract(address _addr) public view returns (bool) {
        uint256 size;
        assembly {
            size := extcodesize(_addr)
        }
        return size > 0;
    }

    function entryPointSetUp() public view returns (address entryPointAddress) {
        entryPointAddress = vm.envAddress("ENTRYPOINT_ADDRESS");
        require(
            isContract(entryPointAddress),
            "The address specified by `ENTRYPOINT_ADDRESS` doesnot have any code deployed"
        );
    }
}
