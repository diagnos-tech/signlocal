/** The toolbar popup (docs/ux.md §9): asks the background for the state and renders it. */

import { installFixture } from "../../popup/fixture";
import { render } from "../../popup/view";

// `wxt build --mode fixtures` only (scripts/screenshots.ts); dead code otherwise.
if (import.meta.env.MODE === "fixtures") installFixture(location.search);
render(document.getElementById("app"));
