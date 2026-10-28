// Turns a message into safe HTML: escapes everything first, then converts the few markers the
// copy uses: **bold**, `code`, __ink underline__ and [label](page-id#fragment) links. Variables
// ({name}) are substituted last and escaped, so a product name can never inject markup.

const ESCAPES: Readonly<Record<string, string>> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

export function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, (char) => ESCAPES[char] ?? char);
}

/** Turns a page id and optional fragment into an href; throws on an unknown id. */
export type LinkResolver = (pageId: string, fragment: string | undefined) => string;

export type Variables = Readonly<Record<string, string | number>>;

const LINK = /\[([^\]]+)\]\(([a-z-]+)(?:#([a-z-]+))?\)/g;

export function rich(template: string, resolve: LinkResolver, variables: Variables = {}): string {
  const html = escapeHtml(template)
    .replace(LINK, (_all, label: string, id: string, fragment: string | undefined) => {
      return `<a href="${escapeHtml(resolve(id, fragment))}">${label}</a>`;
    })
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/`(.+?)`/g, "<code>$1</code>")
    .replace(/__(.+?)__/g, '<span class="ink">$1</span>');
  return substitute(html, variables, escapeHtml);
}

/** Replaces {name} placeholders; a missing variable is a bug in the page, so it throws. */
export function substitute(
  template: string,
  variables: Variables,
  encode: (text: string) => string = (text) => text,
): string {
  return template.replace(/\{([a-z_]+)\}/g, (_all, name: string) => {
    const value = variables[name];
    if (value === undefined) throw new Error(`Missing variable {${name}} for "${template}"`);
    return encode(String(value));
  });
}
