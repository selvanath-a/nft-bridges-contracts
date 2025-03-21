//SPDX-License-Identifier:MIT
pragma solidity 0.8.24;

import {Client} from "@chainlink/contracts/src/v0.8/ccip/libraries/Client.sol";
import {IERC721Receiver} from "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IBridgeNFT} from "./IBridgeNFT.sol";
import {CCIPReceiverUpgradeable} from "./CCIPReceiverUpgradeable.sol";
import {ICCIPBridge} from "./ICCIPBridge.sol";
import {IRouterClient} from "@chainlink/contracts/src/v0.8/ccip/interfaces/IRouterClient.sol";

contract BridgeManager is
    AccessControlUpgradeable,
    CCIPReceiverUpgradeable,
    IERC721Receiver,
    UUPSUpgradeable,
    ReentrancyGuardUpgradeable
{

    mapping(uint64 => string) private s_bridgeManagers; // bridge address on chains
    mapping(uint64 => address) private s_bridgeManagersCCIP; // bridge address on chains
    mapping(address => mapping(uint256 => string))
        public s_nftCollectionMapping; // (src_nft, dst_chain) => dst_nft
    mapping(address => mapping(uint256 => address))
        public s_nftCollectionMappingCCIP; // (src_nft, dst_chain) => dst_nft
    // State variables related to NFT handling.
    mapping(address => mapping(uint256 => address)) public s_nftSenderInfo;
    mapping(address => mapping(uint256 => uint64)) public s_nftBridgedInfo;
    mapping(address => mapping(uint256 => uint256)) public s_nftNonceStore;
    uint256 private s_serviceFee; // fee for this service
    uint64 public CURRENT_CHAIN_ID;
    IRouterClient public router;

    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");
    uint64 public constant VERSION = 1;

    event MessageReceivedCCIP(
        bytes32 indexed messageId, // The unique ID of the message.
        address indexed collection,
        uint256[] tokenIds,
        uint64 srcChain,
        address sender,
        address indexed receiver,
        string bridgeTxId
    );

    event NFTLockedCCIP(
        address indexed owner,
        address indexed collection,
        uint256[] tokenIds
    );

    event NFTUnlockedCCIP(
        address indexed receiver,
        address indexed collection,
        uint256[] tokenIds
    );

    event NFTLocked(
        address indexed owner,
        address indexed collection,
        uint256[] tokenIds,
        uint64 indexed destinationChain,
        string destinationAddress,
        string bridgeTxId
    );

    event NFTUnlocked(
        uint64 indexed srcChain,
        string senderAddress,
        address indexed dstAddress,
        address indexed dstCollection,
        uint256[] tokenIds,
        string bridgeTxId
    );
    event CCIPMessageSent(
        bytes32 indexed messageId,
        address indexed collection,
        uint256[] tokenIds,
        uint64 dstChain,
        address indexed receiverAddress,
        string bridgeTxId,
        uint256 feePaid
    );

    error BridgeManager_ChainNotAllowed();
    error BridgeManager_CollectionNotAllowed();
    error BridgeManager_SenderNotBridgeManager();
    error BridgeManager_InvalidFee();
    error BridgeManager_RefundFailed();
    error BridgeManager_InvalidNFTOwner();
    error BridgeManager_InvalidNFTLength();

      function initialize(address _router,uint64 _chainId) public initializer {
        __AccessControl_init();
        __CCIPReceiver_init(_router);
        __ReentrancyGuard_init();
        router = IRouterClient(_router);
        CURRENT_CHAIN_ID = _chainId;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MANAGER_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(UPGRADER_ROLE, msg.sender);
    }


    function supportsInterface(bytes4 interfaceId)
        public
        pure
        virtual
        override(AccessControlUpgradeable, CCIPReceiverUpgradeable)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    function onERC721Received(
        address,
        address,
        uint256,
        bytes memory
    ) public pure override returns (bytes4) {
        return this.onERC721Received.selector;
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyRole(UPGRADER_ROLE)
    {}

    /*
     * Manager Functions
     */

    function setCurrentChainId(uint64 _chainId)
        external
        onlyRole(MANAGER_ROLE)
    {
        CURRENT_CHAIN_ID = _chainId;
    }

    function addCollectionCCIP(
        address srcCollection,
        uint64 dstChain,
        address dstCollectionCCIP
    ) external onlyRole(MANAGER_ROLE) {
        s_nftCollectionMappingCCIP[srcCollection][dstChain] = dstCollectionCCIP;
    }

    function addCollection(
        address srcCollection,
        uint64 dstChain,
        string calldata dstCollection
    ) external onlyRole(MANAGER_ROLE) {
        s_nftCollectionMapping[srcCollection][dstChain] = dstCollection;
    }

    function removeCollectionMapping(
        address collection,
        uint64[] calldata dstChain,
        bool isCCIP
    ) external onlyRole(MANAGER_ROLE) {
        for (uint256 i = 0; i < dstChain.length; i++) {
            if (isCCIP) {
                delete s_nftCollectionMappingCCIP[collection][dstChain[i]];
            } else {
                delete s_nftCollectionMapping[collection][dstChain[i]];
            }
        }
    }

    function setServiceFee(uint256 _serviceFee)
        external
        onlyRole(MANAGER_ROLE)
    {
        s_serviceFee = _serviceFee;
    }

    function getNftNonce(address collection, uint256 nftId)
        external
        view
        returns (uint256 nonce)
    {
        return s_nftNonceStore[collection][nftId];
    }

    /*
     * NFT Bridges Bridging
     */

    function setBridgeManager(uint64 _chain, string calldata _bridgeManager)
        external
        onlyRole(MANAGER_ROLE)
    {
        s_bridgeManagers[_chain] = _bridgeManager;
    }

    function setBridgeManagerCCIP(uint64 _chain, address _bridgeManagerCCIP)
        external
        onlyRole(MANAGER_ROLE)
    {
        s_bridgeManagersCCIP[_chain] = _bridgeManagerCCIP;
    }

    function lockNFT(
        address collection,
        uint256[] memory tokenIds,
        uint64 dstChain,
        string memory receiver,
        string memory bridgeTxId
    ) external payable nonReentrant {
        if (
            bytes(s_bridgeManagers[dstChain]).length == 0
        ) {
            revert BridgeManager_ChainNotAllowed();
        }

        if (bytes(s_nftCollectionMapping[collection][dstChain]).length == 0) {
            revert BridgeManager_CollectionNotAllowed();
        }

        if (msg.value < s_serviceFee) {
            revert BridgeManager_InvalidFee();
        }
        // require(
        //     tokenIds.length > 0 && tokenIds.length < 6,
        //     "Token ID array length must be 1-5."
        // );
        // for (uint256 i = 0; i < tokenIds.length; i++) {
        //     uint256 tokenId = tokenIds[i];

        //     if (IERC721(collection).ownerOf(tokenId) != msg.sender) {
        //         revert BridgeManager_InvalidNFTOwner();
        //     }
        // }
        _lockNFT(collection, tokenIds, dstChain);
        emit NFTLocked(
            msg.sender,
            collection,
            tokenIds,
            dstChain,
            receiver,
            bridgeTxId
        );
        uint256 restFee = msg.value - s_serviceFee;
        if (restFee > 0) {
            // Refund the rest Fee to the caller
            (bool success, ) = msg.sender.call{value: restFee}("");
            if (!success) {
                revert BridgeManager_RefundFailed();
            }
        }
    }

    function unlockNFT(
        uint64 srcChain,
        uint64 dstChain,
        string calldata senderAddress,
        address receiver,
        address collection,
        string calldata bridgeTxId,
        uint256[] memory tokenIds
    ) external onlyRole(MINTER_ROLE) {
        if (
            dstChain != CURRENT_CHAIN_ID ||
            bytes(s_bridgeManagers[srcChain]).length == 0
        ) {
            revert BridgeManager_ChainNotAllowed();
        }
        // require(
        //     !(dstChain != CURRENT_CHAIN_ID ||
        //         s_ccipEnabled[srcChain] ||
        //         bytes(s_bridgeManagers[srcChain]).length == 0),
        //     "E-5"
        // );

        if (bytes(s_nftCollectionMapping[collection][srcChain]).length == 0) {
            revert BridgeManager_CollectionNotAllowed();
        }

        // require(
        //     !(!s_nftCollectionAdded[collection] ||
        //         bytes(s_nftCollectionMapping[collection][srcChain]).length ==
        //         0),
        //     "E-4"
        // );

        _unlockNFT(collection, tokenIds, receiver);
        emit NFTUnlocked(
            srcChain,
            senderAddress,
            receiver,
            collection,
            tokenIds,
            bridgeTxId
        );
    }

    /**
     *   CCIP BRIDGING
     **/

    function bridgeCCIP(ICCIPBridge.BridgeCCIPParams memory _params)
        external
        payable
        nonReentrant
    {
        // Validate that the destination chain is allowed.
        if (s_bridgeManagersCCIP[_params.dstChain] == address(0)) {
            revert BridgeManager_ChainNotAllowed();
        }
        // require(s_bridgeManagersCCIP[_params.dstChain] != address(0), "E-3");

        // Validate that the NFT collection is allowed.
        if (
            s_nftCollectionMappingCCIP[_params.srcCollection][
                _params.dstChain
            ] == address(0)
        ) {
            revert BridgeManager_CollectionNotAllowed();
        }
        // require(
        //     !s_nftCollectionAdded[_params.srcCollection] ||
        //         s_nftCollectionMappingCCIP[_params.srcCollection][
        //             _params.dstChain
        //         ] !=
        //         address(0),
        //     "E-4"
        // );
        _params.senderAddress = msg.sender;
        // // Lock the NFT
        _lockNFT(_params.srcCollection, _params.tokenIds, _params.dstChain);
        _params.dstCollection = s_nftCollectionMappingCCIP[
            _params.srcCollection
        ][_params.dstChain];
        _params.dstBridgeAddress = s_bridgeManagersCCIP[_params.dstChain];
        emit NFTLockedCCIP(msg.sender, _params.srcCollection, _params.tokenIds);
        // ccipBridge.bridgeCCIP{value: msg.value}(_params);

        Client.EVM2AnyMessage memory evmMessage = _buildMessage(_params);

        uint256 routerFee = router.getFee(_params.dstChain, evmMessage);
        uint256 totalFee = routerFee + s_serviceFee;
        require(msg.value >= totalFee, "Insufficient fee provided");

        bytes32 messageId = router.ccipSend{value: routerFee}(
            _params.dstChain,
            evmMessage
        );

        uint256 extra = msg.value - totalFee;
        emit CCIPMessageSent(
            messageId,
            _params.srcCollection,
            _params.tokenIds,
            _params.dstChain,
            _params.receiverAddress,
            _params.bridgeTxId,
            totalFee
        );
        if (extra > 0) {
            payable(_params.senderAddress).transfer(extra);
        }
    }

    function _ccipReceive(Client.Any2EVMMessage memory any2EvmMessage)
        internal
        override
    {
        uint64 srcChain = any2EvmMessage.sourceChainSelector;
        address sender = abi.decode(any2EvmMessage.sender, (address));
        ICCIPBridge.MessageInfoCCIP memory data = abi.decode(
            any2EvmMessage.data,
            (ICCIPBridge.MessageInfoCCIP)
        );
        if (sender != s_bridgeManagersCCIP[srcChain])
            revert BridgeManager_SenderNotBridgeManager();
        // require(!(sender != s_bridgeManagersCCIP[srcChain]), "E-6");

        if (
            (s_nftCollectionMappingCCIP[data.dstCollection][srcChain] !=
                data.srcCollection)
        ) {
            revert BridgeManager_CollectionNotAllowed();
        }
        // require(
        //     !(!s_nftCollectionAdded[data.dstCollection] ||
        //         (s_nftCollectionMappingCCIP[data.dstCollection][srcChain] !=
        //             data.srcCollection)),
        //     "E-7"
        // );

        emit NFTUnlockedCCIP(data.receiver, data.dstCollection, data.tokenIds);
        emit MessageReceivedCCIP(
            any2EvmMessage.messageId,
            data.dstCollection,
            data.tokenIds,
            srcChain,
            data.sender,
            data.receiver,
            data.bridgeTxId
        );
        _unlockNFT(data.dstCollection, data.tokenIds, data.receiver);
    }

    function _buildMessage(ICCIPBridge.BridgeCCIPParams memory params)
        internal
        pure
        returns (Client.EVM2AnyMessage memory)
    {
        // This is a simplified version of message building.
        // You would normally encode all relevant data into a struct.
        ICCIPBridge.MessageInfoCCIP memory data = ICCIPBridge.MessageInfoCCIP({
            srcCollection: params.srcCollection,
            dstCollection: params.dstCollection,
            tokenIds: params.tokenIds,
            sender: params.senderAddress,
            receiver: params.receiverAddress,
            bridgeTxId: params.bridgeTxId
        });
        Client.EVMTokenAmount[]
            memory tokenAmounts = new Client.EVMTokenAmount[](0);
        return
            Client.EVM2AnyMessage({
                receiver: abi.encode(params.dstBridgeAddress),
                data: abi.encode(data),
                tokenAmounts: tokenAmounts,
                feeToken: params.feeToken,
                extraArgs: params.extraArgs
            });
    }

    /// @dev Locks an NFT by transferring it from the caller to this contract.
    function _lockNFT(
        address collection,
        uint256[] memory tokenIds,
        uint64 dstChain
    ) internal virtual {
        // require(tokenIds.length >= 1 && tokenIds.length <= 5, "E-1");
        if (tokenIds.length < 1 || tokenIds.length > 5)
            revert BridgeManager_InvalidNFTLength();
        for (uint8 i = 0; i < tokenIds.length; i++) {
            uint256 tokenId = tokenIds[i];
            // require(IERC721(collection).ownerOf(tokenId) == msg.sender, "E-2");
            if (IERC721(collection).ownerOf(tokenId) != msg.sender) {
                revert BridgeManager_InvalidNFTOwner();
            }
        }
        for (uint8 i = 0; i < tokenIds.length; i++) {
            uint256 tokenId = tokenIds[i];
            //         if (
            //     IERC721(collection).ownerOf(tokenId) != msg.sender
            // ) {
            //     revert BridgeManager_InvalidNFTOwner();
            // }
            s_nftSenderInfo[collection][tokenId] = msg.sender;
            // Transfer the NFT into custody of this contract.
            IERC721(collection).safeTransferFrom(
                msg.sender,
                address(this),
                tokenId
            );
            unchecked {
                s_nftNonceStore[collection][tokenId]++;
            }
            s_nftBridgedInfo[collection][tokenId] = dstChain;
        }
    }

    /// @dev Unlocks an NFT by transferring it from this contract to the receiver.
    function _unlockNFT(
        address collection,
        uint256[] memory tokenIds,
        address receiver
    ) internal virtual {
        // if (tokenIds.length < 1 || tokenIds.length > 5)
        //     revert BridgeManager_InvalidNFTLength();
        require(!(tokenIds.length < 1 || tokenIds.length > 5), "E-1");

        for (uint256 i = 0; i < tokenIds.length; i++) {
            uint256 tokenId = tokenIds[i];
            // If the NFT is held by this contract, transfer it back.
            if (
                s_nftBridgedInfo[collection][tokenId] != 0 &&
                IERC721(collection).ownerOf(tokenId) == address(this)
            ) {
                IERC721(collection).safeTransferFrom(
                    address(this),
                    receiver,
                    tokenId
                );
                s_nftSenderInfo[collection][tokenId] = address(0);
            } else {
                // If not, assume the NFT is minted on demand.
                IBridgeNFT nft = IBridgeNFT(collection);
                nft.bridgeMint(receiver, tokenId);
            }
            unchecked {
                s_nftNonceStore[collection][tokenId]++;
            }
        }
    }

    /*
     * Admin Functions
     */
    function withdrawBalance(address receiver)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        nonReentrant
    {
        uint256 balance = address(this).balance;
        if (balance > 0) {
            (bool success, ) = receiver.call{value: balance}("");
            if (!success) revert BridgeManager_RefundFailed();

            // require(success, "Withdraw failed");
        }
    }

    function withdrawNFT(address collection, uint256 tokenId)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        nonReentrant
    {
        address receiver = s_nftSenderInfo[collection][tokenId];

        if (
            receiver == address(0) ||
            IERC721(collection).ownerOf(tokenId) != address(this)
        ) {
            revert BridgeManager_InvalidNFTOwner();
        }
        IERC721(collection).safeTransferFrom(address(this), receiver, tokenId);
    }

    //Getters

    function getBridgeManager(uint64 chain)
        external
        view
        returns (string memory)
    {
        return s_bridgeManagers[chain];
    }

    function getBridgeManagerCCIP(uint64 chain)
        external
        view
        returns (address)
    {
        return s_bridgeManagersCCIP[chain];
    }

    function getServiceFee() external view returns (uint256) {
        return s_serviceFee;
    }
}
