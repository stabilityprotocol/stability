// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface FeeRewardsVaultController {
    event RewardClaimed(address dapp, address claimer, address token);
    event WhitelistStatusUpdated(address dapp, bool isWhitelisted);
    event ValidatorPercentageUpdated(uint256 validatorPercentage);
    event TransactionFee(
        address token,
        uint256 totalFee,
        address validator,
        uint256 validatorFee,
        address dapp,
        uint256 dappFee
    );

    function claimReward(address dapp, address token) external;

    function setWhitelisted(address dapp, bool isWhitelisted) external; // onlyOwner

    function canClaimReward(address, address) external view returns (bool);

    function getClaimableReward(
        address dapp,
        address token
    ) external view returns (uint256);

    function isWhitelisted(address dapp) external view returns (bool);

    function getValidatorPercentage() external view returns (uint256);

    function setValidatorPercentage(uint256) external returns (bool);
}
