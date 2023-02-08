# Parameters

There are various parameters that can be adjusted to configure a blockchain and tailor it to specific use cases. These parameters can include things like the block size limit, the block time, and the consensus algorithm used to validate transactions. This page will provide an overview of these parameters and explain how they affect the functionality and performance of a blockchain.

## Variables

- **Block Time**: 2s
- **Existential Deposit**: Minimum deposit of an account to exist. Since this minimum deposit would be reduced from the actual account balance we set it to zero.

### Block size limitting

One important feature of every blockchain is to know how the blocks are being formed and limitting their size in order to assure security and latency. Substrate itself already implements this through two different concepts:

- Byte length limit: Limits actual block size

- Weight limit: Weight is a measure for the computational power need to perform some action.

Besides Substrate, Ethereum also implements a block size limitting through `block_gas_limit` that is similar to Substrate's weight limit but it measures the cost of performing an operation in the EVM.

Stability implements a weight limitation and a gas limitation, while the byte-length is not effective since its set up to `u64::MAX`. Since gas and weight are closely related `Frontier` has established the relationship between how much `WEIGHT` costs to compute one unit of `GAS`.

Having this relationship between these two limits, we can create a unique limit that would lead up to a limit in both Substrate-based and EVM-based operations. Since Stability is mainly EVM-focused the block size limit is estalished as a `block_gas_limit` though the same limit (mapped as `Weight`) is to be applied to Substrate operations.

Additionally, in Substrate there is a constant (`WEIGHT_REF_TIME_PER_MILLIS`) that marks how much weight could be processed in a millisecond.

This is useful because introducing some of the blockchain's parameters we could know if certain block specifications are achieveable so when the blockchain is initalized this check is runned. If this check fails the blockchain would exit.

- **COMPUTATION_BLOCK_TIME_RATIO**: How much part of the block time could be spent in processing transactions.
- **MAXIMUM_NORMAL_BLOCK_WEIGHT**: The maximum weight that could be processed given the block time and `COMPUTATION_BLOCK_TIME_RATIO`
  - MAXIMUM_NORMAL_BLOCK_WEIGHT = 1_333_333_333_333
- **Target Gas Limit**: ~50_000_000. Frontier assumes that a Gas Unit is equals to 20_000 Weight (`WEIGHT_PER_GAS`), and the blocks would allow till 75% (`NORMAL_DISPATCH_RATIO`) of `Normal` extrinsics in each one. The formula looks like:

```
Gas Limit = NORMAL_DISPATCH_RATIO * MBW / WEIGHT_PER_GAS
Gas Limit = 0.75 * 1_333_333_333_333 / 20_000 = 50_000_000
```

## Links

- https://substrate.dev/
- https://docs.substrate.io/build/tx-weights-fees/
