import { KeypairType } from "@polkadot/util-crypto/types";

interface Config {
  addressPrefix: number;
  addressType: KeypairType;
}
export const config: Config = {
  addressPrefix: 42,
  addressType: "sr25519",
};
