# Map Substrate Account to Ethereum Account Pallet

This pallet enables you to link your substrate account to an Ethereum account.

1. [Message to sign](#Message-to-sign)
2. [Extrinsics](#extrinsics)
    1. [link_evm_account](#link_evm_account)
    2. [unlink_evm_account](#unlink_evm_account)
3. [Link to smart contract - ERC1271](#link-substrate-account-with-an-evm-smart-contract-with-erc1271)

## Message to sign

If you want to link your account, you must sign a message in this format:

Message to sign: `I consent to bind my ETH address for time ${nonce} in chain: ${chainId}`

- *nonce*: Number of times the account has been linked in this chain.
- *chainId*: Identifier of the chain. You can get it by calling the RPC method `eth_chainId`

## Extrinsics

### link_evm_account

This extrinsic links your substrate account with a given Ethereum account. 

It`s important to know that the extrinsic must be signed, and the signer of the extrinsic is the substrate account which will be linked to the given Ethereum address.

The arguments of this extrinsic are two:

- *address*: Address which will be linked to your substrate account.
- *signature*: Signature of the [message](#Message-to-sign)

### unlink_evm_account

This extrinsic unlinks your substrate account from your linked EVM account.

It`s important to know that the extrinsic must be signed, and the signer of the extrinsic is the substrate account, which will be unlinked to the given Ethereum address.

## Link Substrate account with an EVM Smart Contract with ERC1271

This pallet enables linking your Substrate account to an EVM smart contract using the [ERC1271](https://eips.ethereum.org/EIPS/eip-1271) .

To do this, you have to call the extrinsic [link_evm_account](#link_evm_account), and provide as an address argument the address of the given smart contract, which will be linked.




