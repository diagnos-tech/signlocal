// The language menus: close on Escape or a click outside, and keep the page's query and hash
// on the links to the other languages (the wizard's choices travel with the visitor).

const MENUS = "details[data-language-menu], details[data-menu]";

function closeAll(except?: Element): void {
  for (const menu of document.querySelectorAll<HTMLDetailsElement>(MENUS)) {
    if (menu !== except) menu.open = false;
  }
}

/** Rewrites each language link to ?lang=<code> plus the page's current parameters and hash. */
export function refreshLanguageLinks(menu: ParentNode): void {
  const current = new URLSearchParams(location.search);
  current.delete("lang");
  for (const link of menu.querySelectorAll<HTMLAnchorElement>("a[data-language-link]")) {
    const target = new URL(link.href, location.href);
    const lang = target.searchParams.get("lang");
    const params = new URLSearchParams(current);
    if (lang) params.set("lang", lang);
    target.search = params.toString();
    target.hash = location.hash;
    link.href = target.pathname + target.search + target.hash;
  }
}

export function initLanguageMenus(): void {
  for (const menu of document.querySelectorAll<HTMLDetailsElement>("details[data-language-menu]")) {
    menu.addEventListener("toggle", () => {
      if (menu.open) refreshLanguageLinks(menu);
    });
  }
  for (const menu of document.querySelectorAll<HTMLDetailsElement>(MENUS)) {
    menu.addEventListener("toggle", () => {
      if (menu.open) closeAll(menu);
    });
  }
  document.addEventListener("click", (event) => {
    const inside = (event.target as Element).closest(MENUS);
    closeAll(inside ?? undefined);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const open = document.querySelector<HTMLDetailsElement>(`${MENUS}[open]`);
    if (!open) return;
    open.open = false;
    open.querySelector<HTMLElement>("summary")?.focus();
  });
}
