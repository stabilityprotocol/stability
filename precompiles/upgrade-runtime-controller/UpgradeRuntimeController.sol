// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface UpgradeRuntimeController {
    event MemberAdded(address member);
    event MemberRemoved(address member);

    function setApplicationBlock(uint32 block) external; // onlyOwner
    function rejectProposedCode() external; // onlyOwner
    function getTechnicalCommitteeMembers() external view returns (address[] memory);
    function addMemberToTechnicalCommittee(address member) external; // onlyOwner
    function removeMemberFromTechnicalCommittee(address member) external; // onlyOwner
    function getHashOfProposedCode() external view returns (bytes32);
    function getHashOfCurrentCode() external view returns (bytes32);
}