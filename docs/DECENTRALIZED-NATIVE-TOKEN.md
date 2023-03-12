# Decentralized native token

Stability has no native token in the way that there is no emitted token by Stability. Instead, Stability implements a decentralized native token in which the user and the validator will select the token for which the transaction fees will be paid/received. The available fee tokens the user can pay is whitelisted.

# How does the user select the token?

Stability users should select the token in which the fees are to be paid. If no token is specified, the default token will be chosen. For calculating the transaction fee in the fee token, a conversion rate decided by the validator is applied to the native units. For example, the native gas price is 50 gwei, and the fee will be paid in USDC. The total fee would be `total_fees = (validator_USDT_conversion_rate * native_gas_price)`.

To select the token, the user has to interact with `FeeTokenPrecompile` at `0x0000000000000000000000000000000000000803`.

Though a user can select the token in which the fees are paid, this doesn't mean that the experience in every fee token is the same within Stability. For instance, if the user selects a fee token with an acceptance rate of 20%, its effective block time would be `2 seconds / 20% = 10s`.

# How does the validator select the token?

Stability validators have to select which tokens want to accept as fees. The validators can accept all the allowed fee tokens though they will only accept the default token` at first. There is no need for the validators to accept the default token, but they should update their acceptance of this token. Along with the token acceptance, the validator should set up the conversion rate of the token that be used when a user selects this token to pay the fees.

To select the token and its conversion rate, the user has to interact with `ValidatorFeeManagerPrecompile` at `0x0000000000000000000000000000000000000802`. Since validators have their keys in the `sr25519` format, they cannot interact directly with the EVM (uses `ed25519` format). Validators should link their `sr25519` key with an EVM address.

## Map validators address to EVM address

todo gabi
