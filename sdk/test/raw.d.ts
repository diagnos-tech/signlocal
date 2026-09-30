// Vitest (Vite) imports any file as text with `?raw`; used to check the site page the SDK links to.
declare module "*?raw" {
  const text: string;
  export default text;
}
