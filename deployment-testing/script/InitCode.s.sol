// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";
import "../src/interfaces/UserOperation.sol";

contract InitCodeScript is Script {

    UserOperation internal userOp;

    function _prepareInitCode(address simpleAccountFactory, address owner, uint256 salt) public pure returns(bytes memory) {
        bytes memory _func = abi.encodeWithSignature("createAccount(address,uint256)", owner, salt);
        bytes memory _addr = abi.encodePacked(simpleAccountFactory);
        return bytes.concat(_addr, _func);
    }

    function _prepareCallData(address receiver, uint256 amount) public pure returns(bytes memory) {
        bytes memory callData = abi.encodeWithSignature("execute(address,uint256,bytes)", receiver, amount, '0x');
        return callData;
    }

    function _prepareGasPayload() public {
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
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address payable beneficiary = payable(vm.envAddress("BENEFICIARY"));
        address receiver = vm.envAddress("RECEIVER");
        address owner = vm.envAddress("OWNER");
        uint256 amount = vm.envUint("AMOUNT");
        uint256 salt = vm.envUint("SALT");

        vm.startBroadcast(privateKey);

        EntryPoint entryPoint = new EntryPoint();
        SimpleAccountFactory simpleAccountFactory = new SimpleAccountFactory(entryPoint);

        userOp.sender = simpleAccountFactory.getAddress(owner, salt);
        userOp.nonce = vm.getNonce(userOp.sender);

        if (isDeployed(userOp.sender)) {
            userOp.initCode = '';
        } else {
            userOp.initCode = _prepareInitCode(address(simpleAccountFactory), owner, salt);
        }
        userOp.callData = _prepareCallData(receiver, amount);

        _prepareGasPayload();

        userOp.paymasterAndData = '';

        entryPoint.depositTo{value: 10 ether}(userOp.sender);
        vm.deal(userOp.sender, 20 ether);

        bytes32 msgHash = entryPoint.getUserOpHash(userOp);
        bytes32 hash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", msgHash));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, hash);

        userOp.signature = abi.encodePacked(r, s, v);

        UserOperation[] memory userOpArray = new UserOperation[](1);
        userOpArray[0] = userOp;

        entryPoint.handleOps(userOpArray, beneficiary);

        vm.stopBroadcast();
    }
}
