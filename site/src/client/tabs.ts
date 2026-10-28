// ARIA tabs with automatic activation: arrows, Home and End move and select; the panel of the
// selected tab gets .is-active (CSS hides the others when scripts run). Without scripts every
// panel shows. The wizard also calls `selectTab` to open the right tab for a system.

export function selectTab(list: HTMLElement, tabId: string): void {
  for (const tab of list.querySelectorAll<HTMLElement>('[role="tab"]')) {
    const selected = tab.dataset.tab === tabId;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    const panel = document.getElementById(tab.getAttribute("aria-controls") ?? "");
    panel?.classList.toggle("is-active", selected);
  }
}

function move(list: HTMLElement, current: HTMLElement, key: string): HTMLElement | undefined {
  const tabs = [...list.querySelectorAll<HTMLElement>('[role="tab"]')];
  const index = tabs.indexOf(current);
  if (key === "Home") return tabs[0];
  if (key === "End") return tabs.at(-1);
  if (key === "ArrowRight") return tabs[(index + 1) % tabs.length];
  if (key === "ArrowLeft") return tabs[(index - 1 + tabs.length) % tabs.length];
  return undefined;
}

export function initTabs(root: ParentNode = document): void {
  for (const list of root.querySelectorAll<HTMLElement>('[role="tablist"][data-tabs]')) {
    const current = list.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]');
    if (current?.dataset.tab) selectTab(list, current.dataset.tab);
    list.addEventListener("click", (event) => {
      const tab = (event.target as HTMLElement).closest<HTMLElement>('[role="tab"]');
      if (tab?.dataset.tab) selectTab(list, tab.dataset.tab);
    });
    list.addEventListener("keydown", (event) => {
      const tab = (event.target as HTMLElement).closest<HTMLElement>('[role="tab"]');
      const next = tab && move(list, tab, event.key);
      if (!next?.dataset.tab) return;
      event.preventDefault();
      selectTab(list, next.dataset.tab);
      next.focus();
    });
  }
}
