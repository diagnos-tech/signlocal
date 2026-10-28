import { describe, expect, it } from "vitest";
import * as client from "../src/index";

describe("public API", () => {
  it("exports the client class and helpers", () => {
    expect(typeof client.Signlocal.connect).toBe("function");
    expect(typeof client.findExecutable).toBe("function");
    expect(typeof client.SignlocalError).toBe("function");
  });
});
