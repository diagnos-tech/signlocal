// "React and Vue": how a component uses the hook; the region is cut out by the developers page
// of the site. The hook itself is react/useSignlocal.ts.

import type { Prepare } from "@signlocal/sdk";
import { useSignlocal } from "../react/useSignlocal";

declare const prepare: Prepare;

export function SignButton() {
  // #region site:react-usage
  const { status, signing, sign } = useSignlocal();

  return (
    <button
      type="button"
      disabled={!status?.ready || signing}
      onClick={() => sign({ hash: "SHA-256", prepare })}
    >
      Sign
    </button>
  );
  // #endregion site:react-usage
}
