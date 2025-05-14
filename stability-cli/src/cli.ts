import yargs from "yargs/yargs";
import { startValidation } from "./commands/start-validation";
import { Result } from "./common/result";

const argv = yargs(process.argv.slice(2))
  .scriptName("stability-cli")
  .command(
    "start-validation",
    `Usage: --seed=<your_seed> --ws=<ws_endpoint>
Example: start-validation --seed="//Alice" --ws="ws://127.0.0.1:9944"
    `
  )
  .wrap(120)
  .options({
    seed: {
      description: "The account seed that is using to validate",
      type: "string",
      required: true,
    },
    ws: {
      default: "ws://127.0.0.1:9944",
      description: "Websocket endpoint to connect, e.g. ws://127.0.0.1:9944",
      required: true,
      type: "string",
    },
  })
  .parseSync();

async function main() {
  let result: Result;

  if (argv._.includes("start-validation")) {
    result = await startValidation(argv.seed, argv.ws);
  } else {
    console.error("No command specified");
    return;
  }

  if (!result.success) {
    console.error("An error ocurred: ", result.error);
    process.exit(1);
  }

  console.info(result.data);
  process.exit(1);
}

main();
