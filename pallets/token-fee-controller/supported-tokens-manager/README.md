# Supported Tokens Manager

Manages the list of tokens in which fees can be paid and establish one of this list as a default token.

The functions in the pallet are exported so the other pallets and precompiles can access them. This gives support to the `SupportedTokensManagerPrecompile` at `0x0000000000000000000000000000000000000801`.

## Responsible for

- Managing fee tokens on-boarding and off-boarding
- Set up a default token
- Having stored the balance slot value for each fee token

Note: Balance slot is the slot of the contract storage from which all the mapping values are derived
