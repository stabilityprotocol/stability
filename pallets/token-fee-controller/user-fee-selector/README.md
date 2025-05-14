# User fee selector

Enables the user to change the token in which they are paying their fees.

The functions in the pallet are exported so the other pallets and precompiles can access them. This gives support to the `FeeTokenPrecompile` `0x0000000000000000000000000000000000000803`.

## Depends on

- SupportedTokensManager: For assuring a user doesn't change to a not-supported token
