# Unified Accounts

For default, Substrate implements `sr25519` keys for block validators (for Aura) this generates a conflict since EVM addresses are generated with `ecdsa` keys. This is a problem since evm addresses are 20 bytes long and `sr25519` are 32 bytes long so there cannot be defined a biyective relation in which for every address in `ecdsa` has a corresponding address in `sr25519` and viceversa.

# Our solution

Since Stability is strongly `EVM-focused`, not having a common type of addresses damages user experience. For this reason, the validators (for aura) key system was changed to `ecdsa`. However, for granpa (that uses `ed25519`) the key system has keep unchanged since it didn't harm user experience of neither validator nor users.

# Key configuration for validator

Since `ecdsa` accept the same scheme of seeds as `sr25519` or `ed25519` the validator key management is exactly the same for the other type of keys except for one single difference that is that the `AccountId` of an ecdsa key in Stability is not the result of executing the following command.

```bash
./target/release/stability key inspect --scheme ecdsa "$YOUR_SEED"

Network ID:        substrate
Secret seed:       0xfac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e # This is Ethereum private key
Public key (hex):  0x035b26108e8b97479c547da4860d862dc08ab2c29ada449c74d5a9a58a6c46a8c4 # This is Ethereum public key compressed
Account ID:        0xbc9539b36a87a586b1aa20fbe23a1db3ef3edcd65b44a2dc4444cc552687633f # This is not the actual account id
Public key (SS58): KWAmpfYSykMXQe2gavYeydBJ5ZBbjvT1vNjyjSfWf3MTS9MMf # Nevermind for evm
SS58 Address:      5GKyBtzbxKU1qjhZrKpMiwtJj7o6jJcXbKQVtYq74DCPerXN # Nevermind for evm
```

**For avoiding to compile the client or for faster development, you can use the following [online tool](https://stabilityprotocol.github.io/validator-key-generator/) for generating keys.**

For calculating the actual AccountId (EVM address) you have to derive it from the Ethereum public key. Ethers library can easily get this job done:

```ts
const { utils } = require("ethers");

const sampleCompressedPubKey =
  "0x035b26108e8b97479c547da4860d862dc08ab2c29ada449c74d5a9a58a6c46a8c4";

console.log(utils.computeAddress(sampleCompressedPubKey));
```

It is not necessary to install ethers to run this code, in [ethers playground](https://playground.ethers.org/) this functionality is available using the following command:

```js
utils.computeAddress(your_hex_pubkey_compressed);
```

_IMPORTANT: Ethers playground is a third-party so is important not to use any secret or private key on this service._
