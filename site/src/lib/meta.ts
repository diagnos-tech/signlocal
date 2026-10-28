// The title and description key of each translated page.

import type { MessageKey } from "../generated/message-keys.ts";
import type { PageId } from "./routes.ts";

export const PAGE_META: Readonly<
  Partial<Record<PageId, { title: MessageKey; description: MessageKey }>>
> = {
  home: { title: "meta.home_title", description: "meta.home_description" },
  install: { title: "meta.install_title", description: "meta.install_description" },
  test: { title: "meta.test_title", description: "meta.test_description" },
  activate: { title: "meta.activate_title", description: "meta.activate_description" },
  privacy: { title: "meta.privacy_title", description: "meta.privacy_description" },
  compatibility: {
    title: "meta.compatibility_title",
    description: "meta.compatibility_description",
  },
  help: { title: "meta.help_title", description: "meta.help_description" },
};
