// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

address constant DELEGATED_TRANSACTION_ADDRESS = 0x000000000000000000000000000000000000080a;

DelegatedTransaction constant DELEGATED_TRANSACTION_CONTRACT = DelegatedTransaction(DELEGATED_TRANSACTION_ADDRESS);

interface DelegatedTransaction {
    function dispatch(
        address from,
        address to,
        uint256 value,
        bytes memory data,
        uint64 gaslimit,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external returns (bytes memory output);

    function nonces(address owner) external view returns (uint256);

    function DOMAIN_SEPARATOR() external view returns (bytes32);
}