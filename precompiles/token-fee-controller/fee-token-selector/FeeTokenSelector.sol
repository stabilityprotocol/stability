// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface FeeTokenSelector {
    function setFeeToken(address tokenAddress) external;

    function getFeeToken(address user) external view returns (address);
}
