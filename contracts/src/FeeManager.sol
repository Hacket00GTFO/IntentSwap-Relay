// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract FeeManager {
    uint256 internal constant BPS_DENOMINATOR = 10_000;

    address public owner;
    address public treasury;
    uint16 public protocolFeeBps;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event TreasuryUpdated(address indexed previousTreasury, address indexed newTreasury);
    event ProtocolFeeBpsUpdated(uint16 previousBps, uint16 newBps);

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner");
        _;
    }

    constructor(address treasury_, uint16 protocolFeeBps_) {
        require(treasury_ != address(0), "treasury is zero");
        require(protocolFeeBps_ <= BPS_DENOMINATOR, "invalid protocol fee");

        owner = msg.sender;
        treasury = treasury_;
        protocolFeeBps = protocolFeeBps_;

        emit OwnershipTransferred(address(0), owner);
        emit TreasuryUpdated(address(0), treasury_);
        emit ProtocolFeeBpsUpdated(0, protocolFeeBps_);
    }

    function setTreasury(address newTreasury) external onlyOwner {
        require(newTreasury != address(0), "treasury is zero");
        address previousTreasury = treasury;
        treasury = newTreasury;
        emit TreasuryUpdated(previousTreasury, newTreasury);
    }

    function setProtocolFeeBps(uint16 newProtocolFeeBps) external onlyOwner {
        require(newProtocolFeeBps <= BPS_DENOMINATOR, "invalid protocol fee");
        uint16 previous = protocolFeeBps;
        protocolFeeBps = newProtocolFeeBps;
        emit ProtocolFeeBpsUpdated(previous, newProtocolFeeBps);
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "owner is zero");
        address previousOwner = owner;
        owner = newOwner;
        emit OwnershipTransferred(previousOwner, newOwner);
    }

    function previewFees(
        uint256 amountIn,
        uint16 maxRelayerFeeBps,
        uint16 requestedRelayerFeeBps
    ) public view returns (uint256 protocolFee, uint256 relayerFee) {
        require(maxRelayerFeeBps <= BPS_DENOMINATOR, "max relayer fee too high");
        require(requestedRelayerFeeBps <= maxRelayerFeeBps, "relayer fee exceeds cap");
        require(requestedRelayerFeeBps >= protocolFeeBps, "relayer fee below protocol fee");

        protocolFee = (amountIn * protocolFeeBps) / BPS_DENOMINATOR;
        relayerFee = (amountIn * (requestedRelayerFeeBps - protocolFeeBps)) / BPS_DENOMINATOR;
    }
}
