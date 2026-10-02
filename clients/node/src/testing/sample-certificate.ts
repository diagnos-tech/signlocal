import type { FakeCertificate } from "./fake-app.js";

/**
 * The certificate the fake app offers unless you pass your own: an ECDSA
 * P-256 key on a token. Every field is obviously fictional.
 */
export const SAMPLE_CERTIFICATE: FakeCertificate = {
  der: "AQID",
  chain: [],
  fingerprint: "ab".repeat(32),
  displayName: "Test Holder",
  issuerName: "Test CA",
  notBefore: 1741000000,
  notAfter: 4102444800,
  key: { type: "EC", curve: "P-256" },
  algorithms: ["ECDSA"],
  profile: { keyStorage: "hardware" },
};
