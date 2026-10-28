// "Errors": the region is cut out by the developers page of the site.

import type { Prepare } from "@signlocal/sdk";

declare const prepare: Prepare;
declare function showBanner(title: string, body: string): void;

// #region site:errors
import { isSignlocalError, sign } from "@signlocal/sdk";
import { errorText } from "@signlocal/sdk/messages";

export async function signContract() {
  try {
    await sign({ hash: "SHA-256", prepare });
  } catch (error) {
    if (!isSignlocalError(error)) throw error;
    if (error.code === "UserCancelled") return; // nothing to show
    console.warn(error.code, error.hint, error.docsUrl); // for you
    const text = errorText(error); // for the person, localized
    if (text) showBanner(text.title, text.body);
  }
}
// #endregion site:errors
