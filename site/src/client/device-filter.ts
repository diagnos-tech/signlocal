// The filter of the compatibility page's device table: shows the rows whose name or driver
// contains what was typed, and says so when none does. Without JavaScript the whole table shows.

export function matchesFilter(haystack: string, query: string): boolean {
  return haystack.includes(query.trim().toLowerCase());
}

export function initDeviceFilter(): void {
  const input = document.querySelector<HTMLInputElement>("#device-filter");
  const rows = [...document.querySelectorAll<HTMLElement>("[data-device-row]")];
  const none = document.querySelector<HTMLElement>("[data-no-match]");
  if (!input || rows.length === 0) return;
  input.addEventListener("input", () => {
    let shown = 0;
    for (const row of rows) {
      const show = matchesFilter(row.dataset.search ?? "", input.value);
      row.hidden = !show;
      if (show) shown += 1;
    }
    if (none) none.hidden = shown > 0;
  });
}
