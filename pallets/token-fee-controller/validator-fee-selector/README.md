# Validator fee selector

Enables the validator to update their acceptance of any fee token and its conversion rate.

The conversion rate is a factor configurated by each validator and each fee token that is used to translate the native cost of the transaction to the fee token selected by the user.

## Depends on

- SupportedTokensManager: For assuring a validator doesn't accept a not-supported token
