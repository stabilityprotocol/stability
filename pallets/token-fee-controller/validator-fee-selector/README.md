# Validator fee selector

Enables the validator to update their acceptance of any fee token and its conversion rate.

The conversion rate is a factor configurated by each validator and each fee token that is used to translate the native cost of the transaction to the fee token selected by the user.

The functions in the pallet are exported so the other pallets and precompiles can access them. This gives support to the `ValidatorFeeManagerPrecompile` at `0x0000000000000000000000000000000000000802`.

## Depends on

- SupportedTokensManager: For assuring a validator doesn't accept a not-supported token
