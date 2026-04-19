// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ExecutionEngine} from "./ExecutionEngine.sol";

contract IntentFactory {
    address public owner;
    address[] public engines;
    mapping(address => address[]) public enginesByCreator;

    event EngineDeployed(
        address indexed engine,
        address indexed creator,
        string domainName,
        string domainVersion,
        address treasury,
        uint16 protocolFeeBps
    );

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "owner is zero");
        owner = newOwner;
    }

    function deployEngine(
        string calldata domainName,
        string calldata domainVersion,
        address treasury,
        uint16 protocolFeeBps
    ) external returns (address engine) {
        engine = address(new ExecutionEngine(domainName, domainVersion, treasury, protocolFeeBps));
        engines.push(engine);
        enginesByCreator[msg.sender].push(engine);

        emit EngineDeployed(engine, msg.sender, domainName, domainVersion, treasury, protocolFeeBps);
    }

    function totalEngines() external view returns (uint256) {
        return engines.length;
    }
}
