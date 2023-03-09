// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface MapSvmEvmController {
    function setTokenAcceptance(address, bool) external;

    function validatorSupportsToken(address, address) external returns (bool);

    function setTokenConversionRate(address, uint256, uint256) external;

    function tokenConversionRate(
        address,
        address
    ) external returns (uint256, uint256);
}
