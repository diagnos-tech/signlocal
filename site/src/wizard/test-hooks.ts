// End-to-end tests drive the live detection with the SDK's fake extension and app:
// `?sdk-fake=<scenario>` installs it before the first status() call. The branch exists only in
// builds made with PUBLIC_SIGNLOCAL_TEST_HOOKS=1 and only runs on localhost, so the published
// site contains no fake.

import type { FakeScenario } from "@signlocal/sdk/testing";

export async function installTestHooks(scenario: string | null): Promise<void> {
  if (import.meta.env.PUBLIC_SIGNLOCAL_TEST_HOOKS !== "1") return;
  if (!scenario || location.hostname !== "localhost") return;
  const { installFakeSignlocal } = await import("@signlocal/sdk/testing");
  await installFakeSignlocal({ scenario: scenario as FakeScenario });
}
