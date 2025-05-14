# Business Share Revenue

1. [Introduction](#introduction)
2. [Claim fees](#claim-fees)
   1. [Validator](#validator)
   2. [Dapps](#dapps)

## Introduction

In Stability, a transaction fee is shared between the validator and the destination dapp. The distribution percentage may change over time. You can check it by calling the function ` function getValidatorPercentage() external view returns (uint256)` in the precompile `FeeRewardsVaultController` with address `0x0000000000000000000000000000000000000807`.

When a transaction is validated, the fees are sent to the `FeeRewardsVaultController` precompile. Then the dapp or the validator can claim his fee rewards using the `function claimReward(address holder, address token) external`

## Claim fees

### Validator

A validator can always claim his rewards without the need to be whitelisted. If a validator ceases to be a validator at any time, it cannot claim his rewards in the `FeeRewardsVaultController` precompile.

To claim the fees as a validator, the address of the validator should call the method `function claimReward(address holder, address token) external` and set as holder the validator address.

### Dapps

The dapps have to be whitelisted to claim their rewards. The rewards of the dapps always are sent to the `FeeRewardsVaultController` even if the dapp is not whitelisted. If, in the future, a dapp is whitelisted, it will be able to claim all the rewards earned before being added to the whitelist.

To claim your earnings as a Dapp, there are two options.

Claim the fees from the dapp.
Claim the fees using the owner of the dapp.

In Stability, we develop the option of claiming as the owner to claim the fees without needing to change the dapp code.
To get the owner of the dapp, we call the method `owner()` in the dapp. If the dapp doesn't implement the `owner()` method, Stability considers it has no owner.
