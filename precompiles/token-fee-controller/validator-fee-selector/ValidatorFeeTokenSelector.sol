// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ValidatorFeeTokenSelector {
    function setTokenAcceptance(address tokenAddress, bool acceptance) external;

    function validatorSupportsToken(
        address validator,
        address tokenAddress
    ) external view returns (bool);

    function setTokenConversionRate(
        address tokenAddress,
        uint256 numerator,
        uint256 denominator
    ) external;

    function updateDefaultController(address tokenAddress) external;

    function tokenConversionRate(
        address validator,
        address tokenAddress
    ) external view returns (uint256, uint256);
}
