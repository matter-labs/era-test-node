// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

//interface IGasBurner {
//    function burnGas(uint256 gasToBurn) external;
//}

contract Counter {
    uint256 public number;

    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }
    function alwaysrevert() public pure {
        require(false, "aaa");
    }

    function increment() public {
        number++;
    }
}
