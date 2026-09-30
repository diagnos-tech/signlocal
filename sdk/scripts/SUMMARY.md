# sdk/scripts

- `consumer.js` — a site re-exporting the whole public API from `dist/`, the bundle `size.ts` measures
- `size.ts` — fails when the minified + gzipped consumer bundle reaches 5 KB (SPEC §13)
