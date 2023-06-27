// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ValidatorFeeTokenSelector {
    function setTokenAcceptance(address tokenAddress, bool acceptance) external;

    function validatorSupportsToken(
        address validator,
        address tokenAddress
    ) external view returns (bool);

    function updateConversionRateController(address controller) external;

    function updateDefaultController(address tokenAddress) external;

    // view functions
    function conversionRateController(address validator) external view returns (address);

    function defaultController() external view returns (address);
}
