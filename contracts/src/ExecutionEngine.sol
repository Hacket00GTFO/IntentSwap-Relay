// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IntentTypes} from "./IntentTypes.sol";
import {IntentManager} from "./IntentManager.sol";
import {FeeManager} from "./FeeManager.sol";

contract ExecutionEngine is IntentManager, FeeManager {
    struct ExecutionReceipt {
        uint256 amountOut;
        uint256 protocolFee;
        uint256 relayerFee;
        address relayer;
        uint256 executedAt;
        bytes32 routeHash;
    }

    mapping(bytes32 => ExecutionReceipt) public executions;

    event IntentExecuted(
        bytes32 indexed intentHash,
        address indexed maker,
        address indexed relayer,
        uint256 amountOut,
        uint256 protocolFee,
        uint256 relayerFee,
        bytes32 routeHash
    );

    constructor(
        string memory name_,
        string memory version_,
        address treasury_,
        uint16 protocolFeeBps_
    ) IntentManager(name_, version_) FeeManager(treasury_, protocolFeeBps_) {}

    function executeIntent(
        IntentTypes.Intent calldata intent,
        bytes calldata signature,
        uint256 amountOut,
        uint16 requestedRelayerFeeBps,
        bytes32 routeHash
    ) external returns (bytes32 intentHash) {
        intentHash = validateIntent(intent, signature);

        if (intent.allowedRelayer != address(0)) {
            require(msg.sender == intent.allowedRelayer, "relayer not allowed");
        }

        require(amountOut >= intent.minAmountOut, "insufficient output");

        (uint256 protocolFee, uint256 relayerFee) = previewFees(
            intent.amountIn,
            intent.maxRelayerFeeBps,
            requestedRelayerFeeBps
        );

        _consumeIntent(intentHash, intent.maker);

        executions[intentHash] = ExecutionReceipt({
            amountOut: amountOut,
            protocolFee: protocolFee,
            relayerFee: relayerFee,
            relayer: msg.sender,
            executedAt: block.timestamp,
            routeHash: routeHash
        });

        emit IntentExecuted(
            intentHash,
            intent.maker,
            msg.sender,
            amountOut,
            protocolFee,
            relayerFee,
            routeHash
        );
    }
}
