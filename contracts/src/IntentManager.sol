// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IntentTypes} from "./IntentTypes.sol";

interface IERC1271 {
    function isValidSignature(bytes32 hash, bytes memory signature) external view returns (bytes4);
}

contract IntentManager {
    using IntentTypes for IntentTypes.Intent;

    bytes32 internal constant EIP712_DOMAIN_TYPEHASH =
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
    uint256 internal constant BPS_DENOMINATOR = 10_000;
    bytes4 internal constant EIP1271_MAGIC_VALUE = 0x1626ba7e;
    uint256 internal constant SECP256K1_HALF_N =
        0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0;

    string public domainName;
    string public domainVersion;

    mapping(address => uint256) public nonces;
    mapping(bytes32 => bool) public cancelled;
    mapping(bytes32 => bool) public consumed;

    event IntentCancelled(bytes32 indexed intentHash, address indexed maker, uint256 nextNonce);
    event IntentConsumed(bytes32 indexed intentHash, address indexed maker, address indexed relayer, uint256 nextNonce);

    constructor(string memory name_, string memory version_) {
        domainName = name_;
        domainVersion = version_;
    }

    function domainSeparator() public view returns (bytes32) {
        return keccak256(
            abi.encode(
                EIP712_DOMAIN_TYPEHASH,
                keccak256(bytes(domainName)),
                keccak256(bytes(domainVersion)),
                block.chainid,
                address(this)
            )
        );
    }

    function getIntentHash(IntentTypes.Intent calldata intent) public pure returns (bytes32) {
        return intent.hash();
    }

    function getIntentDigest(IntentTypes.Intent calldata intent) public view returns (bytes32) {
        return _toTypedDataHash(intent.hash());
    }

    function validateIntent(
        IntentTypes.Intent calldata intent,
        bytes calldata signature
    ) public view returns (bytes32 intentHash) {
        require(intent.deadline >= block.timestamp, "intent expired");
        require(intent.maxRelayerFeeBps <= BPS_DENOMINATOR, "fee too high");
        require(intent.nonce == nonces[intent.maker], "invalid nonce");

        intentHash = intent.hash();
        require(!cancelled[intentHash], "intent cancelled");
        require(!consumed[intentHash], "intent already consumed");

        bool isValidSigner = _isValidSigner(intent.maker, _toTypedDataHash(intentHash), signature);
        require(isValidSigner, "invalid signature");
    }

    function cancelIntent(IntentTypes.Intent calldata intent) external returns (bytes32 intentHash) {
        require(msg.sender == intent.maker, "only maker");
        require(intent.nonce == nonces[intent.maker], "invalid nonce");

        intentHash = intent.hash();
        require(!consumed[intentHash], "intent already consumed");
        require(!cancelled[intentHash], "intent already cancelled");

        cancelled[intentHash] = true;
        nonces[intent.maker] = intent.nonce + 1;

        emit IntentCancelled(intentHash, intent.maker, nonces[intent.maker]);
    }

    function _consumeIntent(bytes32 intentHash, address maker) internal {
        consumed[intentHash] = true;
        nonces[maker] += 1;
        emit IntentConsumed(intentHash, maker, msg.sender, nonces[maker]);
    }

    function _toTypedDataHash(bytes32 structHash) internal view returns (bytes32) {
        return keccak256(abi.encodePacked("\x19\x01", domainSeparator(), structHash));
    }

    function _isValidSigner(address signer, bytes32 digest, bytes memory signature) internal view returns (bool) {
        if (signer.code.length == 0) {
            return _recoverSigner(digest, signature) == signer;
        }

        try IERC1271(signer).isValidSignature(digest, signature) returns (bytes4 magicValue) {
            return magicValue == EIP1271_MAGIC_VALUE;
        } catch {
            return false;
        }
    }

    function _recoverSigner(bytes32 digest, bytes memory signature) internal pure returns (address) {
        require(signature.length == 65, "bad signature length");

        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(signature, 0x20))
            s := mload(add(signature, 0x40))
            v := byte(0, mload(add(signature, 0x60)))
        }

        if (v < 27) {
            v += 27;
        }

        require(v == 27 || v == 28, "bad signature v");
        require(uint256(s) <= SECP256K1_HALF_N, "bad signature s");

        address signer = ecrecover(digest, v, r, s);
        require(signer != address(0), "bad signature");
        return signer;
    }
}
