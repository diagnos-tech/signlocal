// "Show the Sign button only when signlocal is ready": the region is cut out by the developers
// page of the site.

declare const signButton: HTMLButtonElement;
declare const installLink: HTMLAnchorElement;
declare const notice: HTMLElement;

// #region site:status
import { installUrl, onChange, type Status, status } from "@signlocal/sdk";
import { errorText } from "@signlocal/sdk/messages";

function show({ ready, problem }: Status) {
  signButton.hidden = !ready;
  installLink.hidden = problem !== "ExtensionMissing";
  installLink.href = installUrl(); // the extension store of this browser
  notice.textContent = errorText(problem)?.body ?? "";
}
show(await status());
onChange(show);
// #endregion site:status
