import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { setUpFake } from "../shared/fake";
import { App } from "./App";

// The fake goes in before the first render, so the first status() already sees it.
await setUpFake();

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
