// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IAny2EVMMessageReceiver} from "@chainlink/contracts/src/v0.8/ccip/interfaces/IAny2EVMMessageReceiver.sol";
import {Client} from "@chainlink/contracts/src/v0.8/ccip/libraries/Client.sol";
import {IERC165} from "@chainlink/contracts/src/v0.8/vendor/openzeppelin-solidity/v4.8.3/contracts/utils/introspection/IERC165.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/// @title CCIPReceiverUpgradeable - Base contract for CCIP applications that can receive messages.
abstract contract CCIPReceiverUpgradeable is
    Initializable,
    IAny2EVMMessageReceiver,
    IERC165
{
    address internal ccipRouter;

    function __CCIPReceiver_init(address router) internal initializer {
        require(router != address(0), "InvalidRouter");
        ccipRouter = router;
    }

    /// @notice IERC165 supports an interfaceId
    /// @param interfaceId The interfaceId to check
    /// @return true if the interfaceId is supported
    /// @dev Should indicate whether the contract implements IAny2EVMMessageReceiver
    function supportsInterface(
        bytes4 interfaceId
    ) public pure virtual override returns (bool) {
        return
            interfaceId == type(IAny2EVMMessageReceiver).interfaceId ||
            interfaceId == type(IERC165).interfaceId;
    }

    /// @inheritdoc IAny2EVMMessageReceiver
    function ccipReceive(
        Client.Any2EVMMessage calldata message
    ) external virtual override onlyRouter {
        _ccipReceive(message);
    }

    /// @notice Override this function in your implementation.
    /// @param message Any2EVMMessage
    function _ccipReceive(
        Client.Any2EVMMessage memory message
    ) internal virtual;

    /// @notice Return the current router
    /// @return CCIP router address
    function getRouter() public view returns (address) {
        return ccipRouter;
    }

    error InvalidRouter(address router);

    /// @dev only calls from the set router are accepted.
    modifier onlyRouter() {
        require(msg.sender == ccipRouter, "InvalidRouter");
        _;
    }
}
