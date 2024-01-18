// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ValidatorController {
    function addValidator(address validator) external; // onlyOwner

    function removeValidator(address validator) external; // onlyOwner

    function getValidatorList() external view returns (address[] memory);

    function getActiveValidatorList() external view returns (address[] memory);

    function getValidatorMissingBlocks(
        address validator
    ) external view returns (uint256);
}
