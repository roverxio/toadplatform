// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/interfaces/UserOperation.sol";

contract InitCodeScript is Script {

    uint256 private privateKey = vm.envUint("PRIVATE_KEY");
    address payable private beneficiary = payable(vm.envAddress("BENEFICIARY"));
    address private receiver = vm.envAddress("RECEIVER");
    address private owner = vm.envAddress("OWNER");
    uint256 private amount = vm.envUint("AMOUNT");
    uint256 private salt = vm.envUint("SALT");

    UserOperation internal userOp;

    function prepareInitCode(address simpleAccountFactory) public view returns(bytes memory) {
        bytes memory _func = abi.encodeWithSignature("createAccount(address,uint256)", owner, salt);
        bytes memory _addr = abi.encodePacked(simpleAccountFactory);
        return bytes.concat(_addr, _func);
    }


    function prepareCallData() public view returns(bytes memory) {
        bytes memory callData = abi.encodeWithSignature("execute(address,uint256,bytes)", receiver, amount, '0x');
        return callData;
    }

    function prepareGasPayload() public {
        userOp.callGasLimit = 10000000;
        userOp.maxFeePerGas = 5;
        userOp.maxPriorityFeePerGas = 1000000000;
        userOp.preVerificationGas = 1000000000000000;
        userOp.verificationGasLimit = 10000000;
    }

    function isDeployed(address addr) public view returns(bool) {
        uint size;
        assembly {
            size := extcodesize(addr)
        }
        return size > 0;
    }

    function run() external {
        vm.startBroadcast(privateKey);

        EntryPoint entryPoint = new EntryPoint();
        SimpleAccountFactory simpleAccountFactory = new SimpleAccountFactory(entryPoint);

        userOp.sender = simpleAccountFactory.getAddress(owner, salt);
        userOp.nonce = vm.getNonce(userOp.sender);

        if (isDeployed(userOp.sender)) {
            userOp.initCode = '';
        } else {
            userOp.initCode = prepareInitCode(address(simpleAccountFactory));
        }
        userOp.callData = prepareCallData();

        prepareGasPayload();

        userOp.paymasterAndData = '';

        entryPoint.depositTo{value: 10 ether}(userOp.sender);
//        vm.deal(userOp.sender, 20 ether);
        (bool sent, ) = userOp.sender.call{value: 20 ether}("");
        require(sent, "Failed to fund SCW");

        bytes32 msgHash = entryPoint.getUserOpHash(userOp);
        bytes32 hash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", msgHash));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, hash);

        userOp.signature = abi.encodePacked(r, s, v);

        console.logAddress(userOp.sender);

        UserOperation[] memory userOpArray = new UserOperation[](1);
        userOpArray[0] = userOp;

        entryPoint.handleOps(userOpArray, beneficiary);

        vm.stopBroadcast();
    }
}
