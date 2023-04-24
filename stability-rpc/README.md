# stability-rpc

This custom extension extends the functionality of Substrate-based blockchain by providing additional RPC endpoints and features tailored to your specific needs. One such feature is the `stability_getSupportedTokens` RPC, which allows developers to access information about supported tokens in the network.

## Methods

- `stability_getSupportedTokens`

## Example

```sh
$ curl 'http://localhost:9933' \
  -X POST \
  -H "Content-Type: application/json" \
  --data '{"method":"stability_getSupportedTokens","params":[], "id":1,"jsonrpc":"2.0"}'


{"jsonrpc":"2.0","result":{"code":200,"value":["0xdc2b93f3291030f3f7a6d9363ac37757f7ad5c43"]},"id":1}
```
