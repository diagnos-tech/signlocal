// Command-line interface of run.mjs.

import { parseArgs } from "node:util";

const USAGE = `usage: node run.mjs --probe <websign-probe> [options]

  --probe <path>     the websign-probe binary to register and run as host (required)
  --chromium <path>  Chromium/Chrome executable (default: the one playwright-core installed)
  --no-register      skip \`websign-probe register\` (the registration already exists,
                     e.g. the Windows MSIX package wrote the registry keys)
  --expect-sign      fail unless a certificate is listed and signs a digest
  --headed           show the browser window instead of running headless
`;

/** Parses argv; exits with the usage text on `--help` or bad input. */
export function parseCli(argv) {
  let values;
  try {
    ({ values } = parseArgs({
      args: argv,
      strict: true,
      options: {
        probe: { type: "string" },
        chromium: { type: "string" },
        "no-register": { type: "boolean", default: false },
        "expect-sign": { type: "boolean", default: false },
        headed: { type: "boolean", default: false },
        help: { type: "boolean", short: "h", default: false },
      },
    }));
  } catch (error) {
    console.error(`${error.message}\n\n${USAGE}`);
    process.exit(2);
  }
  if (values.help || !values.probe) {
    console.error(USAGE);
    process.exit(values.help ? 0 : 2);
  }
  return {
    probe: values.probe,
    chromium: values.chromium,
    register: !values["no-register"],
    expectSign: values["expect-sign"],
    headed: values.headed,
  };
}
