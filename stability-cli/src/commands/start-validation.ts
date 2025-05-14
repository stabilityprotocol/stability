import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { config } from "../config";
import { Result, Ok, Err } from "../common/result";
import { z } from "zod";

const validatorCodec = z.array(z.string());

export async function startValidation(
  seed: string,
  ws: string
): Promise<Result> {
  const provider = new WsProvider(ws);
  const api = await ApiPromise.create({ provider });
  const keyring = new Keyring({
    type: config.addressType,
    ss58Format: config.addressPrefix,
  });
  const account = keyring.createFromUri(seed);

  const approvedValidatorsResponse = await (
    await api.query.validatorSet.approvedValidators()
  ).toHuman();
  const parsedApprovedValidators = validatorCodec.safeParse(
    approvedValidatorsResponse
  );
  if (!parsedApprovedValidators.success) {
    return Err("Could not parse approved validators");
  }

  const approvedValidators = parsedApprovedValidators.data;

  if (!approvedValidators.includes(account.address)) {
    return Err("The account is not an approved validator");
  }

  const sessionKeys = await api.rpc.author.rotateKeys();

  const setKeysExtrinsic = api.tx.session.setKeys(sessionKeys, "0x");
  await setKeysExtrinsic.signAndSend(account, {
    nonce: -1,
  });

  const validatorsResponse = await (
    await api.query.validatorSet.validators()
  ).toHuman();
  const parsedValidators = validatorCodec.safeParse(validatorsResponse);

  if (!parsedValidators.success) {
    return Err("Could not parse validators");
  }

  const validators = parsedValidators.data;

  if (validators.includes(account.address)) {
    return Err("The account is already a validator");
  }

  const addValidatorAgainExtrinsic = api.tx.validatorSet.addValidatorAgain(
    account.address
  );

  await addValidatorAgainExtrinsic.signAndSend(account, { nonce: -1 });

  return Ok("Validator configured successfully");
}
