# Decentralized Token Fee Controller

This pallet and, more precisely, the `OnChargeDecentralizedNativeTokenFee` interface manages the fee payment in Stability.

## Shared Revenue

One of the unique characteristics of Stability is that fees are distributed among validators and dApps. Each part would receive a portion of the fee. `ValidatorPercentage` determines what percentage of the fee belongs to the validator.

Through `FeeVaultPrecompiles`, the owner can change this value, is now set to `50%.`

Note: Fees are not distributed directly, are stored in `FeeVaultPrecompiles` where every user can claim their accrued fees.
