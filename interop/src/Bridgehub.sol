// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

struct L2TransactionRequestTwoBridgesOuter {
    uint256 chainId;
    uint256 mintValue;
    uint256 l2Value;
    uint256 l2GasLimit;
    uint256 l2GasPerPubdataByteLimit;
    address refundRecipient;
    address secondBridgeAddress;
    uint256 secondBridgeValue;
    bytes secondBridgeCalldata;
}

contract Bridgehub {
    event InteropMessage(
        uint256 indexed destinationChain,
        address indexed destinationAddress,
        address indexed sourceAddress,
        bytes payload
    );

    function requestL2TransactionTwoBridges(
        L2TransactionRequestTwoBridgesOuter memory request
    ) public payable returns (bytes32 canonicalTxHash) {
        emit InteropMessage(
            request.chainId,
            request.secondBridgeAddress,
            msg.sender,
            abi.encode(request)
        );
        canonicalTxHash = keccak256(abi.encode(request.chainId));
    }

    function requestInterop(
        uint256 chainId,
        address destinationAddress,
        uint256 gasLimit,
        bytes calldata destinationCalldata
    ) external payable returns (bytes32 canonicalTxHash) {
        L2TransactionRequestTwoBridgesOuter
            memory request = L2TransactionRequestTwoBridgesOuter({
                chainId: chainId,
                mintValue: 0,
                l2Value: 0,
                l2GasLimit: gasLimit,
                l2GasPerPubdataByteLimit: 50000,
                refundRecipient: address(0),
                secondBridgeAddress: destinationAddress,
                secondBridgeValue: 0,
                secondBridgeCalldata: destinationCalldata
            });
        canonicalTxHash = requestL2TransactionTwoBridges(request);
    }
}
