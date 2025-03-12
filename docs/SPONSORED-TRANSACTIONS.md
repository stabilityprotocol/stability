# Sponsored Transactions

In common EVM blockchains the executor of a transaction is the payer of the fees, this complicates the onboarding of new users to the blockchain not just because they would have fund their account in order to start to operate but because they have to be invested in the corresponding token. For removing this restriction, sponsored transactions have been implemented.

Sponsored transactions is a new type of transaction in which a third-party pays the fees of an standard EVM transaction. This could be used for several reasons such as creating a BaaS with a monthly plane fee, onboarding processes, ...

## How they work?

Sponsored transactions contains two key elements:

- Signed transaction: A standard user-signed EVM transaction that would demostrate Stability nodes that the user agreed to execute that transaction
- Sponsored transaction signature: The sponsor needs to sign the message described below so this proof could be used in Stabiliy nodes.

### Sponsor signing message

The message that the sponsor has to sign is the following

`I consent to be a sponsor of transaction: ${TransactionHash}`

Note:

- `${TransactionHash}` is a `0x` prefixed hexadecimal string

### API

For executing a sponsored transaction a new a Stability's RPC method has been created and one auxiliary method:

`stability_sendSponsoredTransaction`:

- It receives three arguments:
  - Raw signed transaction (same format as in eth_sendRawTransaction)
  - Sponsor address: Sponsor of the transaction
  - Sponsor signaure: Signature of the signing sponsored transaction message
- It submits a transaction to the mempool as long as the transaction met all the prechecks

### Example - How-To generate a valid Sponsored Transaction

```typescript
import { ethers, Transaction, ZeroAddress } from "ethers";

// Function to create a transaction that writes "test" to the blockchain
const createTestWritingTransaction = async (
  userWallet: ethers.Wallet | ethers.HDNodeWallet
) => {
  // Creating a transaction that writes "test" to the blockchain
  // We use data field to write "test" as hex
  const txn = {
    nonce: 0, // This would typically come from the user's account
    to: ZeroAddress, // Using zero address, could be a contract
    value: 0,
    gasLimit: 100_000,
    chainId: 20180428,
    maxFeePerGas: ethers.parseUnits("1.2", "gwei"),
    data: ethers.hexlify(ethers.toUtf8Bytes("test")), // Converting "test" to hex
  };

  // User signs the transaction
  const signedTx = await userWallet.signTransaction(txn);
  return Transaction.from(signedTx);
};

// Function to create a sponsor signature for a transaction
const createSponsorSignature = async (
  sponsorWallet: ethers.Wallet | ethers.HDNodeWallet,
  txHash: string
) => {
  // The message format according to the specification
  const message = `I consent to be a sponsor of transaction: ${txHash}`;

  // Create EIP-191 compatible signature (personal_sign format)
  // This prepends the Ethereum signed message prefix
  const messageBytes = ethers.toUtf8Bytes(message);
  const messageHash = ethers.hashMessage(messageBytes);

  // Sign the message hash with the sponsor's wallet
  const signingKey = sponsorWallet.signingKey;
  const signature = signingKey.sign(messageHash);

  // Return the serialized signature
  return signature.serialized;
};

// Main function to create and display a sponsored transaction
const createSponsoredTransaction = async () => {
  try {
    // Create user wallet (in real scenario, this would be the user's wallet)
    const userWallet = ethers.Wallet.createRandom();
    console.log("Generated User Wallet:", userWallet.address);

    // Create sponsor wallet (in real scenario, this would be the sponsor's wallet)
    const sponsorWallet = ethers.Wallet.createRandom();
    console.log("Generated Sponsor Wallet:", sponsorWallet.address);

    // Create and sign the transaction
    const signedTx = await createTestWritingTransaction(userWallet);
    console.log("Transaction created and signed by user");

    // Get transaction hash
    const txHash = ethers.keccak256(signedTx.serialized);
    console.log("Transaction Hash:", txHash);

    // Create sponsor signature
    const sponsorSignature = await createSponsorSignature(
      sponsorWallet,
      txHash
    );
    console.log("Sponsor signature created");

    // Output all the necessary information
    console.log("\n===== SPONSORED TRANSACTION DETAILS =====");
    console.log("User Address:", userWallet.address);
    console.log("Sponsor Address:", sponsorWallet.address);
    console.log("Transaction Hash:", txHash);
    console.log("Raw Transaction:", signedTx.serialized);
    console.log("Sponsor Signature:", sponsorSignature);
    console.log("\n=== FOR RPC CALL ===");
    console.log("stability_sendSponsoredTransaction parameters:");
    console.log("1. Raw Signed Transaction:", signedTx.serialized);
    console.log("2. Sponsor Address:", sponsorWallet.address);
    console.log("3. Sponsor Signature:", sponsorSignature);

    // Return the data (useful if this function is imported elsewhere)
    return {
      userAddress: userWallet.address,
      sponsorAddress: sponsorWallet.address,
      txHash: txHash,
      rawSignedTx: signedTx.serialized,
      sponsorSignature: sponsorSignature,
    };
  } catch (error) {
    console.error("Error creating sponsored transaction:", error);
    throw error;
  }
};

// Run the script directly when executed
if (require.main === module) {
  createSponsoredTransaction()
    .then(() => {
      console.log("Script completed successfully!");
    })
    .catch((error) => {
      console.error("Script failed:", error);
      process.exit(1);
    });
}

// Export the function for potential use in other scripts
export { createSponsoredTransaction };
```
