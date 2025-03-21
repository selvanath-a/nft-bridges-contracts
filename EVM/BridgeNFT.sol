//SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {IBridgeNFT} from "./IBridgeNFT.sol";


contract BridgeNFT is ERC721Enumerable, Ownable, IBridgeNFT {
    using Strings for uint256;

    string public baseURI;

    address public bridgeReceiver;

    address public admin;

    constructor(
        string memory name,
        string memory symbol,
        string memory baseUri,
        address _bridgeReceiver
    ) ERC721(name, symbol) Ownable(msg.sender) {
        baseURI = baseUri;
        admin = msg.sender;
        bridgeReceiver = _bridgeReceiver;
    }

    function setBridgeReceiver(
        address _bridgeReceiver
    ) external onlyBridgeAdmin {
        bridgeReceiver = _bridgeReceiver;
    }

    function setAdmin(address _admin) external onlyOwner {
        admin = _admin;
    }

    function _baseURI() internal view override returns (string memory) {
        return baseURI;
    }

    function bridgeMint(address to, uint256 tokenId) public {
        require(
            msg.sender == bridgeReceiver,
            "BridgeNFT: Only Bridge can mint"
        );
        _safeMint(to, tokenId);
        emit BridgeMinted(to, tokenId);
    }

    // Override tokenURI function to append .json
    function tokenURI(
        uint256 tokenId
    ) public view virtual override returns (string memory) {
        require(
            ownerOf(tokenId) != address(0),
            "ERC721Metadata: URI query for nonexistent token"
        );

        // Concatenate the base URI, tokenId, and ".json"
        string memory jsonFile = string(
            abi.encodePacked(baseURI, tokenId.toString(), ".json")
        );
        return jsonFile;
    }

    /// Check if owner or admin.
    modifier onlyBridgeAdmin() {
        require(msg.sender == admin || msg.sender == owner(), "Not allowed.");
        _;
    }
}
