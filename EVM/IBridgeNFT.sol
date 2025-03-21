// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
interface IBridgeNFT {
    event BridgeMinted(address to, uint256 tokenId);

    function bridgeMint(address to, uint256 tokenId) external;
}