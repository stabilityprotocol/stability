# runner

This primitive takes the `pallet_evm::runner::stack::Runner` trait to provide a custom runner for the EVM pallet.
Basically, it forces the EVM pallet to use the new `StabilityRunner` and forces the `value` always to zero.

This constraint is required because the blockchain doesn't use native tokens for transfers or other scenarios, it also would include custom logics that will handle the value sent by parameter.
