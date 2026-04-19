// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library IntentTypes {
    struct Intent {
        address maker;
        address tokenIn;
        address tokenOut;
        uint256 amountIn;
        uint256 minAmountOut;
        address receiver;
        uint256 deadline;
        uint256 nonce;
        bytes32 salt;
        uint16 maxRelayerFeeBps;
        address allowedRelayer;
        bytes32 referralCode;
        bool partialFillAllowed;
    }

    bytes32 internal constant INTENT_TYPEHASH = keccak256(
        "Intent(address maker,address tokenIn,address tokenOut,uint256 amountIn,uint256 minAmountOut,address receiver,uint256 deadline,uint256 nonce,bytes32 salt,uint16 maxRelayerFeeBps,address allowedRelayer,bytes32 referralCode,bool partialFillAllowed)"
    );

    function hash(Intent calldata intent) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                INTENT_TYPEHASH,
                intent.maker,
                intent.tokenIn,
                intent.tokenOut,
                intent.amountIn,
                intent.minAmountOut,
                intent.receiver,
                intent.deadline,
                intent.nonce,
                intent.salt,
                intent.maxRelayerFeeBps,
                intent.allowedRelayer,
                intent.referralCode,
                intent.partialFillAllowed
            )
        );
    }
}
