# Decentralized native token

Stability has no native token in the way that there is no emitted token by Stability. Instead, Stability implements a decentralized native token in which the user and the validator will select the token for which the transaction fees will be paid/received. The available fee tokens the user can pay is whitelisted.

# How does the user select the token?

Stability users should select the token in which the fees are to be paid. If no token is specified, the default token will be chosen. For calculating the transaction fee in the fee token, a conversion rate decided by the validator is applied to the native units. For example, the native gas price is 50 gwei, and the fee will be paid in USDC with a 1.2x of conversion rate, so `gas_price = 50 * 1.2 / 1e18 = 0.00000006 USDC`. The Total Fee math would be `total_fee = gas_used * (base_fee + priority_fee) * conversion_rate`, and for the Gas Price `gas_price = (base_fee + priority_fee) * conversion_rate`.

To select the token, the user has to interact with `FeeTokenPrecompile` at `0x0000000000000000000000000000000000000803`.

Though a user can select the token in which the fees are paid, this doesn't mean that the experience in every fee token is the same within Stability. For instance, if the user selects a fee token with an acceptance rate of 20%, its effective block time would be `2 seconds / 20% = 10s`.

# How does the validator select the token?

Stability validators have to select which tokens want to accept as fees. The validators can accept all the allowed fee tokens though they will only accept the default token` at first. There is no need for the validators to accept the default token, but they should update their acceptance of this token. Along with the token acceptance, the validator should set up the conversion rate of the token that be used when a user selects this token to pay the fees.

To select the token and its conversion rate, the user has to interact with `ValidatorFeeManagerPrecompile` at `0x0000000000000000000000000000000000000802`. Since validators have their keys in the `sr25519` format, they cannot interact directly with the EVM (uses `ed25519` format). Validators should link their `sr25519` key with an EVM address.

## Map validators address to EVM address

Stability validators have to sign the message:

`I consent to bind my ETH address for time ${nonce} in chain: ${chainId}`

- _nonce_: Number of times the account has been linked in this chain.
- _chainId_: Identifier of the chain. You can get it by calling the RPC method `eth_chainId`

Once the message is signed, validators must submit the signature using the extrinsic `MapSvmEvm.link_evm_account`.

If the extrinsic execution succeeds, the validator account is linked to the EVM account.

If the validators wish to change the linked EVM address, it has two options:

The first is a call to the extrinsic `MapSvmEvm.unlink_evm_account`.

The second option is to call the method `unLink` in the `MapSvmEvmController` EVM contract at `0x0000000000000000000000000000000000000804` with your EVM linked account.

If the validator wants to link a smart contract, it is possible using the ERC1271.

If you want to read a more detailed guide about how to link a substrate account to an EVM account, you can read the pallet `map-svm-evm` docs.
