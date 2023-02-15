# Stability Cli

1. [Build](#build)
2. [Commands](#commands)
   1. [start-validation](#start-validation)

Implementation of a cli with some utils for the stability chain.

## Build

```sh
yarn build
```

## Commands

### start-validation

Command to configure a node as validator. If you have a node approved as a validator and you want it to start validating. You must use this command

#### Parameters

- _seed_ : The seed of the account that is configured in your node. The account must be an approved validator for use this command.
- _ws_: The websocket endpoint of your node. This parameter is optional because it has the default value of `ws://127.0.0.1:9944`

#### Usage

```sh
yarn start-validation --seed=<your_seed> --ws=<ws_endpoint>
```

Example:

```sh
yarn start-validation --seed="//Alice"
```
