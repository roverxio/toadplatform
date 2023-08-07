// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TestHelper.sol";
import "../src/EntryPoint.sol";
import "../src/VerifyingPaymaster.sol";

//Utils
import {Utilities} from "./Utilities.sol";

struct ReturnInfo {
    uint256 preOpGas;
    uint256 prefund;
    bool sigFailed;
    uint48 validAfter;
    uint48 validUntil;
    bytes paymasterContext;
}

struct StakeInfo {
    uint256 stake;
    uint256 unstakeDelaySec;
}

contract VerifyingPaymasterTest is TestHelper {
    UserOperation[] internal ops;
    Utilities internal utils;

    Account internal offchainSigner;
    VerifyingPaymaster internal paymaster;

    uint48 internal constant MOCK_VALID_UNTIL = 0x00000000deadbeef;
    uint48 internal constant MOCK_VALID_AFTER = 0x0000000000001234;
    bytes internal constant MOCK_SIG = "0x1234";

    function setUp() public {
        utils = new Utilities();
        // timeout feature is not implemented
        entryPoint = utils.deployEntryPoint(123456);
        entryPointAddress = address(entryPoint);

        offchainSigner = createAddress("offchainSigner");
        accountOwner = createAddress("accountOwner");

        paymaster = new VerifyingPaymaster(entryPoint, offchainSigner.addr);
        paymaster.addStake{value: 2 ether}(1);
        entryPoint.depositTo{value: 1 ether}(address(paymaster));

        (account, simpleAccountFactory) =
            utils.createAccountWithEntryPoint(accountOwner.addr, entryPoint, simpleAccountFactory);
        accountAddress = address(account);
    }

    //#parsePaymasterAndData
    //should parse data properly
    function test_ParseDataProperly() public {
        bytes memory paymasterAndData =
            abi.encodePacked(address(paymaster), abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER), MOCK_SIG);
        console.logBytes(paymasterAndData);
        (uint48 validUntil, uint48 validAfter, bytes memory signature) =
            paymaster.parsePaymasterAndData(paymasterAndData);
        assertEq(validUntil, MOCK_VALID_UNTIL);
        assertEq(validAfter, MOCK_VALID_AFTER);
        assertEq(signature, MOCK_SIG);
    }

    //#validatePaymasterUserOp
    //should reject on no signature
    function test_RejectOnNoSignature() public {
        UserOperation memory userOp = _defaultOp;
        userOp.sender = accountAddress;
        userOp.paymasterAndData =
            abi.encodePacked(address(paymaster), abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER), "0x1234");
        userOp = signUserOp(userOp, entryPointAddress, chainId);

        vm.expectRevert(
            abi.encodeWithSignature(
                "FailedOp(uint256,string)",
                0,
                "AA33 reverted: VerifyingPaymaster: invalid signature length in paymasterAndData"
            )
        );
        entryPoint.simulateValidation(userOp);
    }

    //should reject on invalid signature
    function test_RejectOnInvalidSignature() public {
        UserOperation memory userOp = _defaultOp;
        userOp.sender = accountAddress;
        userOp.paymasterAndData = abi.encodePacked(
            address(paymaster),
            abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER),
            hex"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        userOp = signUserOp(userOp, entryPointAddress, chainId);

        vm.expectRevert(
            abi.encodeWithSignature("FailedOp(uint256,string)", 0, "AA33 reverted: ECDSA: invalid signature length")
        );
        entryPoint.simulateValidation(userOp);
    }

    //with wrong signature
    function _withWrongSignatureSetup()
        public
        returns (UserOperation memory wrongSigUserOp, address payable beneficiary)
    {
        beneficiary = payable(makeAddr("beneficiary"));
        bytes memory sig = signMessage("0xdead", offchainSigner.key);

        wrongSigUserOp = _defaultOp;
        wrongSigUserOp.sender = accountAddress;
        wrongSigUserOp.paymasterAndData =
            abi.encodePacked(address(paymaster), abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER), sig);
        wrongSigUserOp = signUserOp(wrongSigUserOp, entryPointAddress, chainId);
    }

    //should return signature error (no revert) on wrong signer signature
    function test_ShouldReturnSignatureError() public {
        (UserOperation memory wrongSigUserOp,) = _withWrongSignatureSetup();

        try entryPoint.simulateValidation(wrongSigUserOp) {}
        catch (bytes memory revertReason) {
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfo,,,) = abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));
            assertEq(returnInfo.sigFailed, true);
        }
    }

    //handleOp revert on signature failure in handleOps
    function test_HandleOpsRevertOnSignatureFailure() public {
        (UserOperation memory wrongSigUserOp, address payable beneficiary) = _withWrongSignatureSetup();
        ops.push(wrongSigUserOp);

        vm.expectRevert(abi.encodeWithSignature("FailedOp(uint256,string)", 0, "AA34 signature error"));
        entryPoint.handleOps(ops, beneficiary);
    }

    //succeed with valid signature
    function test_SucceedWithValidSignature() public {
        UserOperation memory userOp1 = _defaultOp;
        userOp1.sender = accountAddress;
        userOp1.paymasterAndData = abi.encodePacked(
            address(paymaster),
            abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER),
            hex"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        userOp1 = signUserOp(userOp1, entryPointAddress, chainId);
        bytes32 hash = paymaster.getHash(userOp1, MOCK_VALID_UNTIL, MOCK_VALID_AFTER);
        bytes memory sig = signMessage(hash, offchainSigner.key);

        UserOperation memory userOp = userOp1;
        userOp.paymasterAndData =
            abi.encodePacked(address(paymaster), abi.encode(MOCK_VALID_UNTIL, MOCK_VALID_AFTER), sig);
        userOp = signUserOp(userOp, entryPointAddress, chainId);

        try entryPoint.simulateValidation(userOp) {}
        catch (bytes memory revertReason) {
            bytes memory data = utils.getDataFromEncoding(revertReason);
            (ReturnInfo memory returnInfo,,,) = abi.decode(data, (ReturnInfo, StakeInfo, StakeInfo, StakeInfo));
            assertEq(returnInfo.sigFailed, false);
            assertEq(returnInfo.validAfter, MOCK_VALID_AFTER);
            assertEq(returnInfo.validUntil, MOCK_VALID_UNTIL);
        }
    }
}
