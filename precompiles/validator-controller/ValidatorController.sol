// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ValidatorController {
    function addValidator(bytes32 validator) external; // onlyOwner
    function removeValidator(bytes32 validator) external; // onlyOwner
}