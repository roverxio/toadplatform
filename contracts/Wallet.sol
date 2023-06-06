// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Wallet {

    address owner;
    event Received(uint256 indexed value);

    constructor (address _owner) payable {
        owner = _owner;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Restricted Access");
        _;
    }

    function getBalance() public view returns (uint256) {
        return address(this).balance;
    }

    function getOwner() public view returns (address) {
        return owner;
    }

    function withdraw(uint256 amount) public onlyOwner {
        payable(msg.sender).transfer(amount);
    }

    function transfer(address to, uint256 amount) public onlyOwner {
        payable(to).transfer(amount);
    }

    function deposit() external payable {}

    receive() external payable {}
}

contract Factory {

    event ContractCreated(address indexed);

    function deploy(uint256 _salt) public payable returns (address) {
        address addr = address(new Wallet{salt: bytes32(_salt)}(msg.sender));
        emit ContractCreated(addr);
        return addr;
    }

    function getBytecode() public view returns (bytes memory) {
        bytes memory bytecode = type(Wallet).creationCode;
        return abi.encodePacked(bytecode, abi.encode(msg.sender));
    }

    function getAddress(uint256 _salt) public view returns (address) {
        bytes32 hash = keccak256(abi.encodePacked(
                bytes1(0xff),
                address(this),
                _salt,
                keccak256(getBytecode())
            )
        );
        return address(uint160(uint256(hash)));
    }
}

contract EntryPoint {

    function executeOp(address _sender, bytes memory callData) external {
        (bool _success, ) = _sender.call(callData);
        require(_success);
    }
}