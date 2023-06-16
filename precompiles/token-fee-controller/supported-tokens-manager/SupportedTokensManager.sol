// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface SupportedTokensManager {
    function addToken(address token, bytes32 slot) external;

    function supportedTokens() external view returns (address[] memory);

    function isTokenSupported(address token) external view returns (bool);

    function removeToken(address token) external;

    function updateDefaultToken(address token) external;

    function defaultToken() external view returns (address);
}
