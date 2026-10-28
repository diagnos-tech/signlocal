// "The same fake lets you build your page before installing anything": the region is cut out by
// the developers page of the site.

// #region site:testing-dev
if (import.meta.env.DEV) {
  const { installFakeSignlocal } = await import("@signlocal/sdk/testing");
  await installFakeSignlocal();
}
// #endregion site:testing-dev
