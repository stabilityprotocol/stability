# Stability: Transaction Sorting in Stability

## Table of Contents

1. [Introduction](#introduction)
2. [Prioritization in Ethereum](#prioritization-in-ethereum)
3. [The Problem in Stability](#the-problem-in-stability)
4. [Approach Adopted in Stability](#approach-adopted-in-stability)
5. [Conclusion](#conclusion)

## Introduction

This document explains how transactions are prioritized in **Stability**, as fees can be paid in multiple ERC20 tokens. This feature can cause some confusion when comparing it to prioritization in blockchains like Ethereum, where a single cryptocurrency is used to pay fees.

To understand how transactions are prioritized in Stability, we will first analyze how it is done in Ethereum. Then, we will address the problem that arises in Stability due to the possibility of paying fees in different tokens and the solution adopted.

## Prioritization in Ethereum

In Ethereum, transactions are prioritized based on the **gas_price**.

Validators select transactions with higher reward per unit of gas with more priority.

This way of ordering is clear when all transactions are paid with the same token. However, in Stability, the reward has an additional factor, which is the **conversion_rate**.

## The Problem in Stability

In **Stability**, we cannot order by validator reward, as the reward is not paid in the same token.

Let's consider an example:

```
Transaction A: Pays fees in token X, gas_price_wei = 40
Transaction B: Pays fees in token Y, gas_price_wei = 30
```

The validator uses the conversion rates:

```
conversion_rate_X = 1
conversion_rate_Y = 2
```

Then, in tokens they would be:

```
GAS_PRICE_A = 40 * 1 = 40 X/gas
GAS_PRICE_B = 30 * 2 = 60 Y/gas
```

We cannot prioritize **B** over **A** since they are not the same token. If **A** had a value of $2 and **B** had a value of $0.5, we should prioritize **A** over **B**. But the market price is something external to the chain, and therefore, this value is unknown when prioritizing.

One possible solution would be to use an oracle to know the market value of each token and measure the **gas_price** in ```dollars/gas```. However, in **Stability**, we have decided not to do it because:

1. We do not know the value of each token, and our fees would depend on off-chain computation through an oracle, which can be vulnerable.
2. If we prioritize based on the dollar value, we would not allow validators to have lower fees for some tokens, as it could cause a transaction in the most economical token not to be prioritized, which would create the paradox that that token, which the validator wants to prioritize, is left out of the block because it offers less reward.
3. Since the conversion_rate and the dollar value can change over time, we would have to reorder transactions every time transactions or market value changes, which would add time to block production, and therefore, we would lose performance.

## Approach Adopted in Stability

For the above reasons, in Stability, we have decided to maintain prioritization ignoring the **conversion_rate**. In Stability, we prioritize based on the **gas_price** before being multiplied by each validator's conversion_rate.

Let's go back to the previous example:

```
Transaction A: Pays fees in token X, gas_price_wei = 40
Transaction B: Pays fees in token Y, gas_price_wei = 30
```

The validator uses the conversion rates:

```
conversion_rate_X = 1
conversion_rate_Y = 2
```

Then, in tokens they would be:

```
GAS_PRICE_A = 40 * 1 = 40 X/gas
GAS_PRICE_B = 30 * 2 = 60 Y/gas
```

In this case, we would use the **gas_price** in **wei** to order. Ignoring the **conversion_rate**. Therefore, in this case, transaction A would be prioritized over **transaction B** since **transaction A** has a **gas_price_wei** of 40, and **transaction B** has a **gas_price_wei** of 30.

## Conclusion

Prioritization in Stability faces the challenge of handling multiple ERC20 tokens for fee payment, which generates the need to find an adequate solution to compare and order transactions.

We have identified the problem of the gas_price unit of measurement and the difficulty of comparing transactions that pay fees in different tokens. Despite possible solutions, such as using oracles to obtain the market value of each token, in Stability, we have decided to maintain prioritization similar to Ethereum, not taking into account the conversion_rate.

The approach adopted in Stability allows validators to adjust conversion rates according to their preferences and criteria, rather than depending on an oracle to establish prioritization based on market value.

The possibility of charging different prices than market prices is an additional advantage of this approach since it allows validators to adjust their fees according to their preferences and strategies. This ensures appropriate prioritization of transactions, avoiding problems such as dependence on off-chain data or the need for frequent reorderings. We believe that the validators will be adjusting their conversion rates to fit with the actual price of the computing power and block congestion.

In summary, Stability has adopted a prioritization approach that respects the multitoken nature of the platform and allows validators to make decisions based on their preferences while maintaining a solid foundation for transaction inclusion in blocks.
