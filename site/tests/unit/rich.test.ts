import { describe, expect, test } from "bun:test";
import { escapeHtml, rich, substitute } from "../../src/i18n/rich.ts";

const resolve = (id: string, fragment: string | undefined) => {
  if (id === "unknown") throw new Error("unknown page id");
  return `/${id}.html${fragment ? `#${fragment}` : ""}`;
};

describe("rich", () => {
  test("escapes HTML before anything else", () => {
    expect(rich("<script>alert('x')</script> & \"q\"", resolve)).toBe(
      "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt; &amp; &quot;q&quot;",
    );
  });

  test("converts bold, code and the ink underline", () => {
    expect(rich("Press **Enter**, run `ls` and __sign__.", resolve)).toBe(
      'Press <strong>Enter</strong>, run <code>ls</code> and <span class="ink">sign</span>.',
    );
  });

  test("turns [label](page-id#fragment) into a link built by the resolver", () => {
    expect(rich("See [Help](help) and [models](compatibility#devices).", resolve)).toBe(
      'See <a href="/help.html">Help</a> and <a href="/compatibility.html#devices">models</a>.',
    );
  });

  test("an unknown page id is a build error", () => {
    expect(() => rich("[x](unknown)", resolve)).toThrow("unknown page id");
  });

  test("variables are substituted last and escaped, so they cannot inject markup", () => {
    expect(rich("Hello {name}", resolve, { name: "<b>**x**</b>" })).toBe(
      "Hello &lt;b&gt;**x**&lt;/b&gt;",
    );
  });

  test("a missing variable throws", () => {
    expect(() => rich("Hello {name}", resolve)).toThrow("{name}");
  });

  test("keeps backslashes and percent signs of paths", () => {
    expect(rich("`%LOCALAPPDATA%\\Programs\\signlocal`", resolve)).toBe(
      "<code>%LOCALAPPDATA%\\Programs\\signlocal</code>",
    );
  });
});

describe("substitute and escapeHtml", () => {
  test("fills every occurrence", () => {
    expect(substitute("{a}-{a}-{b}", { a: 1, b: "x" })).toBe("1-1-x");
  });
  test("escapes the five characters", () => {
    expect(escapeHtml(`&<>"'`)).toBe("&amp;&lt;&gt;&quot;&#39;");
  });
});
