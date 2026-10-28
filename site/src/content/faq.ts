// The questions people ask, in the order the help page lists them. `home` marks the six the
// home page shows. Keys are written out in full so they can be found and type-checked.

import type { MessageKey } from "../generated/message-keys.ts";

export interface FaqEntry {
  readonly id: string;
  readonly question: MessageKey;
  readonly answer: MessageKey;
  readonly home: boolean;
}

export const FAQ: readonly FaqEntry[] = [
  { id: "free", question: "faq.free_q", answer: "faq.free_a", home: true },
  { id: "safe", question: "faq.safe_q", answer: "faq.safe_a", home: true },
  { id: "certificates", question: "faq.certificates_q", answer: "faq.certificates_a", home: true },
  { id: "phone", question: "faq.phone_q", answer: "faq.phone_a", home: true },
  { id: "warnings", question: "faq.warnings_q", answer: "faq.warnings_a", home: true },
  { id: "background", question: "faq.background_q", answer: "faq.background_a", home: true },
  { id: "formats", question: "faq.formats_q", answer: "faq.formats_a", home: false },
  { id: "websites", question: "faq.websites_q", answer: "faq.websites_a", home: false },
];
