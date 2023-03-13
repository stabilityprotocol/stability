# ERC20 Manager

One of requirements for using an ERC20 for paying fees is to modify the contract's storage in order to reflect this changes in the balances. This pallet is responsible of manage memory access and control overflow and underflow.

This pallet exports the implementation of the trait ERC20Manager that could be used by other pallets.

## Depends on

- SupportedTokensManager: For retrieving the balance slot configurated
