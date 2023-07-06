# Custom RPCs

Our Substrate-based blockchain has been designed to cater to the specific needs of our users and developers. To achieve this, we have extended the core API by incorporating custom Remote Procedure Call (RPC) endpoints that offer additional features and functionality. These custom RPC endpoints provide specialized access to blockchain data and enable more efficient interactions with the underlying platform. By leveraging the power and flexibility of the Substrate framework, our custom RPC endpoints facilitate seamless integration with the core API while offering a tailored solution to cater to the unique requirements of our blockchain ecosystem.

## Endpoints

- The `stability_getValidatorList` endpoint retrieves the current list of validators on the network, providing essential information about the active validator set, which is crucial for understanding the consensus and security dynamics of our Substrate-based blockchain.
- The `stability_getSupportedTokens` endpoint returns a list of tokens supported by the chain, offering a convenient way for developers and users to access information about the available assets within our Substrate-based blockchain ecosystem.
- The `stability_sendSponsoredTransaction` endpoint submits a sponsored transaction to the mempool. For further info, check [sponsored transactions](SPONSORED-TRANSACTIONS.md) documentation.
