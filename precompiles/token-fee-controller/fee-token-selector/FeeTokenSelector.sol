// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface FeeTokenSelector {
    function setFeeToken(address) external;

    function getFeeToken(address) external view returns (address);
}
