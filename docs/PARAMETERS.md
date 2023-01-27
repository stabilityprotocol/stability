# Parameters

There are various parameters that can be adjusted to configure a blockchain and tailor it to specific use cases. These parameters can include things like the block size limit, the block time, and the consensus algorithm used to validate transactions. This page will provide an overview of these parameters and explain how they affect the functionality and performance of a blockchain.

## Variables

- **Block Time**: 2s
- **Maximum Block Weight (MBW)**: For calculating it, we assumed that 2/3 of the block time are for computating the block, so, `MS_PER_BLOCK * WEIGHT*PER_MS * 2 / 3` will give us the value.
  - Max Block Weight = 1_333_333_333_333
- **EVM Gas Limit**: ~50_000_000. Frontier assumes that a Gas Unit is equals to 20_000 Weight (`WEIGHT_PER_GAS`), and the blocks would allow till 75% (`NORMAL_DISPATCH_RATIO`) of `Normal` extrinsics in each one. The formula looks like:

```
Gas Limit = NORMAL_DISPATCH_RATIO * MBW / WEIGHT_PER_GAS
Gas Limit = 0.75 * 1_333_333_333_333 / 20_000 = 50_000_000
```

## Links

- https://substrate.dev/
- https://docs.substrate.io/build/tx-weights-fees/
