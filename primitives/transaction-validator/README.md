# Transaction Validator

## Overview

The Transaction Validator module provides utilities for validating Ethereum transactions within the Stability blockchain framework. It serves as a critical infrastructure component that ensures transactions meet specific criteria before being included in blocks.

## Key Features

- **Balance Verification**: Validates that transaction senders have sufficient balance to cover gas costs and transaction value
- **Custom Fee Token Support**: Works with Stability's user-defined fee token system
- **Gas Price Validation**: Ensures that gas prices meet minimum requirements
- **Transaction Pool Integration**: Provides validity checks for transactions in the pool
- **Nonce Management**: Prevents transaction replay attacks and ensures proper transaction ordering

## Main Components

### FallbackTransactionValidator

The core component that handles transaction validation logic. It works with both standard Ethereum transactions and custom transaction types in the Stability ecosystem.

Key methods:
- `check_actual_balance`: Verifies that a sender has sufficient balance to cover a transaction's costs
- `check_actual_balance_raw`: Similar validation but using raw fee amounts
- `build_validity_success_transaction`: Constructs a valid transaction response when checks pass

### StbleCheckNonce

A specialized SignedExtension implementation that manages transaction nonces to prevent replay attacks. It differs from Substrate's standard CheckNonce by removing account creation checks for compatibility with Stability's DNT (Decentralized Native Token) system.

Key functionality:
- **Nonce Verification**: Ensures transactions are executed in the correct order by verifying the nonce matches the account's current nonce
- **Nonce Increment**: Automatically increments the account's nonce after successful transaction execution
- **Transaction Ordering**: Creates appropriate `requires` and `provides` tags to maintain proper transaction sequencing in the pool
- **Replay Protection**: Prevents transaction replay by rejecting transactions with already-used nonces

## Integration Points

The Transaction Validator integrates with several other components:
- **EVM Module**: For gas price calculations and transaction execution
- **User Fee Selector**: To determine which token a user is paying fees with
- **Ethereum Compatibility Layer**: For handling Ethereum transaction formats
- **Frame System**: For accessing and updating account information including nonces

## Usage in the Ecosystem

This module is particularly important for:
- Zero Gas Transactions
- Sponsored Transactions
- Standard EVM transactions

It helps maintain the integrity of the transaction pool and ensures that all transactions included in blocks are valid according to Stability's economic model.

## Technical Requirements

The module requires implementing types to fulfill the following traits:
- `frame_system::Config`
- `pallet_evm::Config`
- `pallet_ethereum::Config`
- `pallet_user_fee_selector::Config`

## License

Stability is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Stability is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.