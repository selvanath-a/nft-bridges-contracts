// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
interface ICCIPBridge {
    struct BridgeCCIPParams {
        address srcCollection;
        address dstCollection;
        uint256[] tokenIds;
        uint64 dstChain;
        address senderAddress;
        address receiverAddress;
        address dstBridgeAddress;
        uint256 serviceFee;
        string bridgeTxId;
        address feeToken;
        bytes extraArgs;
    }

    function bridgeCCIP(BridgeCCIPParams calldata params) external payable;

    struct MessageInfoCCIP {
        address srcCollection;
        address dstCollection;
        uint256[] tokenIds;
        address sender;
        address receiver;
        string bridgeTxId;
    }
}
