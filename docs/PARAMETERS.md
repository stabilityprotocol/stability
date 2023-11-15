# Parameters

There are various parameters that can be adjusted to configure a blockchain and tailor it to specific use cases. These parameters can include things like the block size limit, the block time, and the consensus algorithm used to validate transactions. This page will provide an overview of these parameters and explain how they affect the functionality and performance of a blockchain.

## Variables

- **Block Time**: 2s
- **Existential Deposit**: Minimum deposit of an account to exist. Since this minimum deposit would be reduced from the actual account balance we set it to zero.
- **Default Base Fee**: The fixed value of the base fee in terms of internal fee units. For further info, check [DNT](./DECENTRALIZED-NATIVE-TOKEN.md).
- **Default Elasticity**: How much does the base fee vary with network demand. Since base fee is fixed then `elasticity = 0`.

### Block size limiting

One important feature of every blockchain is to know how the blocks are being formed and limiting their size in order to assure security and latency. Substrate itself already implements this through two different concepts:

- Byte length limit: Limits actual block size

- Weight limit: Weight is a measure for the computational power need to perform some action.

Besides Substrate, Ethereum also implements a block size limiting through `block_gas_limit` that is similar to Substrate's weight limit but it measures the cost of performing an operation in the EVM.

Stability implements a weight limitation and a gas limitation, while the byte-length is not effective since its set up to `u64::MAX`. Since gas and weight are closely related `Frontier` has established the relationship between how much `WEIGHT` costs to compute one unit of `GAS`.

Having this relationship between these two limits, we can create a unique limit that would lead up to a limit in both Substrate-based and EVM-based operations. Since Stability is mainly EVM-focused the block size limit is estalished as a `block_gas_limit` though the same limit (mapped as `Weight`) is to be applied to Substrate operations.

Additionally, in Substrate there is a constant (`WEIGHT_REF_TIME_PER_MILLIS`) that marks how much weight could be processed in a millisecond.

Using this value and estimating the share of the block time that is spent in computing it we reach to the block gas limit

- **COMPUTATION_BLOCK_TIME_RATIO**: How much part of the block time could be spent in processing transactions.
- **MAXIMUM_BLOCK_WEIGHT**: The maximum weight that could be processed given the block time and `COMPUTATION_BLOCK_TIME_RATIO`
  - MAXIMUM_NORMAL_BLOCK_WEIGHT = 8_000_000_000_000
- **Block Gas Limit**: ~300_000_000. Frontier assumes that a Gas Unit is equals to 20_000 Weight (`WEIGHT_PER_GAS`), and the blocks would allow till 75% (`NORMAL_DISPATCH_RATIO`) of `Normal` extrinsics in each one. The formula looks like:

```
Gas Limit = NORMAL_DISPATCH_RATIO * MBW / WEIGHT_PER_GAS
Gas Limit = 0.75 * 8_000_000_000_000 / 20_000 = 300_000_000
```

## Transactions

This section provides an overview of two key parameters in our blockchain: the Transaction Gas Limit and the Gas Price.

- Transaction Gas Limit: `260_000_000` gas units. This is the maximum amount of gas that a transaction can consume. This is a hard limit and cannot be changed. This limit is set to prevent DoS attacks on the network to avoid having transactions that fill the block and prevent other transactions from being included.
- Gas Price: Fixed to `1 gwei`. This limit can be changed by the users to get their transactions mined faster or slower. This Gas Price is not definitive due to features of the blockchain like the Decentralized Native Token and the Conversion Ratios. For further info, check [DNT](./DECENTRALIZED-NATIVE-TOKEN.md) and [BSR](./BUSINESS-SHARE-REVENUE.md).
  - When the Gas Price is set to 0 then the transaction is not paying fees and it is not included in the block. The only way to make this work is through the [Zero Gas Transaction](./ZERO-GAS-TRANSACTIONS.md) flow.

## Links

- https://substrate.dev/
- https://docs.substrate.io/build/tx-weights-fees/
