// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface SupportedTokensManager {
    function addToken(address token, bytes32 slot) external;

    function isTokenSupported(address token) external returns (bool);

    function removeToken(address token) external;
}
