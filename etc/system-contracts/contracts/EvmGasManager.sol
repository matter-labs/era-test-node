// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./EvmConstants.sol";
import {DEPLOYER_SYSTEM_CONTRACT} from "./Constants.sol";

// import "hardhat/console.sol";

// blake2f at address 0x9 is currently the last precompile
uint160 constant PRECOMPILES_END = 0x0a;

contract EvmGasManager {
    modifier onlySystemEvm() {
        // TODO: uncomment
        require(DEPLOYER_SYSTEM_CONTRACT.isEVM(msg.sender), "only system evm");
        _;
    }

    /*
        returns true if the account was already warm
    */
    function warmAccount(address account) external payable onlySystemEvm returns (bool wasWarm) {
        return false;
    }

    function warmSlot(uint256 slot) external payable onlySystemEvm returns (bool wasWarm) {
        return false;
    }

    function pushGasLeft(uint256 gasLeft) external payable onlySystemEvm {}

    function reportGasLeft(uint256 gasLeft) external payable onlySystemEvm {}

    function getGasLeft() external view returns (uint256 gasLeft) {
        return gasleft();
    }

    function popGasLeft() external payable onlySystemEvm returns (uint256 gasLeft) {
        return gasleft();
    }

    function returnDataCopy(bytes memory data) external pure returns (bytes memory) {
        return data;
    }
}
