import { installUrl, type SignResult } from "@websign/sdk";
import { errorText } from "@websign/sdk/messages";
import { useState } from "react";
import { useWebSign } from "./useWebSign";

export function App() {
  const { status, signing, error, sign } = useWebSign();
  const [text, setText] = useState("I agree to the terms.");
  const [result, setResult] = useState<SignResult>();

  const setup = errorText(status?.problem);
  const failure = error && (errorText(error) ?? { title: "Signing failed", body: error.message });

  async function onSign() {
    const data = new TextEncoder().encode(text);
    setResult(
      await sign({
        hash: "SHA-256",
        prepare: (_certificate, { hash }) => crypto.subtle.digest(hash, data),
      }),
    );
  }

  return (
    <main>
      <h1>Sign a text (React)</h1>
      <p className="muted" role="status">
        {!status
          ? "Checking WebeSign…"
          : setup
            ? `${setup.title}. ${setup.body}`
            : "WebeSign is ready."}
      </p>
      {status?.problem === "ExtensionMissing" && <a href={installUrl()}>Install WebeSign</a>}
      <label htmlFor="text">Text to sign</label>
      <textarea
        id="text"
        rows={3}
        style={{ width: "100%" }}
        value={text}
        onChange={(e) => setText(e.target.value)}
      />
      <p>
        <button type="button" disabled={!status?.ready || signing} onClick={onSign}>
          {signing ? "Confirm in the WebeSign window…" : "Sign with my certificate"}
        </button>
      </p>
      <div aria-live="polite">
        {failure && (
          <div className="card error">
            <strong>{failure.title}</strong>
            <p>{failure.body}</p>
          </div>
        )}
        {result && !signing && !error && (
          <div className="card">
            <p className="ok">
              <strong>Signed</strong> by {result.certificate.displayName}
            </p>
            <p>
              {result.algorithm} with {result.hash}, {result.signature.length}-byte signature.
            </p>
          </div>
        )}
      </div>
    </main>
  );
}
