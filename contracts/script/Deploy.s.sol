// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {IntentFactory} from "../src/IntentFactory.sol";
import {ExecutionEngine} from "../src/ExecutionEngine.sol";

contract Deploy is Script {
    function run() external {
        address deployer = vm.envAddress("DEPLOYER_ADDRESS");
        uint16 protocolFeeBps = uint16(vm.envOr("PROTOCOL_FEE_BPS", uint256(10)));
        string memory domainName = vm.envOr("INTENT_DOMAIN_NAME", string("IntentSwap Relay"));
        string memory domainVersion = vm.envOr("INTENT_DOMAIN_VERSION", string("1"));

        vm.startBroadcast();

        IntentFactory factory = new IntentFactory();
        console.log("FACTORY_ADDRESS:", address(factory));

        address engine = factory.deployEngine(domainName, domainVersion, deployer, protocolFeeBps);
        console.log("ENGINE_ADDRESS:", engine);

        vm.stopBroadcast();
    }
}
