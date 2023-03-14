// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface MapSvmEvmController {
    function linkOf(address) external view returns (bytes32);
    function unLink() external returns (bool);
}