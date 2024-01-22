// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface UpgradeRuntimeController {
    function setApplicationBlock(uint32 block) external; // onlyOwner
    function rejectProposedCode() external; // onlyOwner
    function addMemberToTechnicalCommittee(address member) external; // onlyOwner
    function removeMemberFromTechnicalCommittee(address member) external; // onlyOwner
    function getHashOfProposedCode() external view returns (bytes32);
}