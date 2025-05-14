# Custom Balances

This pallet is a custom version of substrate's pallet_balances to enable `pallet_evm` to get access the Decentralized Native Token (DNT) user balance.

## Mock pallet

This pallet is not intented to have a similar behaviour to `pallet_balances` instead this pallet implements the same interface mocking writing functions and actually implmenting read functions using DNT user balance.
