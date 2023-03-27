// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface FeeRewardsVaultController {
    function claimReward(address dapp, address token) external;
    function getClaimableReward(address token) external view returns (uint256);
    function isWhitelisted(address dapp) external view returns (bool);
    function setWhitelisted(address dapp, bool isWhitelisted) external; // onlyOwner
}