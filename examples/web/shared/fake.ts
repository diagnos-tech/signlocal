/**
 * Every example runs on the SDK's testing fake unless the URL has `?real`:
 * then it talks to the real SignLocal extension and app. The panel in the
 * corner scripts what the "person" does next, so each state of your UI can
 * be seen without a token.
 */

import type { ErrorCode } from "@websign/sdk";
import type { FakeScenario, FakeWebSign } from "@websign/sdk/testing";

/** Whether this page should use the real extension (`?real` in the URL). */
export const useRealExtension = new URLSearchParams(location.search).has("real");

/**
 * Installs the fake (unless `?real`) and shows its control panel. The
 * dynamic import keeps `@websign/sdk/testing` out of the bundle that talks to
 * the real extension.
 */
export async function setUpFake(): Promise<FakeWebSign | undefined> {
  if (useRealExtension) return undefined;
  const { installFakeWebSign } = await import("@websign/sdk/testing");
  const fake = await installFakeWebSign({ latencyMs: 600 });
  mountPanel(fake);
  return fake;
}

const SCENARIOS: readonly Exclude<FakeScenario, "extension-missing">[] = [
  "ready",
  "app-missing",
  "app-outdated",
  "extension-outdated",
];
const FAILURES: readonly ErrorCode[] = ["UserCancelled", "PinLocked", "TokenRemoved", "Timeout"];

function select(label: string, placeholder: string, values: readonly string[]) {
  const wrapper = document.createElement("label");
  wrapper.textContent = `${label} `;
  const element = document.createElement("select");
  if (placeholder) element.add(new Option(placeholder, ""));
  for (const value of values) element.add(new Option(value, value));
  wrapper.append(element);
  return { wrapper, element };
}

function mountPanel(fake: FakeWebSign): void {
  const panel = document.createElement("aside");
  panel.className = "fake-panel";
  panel.setAttribute("aria-label", "Fake SignLocal controls");
  const title = document.createElement("strong");
  title.textContent = "FAKE SignLocal";
  const scenario = select("Computer:", "", SCENARIOS);
  scenario.element.addEventListener("change", () =>
    fake.setScenario(scenario.element.value as (typeof SCENARIOS)[number]),
  );
  const next = select("Next request:", "succeeds", FAILURES);
  const armed = document.createElement("span");
  armed.setAttribute("role", "status");
  next.element.addEventListener("change", () => {
    const code = next.element.value as ErrorCode;
    fake.failNext(code);
    armed.textContent = `${code} armed (once)`;
    next.element.value = "";
  });
  const real = document.createElement("a");
  real.href = "?real";
  real.textContent = "Use the real extension";
  panel.append(title, scenario.wrapper, next.wrapper, armed, real);
  document.body.append(panel);
}
