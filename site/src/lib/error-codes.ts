// The error codes the SDK can reject with, from its own catalog (`HINTS` is a Record over
// ErrorCode, so this list cannot miss one).

import type { ErrorCode } from "../../../sdk/src/generated/ErrorCode.ts";
import { HINTS } from "../../../sdk/src/hints.ts";

export const ERROR_CODES = Object.keys(HINTS) as ErrorCode[];
