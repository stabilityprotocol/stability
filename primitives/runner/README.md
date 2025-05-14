# Runner

This primitive takes the `pallet_evm::runner::stack::Runner` trait to provide a custom runner for the EVM pallet.
Basically, it forces the EVM pallet to use the new `StabilityRunner` and forces the `value` always to zero.

This constraint is required because the blockchain doesn't use native tokens for transfers or other scenarios, it also would include custom logics that will handle the value sent by parameter.

When a `Runner::call` is produced and the `input` data is just `0x`, a regular ERC20 `transfer` will be perform sending the token selected by the user, simulating the normal _modus operandi_ of ETH.

## Decentralized Fee Payment

Stability runner has swap the `OnChargeTransaction` frontier's interface for the evm fee payment for the `OnChargeDecentralizedNativeTokenFee` to implement the DNT (Decentralized Native Token) fee payment.
