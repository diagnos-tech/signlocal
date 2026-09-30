/** DOM building blocks of the popup: elements, actions and the status block. */

import type { Action, Card } from "./card";
import { icon } from "./icons";

/** Id of the hidden "opens in a new tab" text that every link points to. */
export const NEW_TAB_HINT_ID = "new-tab-hint";

/** A new `tag` element with an optional class and text. */
export function element<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/**
 * A link that opens `href` in a tab through `openTab` on a plain click
 * (Safari's popover ignores `target=_blank`; the popup should close after),
 * while keeping the browser's own handling of middle and modified clicks.
 */
export function linkNode(
  label: string,
  href: string,
  className: string,
  openTab: (url: string) => void,
  arrow = true,
): HTMLAnchorElement {
  const link = element("a", className);
  link.href = href;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  link.setAttribute("aria-describedby", NEW_TAB_HINT_ID);
  link.append(element("span", undefined, label));
  if (arrow) link.insertAdjacentHTML("beforeend", icon("external", 16));
  link.addEventListener("click", (event) => {
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
      return;
    }
    event.preventDefault();
    openTab(href);
  });
  return link;
}

/** The button or link for `action`. */
export function actionNode(
  action: Action,
  className: string,
  openTab: (url: string) => void,
): HTMLElement {
  if (action.href !== undefined) {
    // The arrow marks the main link only: beside a wrapping secondary label it floats off.
    return linkNode(action.label, action.href, className, openTab, !className.includes("ghost"));
  }
  const button = element("button", className);
  button.type = "button";
  if (action.icon) button.innerHTML = icon(action.icon, 16);
  button.append(element("span", undefined, action.label));
  button.addEventListener("click", () => {
    // The actions are busy while a request they started is in flight (Open diagnostics).
    if (button.closest('[aria-busy="true"]') === null) action.run?.();
  });
  return button;
}

/** The icon, title, text and code of `card`, for the status region. */
export function statusNodes(card: Card): HTMLElement[] {
  const symbol = element("span", `sym ${card.tone}`);
  symbol.innerHTML = icon(card.icon, 22);
  const text = element("div", "txt");
  text.append(element("h1", undefined, card.title));
  if (card.body) text.append(element("p", "muted", card.body));
  if (card.code) text.append(element("p", "mono", card.code));
  return [symbol, text];
}
