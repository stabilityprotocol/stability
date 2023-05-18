# Sponsored Transactions

In common EVM blockchains the executor of a transaction is the payer of the fees, this complicates the onboarding of new users to the blockchain not just because they would have fund their account in order to start to operate but because they have to be invested in the corresponding token. For removing this restriction, sponsored transactions have been implemented.

Sponsored transactions is a new type of transaction in which a third-party pays the fees of an standard EVM transaction. This could be used for several reasons such as creating a BaaS with a monthly plane fee, onboarding processes, ...

## How they work?

Sponsored transactions contains three key elements:

- Signed transaction: A standard user-signed EVM transaction that would demostrate Stability nodes that the user agreed to execute that transaction
- Sponsor nonce: A incrementing counter that would make imposible to execute twice the same metatransaction.
- Sponsored transaction signature: The sponsor needs to sign the message described below so this proof could be used in Stabiliy nodes.

### Sponsor signing message

The message that the sponsor has to sign is the following

`I consent to be a sponsor of transaction: ${TransactionHash} with nonce: ${Nonce}`

Note:

- `${TransactionHash}` is a `0x` prefixed hexadecimal string
- `Nonce` is a decimal number

### API

For executing a sponsored transaction a new a Stability's RPC method has been created and one auxiliary method:

`stability_sendSponsoredTransaction`:

- It receives three arguments:
  - Raw signed transaction (same format as in eth_sendRawTransaction)
  - Sponsor nonce: The count of sponsored transaction by the sponsor
  - Sponsor address: Sponsor of the transaction
  - Sponsor signaure: Signature of the signing sponsored transaction message
- It submits a transaction to the mempool as long as the transaction met all the prechecks

`stability_getSponsorNonce`:

- It receives one arguments:
  - Sponsor address
- It returns the count of sponsored transactions by the given address
