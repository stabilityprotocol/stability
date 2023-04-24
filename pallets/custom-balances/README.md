# Custom Balances

This pallet is a mocked version of substrate's pallet_balances to enable `pallet_evm` to get access the Decentralized Native Token (DNT) user balance.

## Useful functions

There are only three functions really working, we could consider the rest of them mocks.

- `total_balance(address)`: Calculates the total balance of a user in its selected "native" token.
- `reducible_balance(address)`: Idem as `total_balance`.
- `free_balance(address)`: Idem as `total_balance`.
