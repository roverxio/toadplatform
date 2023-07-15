// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/SimpleAccount.sol";
import "../src/EntryPoint.sol";
import "../src/SimpleAccountFactory.sol";

contract SimpleAccountTest is Test {
    EntryPoint private entryPoint;
    SimpleAccountFactory private factory;
    SimpleAccount private wallet;
    address payable private walletAddress;
    address private owner = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    uint256 private ownerPrivateKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
    address private epAddress;
    uint256 private chainId = vm.envOr('FOUNDRY_CHAIN_ID', uint256(31337));

    function setUp() public {
        entryPoint = new EntryPoint();
        factory = new SimpleAccountFactory(entryPoint);
        wallet = factory.createAccount(owner, 1);

        walletAddress = payable(wallet);
        epAddress = payable(entryPoint);
    }

    // Owner should be able to call transfer
    function testTransferByOwner(address receiver) public {
        // add balance to scw
        vm.deal(walletAddress, 3 ether);
        // set msg.sender to owner address
        vm.prank(owner);
        wallet.execute(receiver, 1 ether, '0x');
        assertEq(walletAddress.balance, 2 ether);
    }

    // Other account should not be able to call transfer
    function testTransferByNonOwner(address receiver) public {
        // add balance to scw
        vm.deal(walletAddress, 3 ether);
        vm.expectRevert(bytes('account: not Owner or EntryPoint'));
        wallet.execute(receiver, 1 ether, '0x');
    }

    // #validateUserOp
    // Should pay
    function testPayment() public {
        vm.deal(walletAddress, 0.2 ether);

        UserOperation memory userOp = getUserOp(epAddress, chainId, 0);
        uint256 expectedPay = 1000000000 * (userOp.callGasLimit + userOp.verificationGasLimit);
        bytes32 userOpHash = getUserOpHash(userOp, epAddress, chainId);
        uint256 preBalance =  walletAddress.balance;

        // set msg.sender to entry point address
        vm.prank(epAddress);
        wallet.validateUserOp{gas: 1000000000}(userOp, userOpHash, expectedPay);

        uint256 postBalance = address(wallet).balance;
        assertEq(preBalance - postBalance, expectedPay);
    }

    function testWrongSignature() public {
        bytes32 zeroHash = 0x0000000000000000000000000000000000000000000000000000000000000000;
        UserOperation memory op = getUserOp(epAddress, chainId, 1);

        // set msg.sender to entry point address
        vm.prank(epAddress);
        uint256 deadline = wallet.validateUserOp(op, zeroHash, 0);

        assertEq(deadline, 1);
    }

    function getUserOp(address entryPointAddress, uint256 id, uint256 nonce) internal view returns (UserOperation memory) {
        UserOperation memory op;
        op.sender = walletAddress;
        op.nonce = nonce;
        op.callData = '0x';
        op.initCode = '0x';
        op.callGasLimit = 200000;
        op.verificationGasLimit = 100000;
        op.preVerificationGas = 21000;
        op.maxFeePerGas = 3000000000;
        op.maxPriorityFeePerGas = 1000000000;
        op.paymasterAndData = '0x';
        op.signature = '0x';

        UserOperation memory userOp = signUserOp(op, entryPointAddress, id);
        return userOp;
    }

    function packUserOp(UserOperation memory op, bool signature) internal pure returns (bytes memory) {
        if (signature) {
            return abi.encode(
                op.sender, op.nonce, keccak256(op.initCode), keccak256(op.callData), op.callGasLimit,
                op.verificationGasLimit, op.preVerificationGas, op.maxFeePerGas, op.maxPriorityFeePerGas,
                keccak256(op.paymasterAndData));
        } else {
            return abi.encode(
                op.sender, op.nonce, op.initCode, op.callData, op.callGasLimit, op.verificationGasLimit,
                op.preVerificationGas, op.maxFeePerGas, op.maxPriorityFeePerGas, op.paymasterAndData, op.signature);
        }
    }

    function getUserOpHash(UserOperation memory op, address ep, uint256 id) internal pure returns (bytes32) {
        bytes32 userOpHash = keccak256(packUserOp(op, true));
        bytes memory encoded = abi.encode(userOpHash, ep, id);
        return bytes32(keccak256(encoded));
    }

    function signUserOp(UserOperation memory op, address ep, uint256 id) internal view returns (UserOperation memory) {
        bytes32 message = getUserOpHash(op, ep, id);
        bytes32 digest = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, digest);
        op.signature = abi.encodePacked(r, s, v);
        return op;
    }
}
