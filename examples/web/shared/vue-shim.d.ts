// Lets `tsc` type-check imports of single-file components (vite compiles them).
declare module "*.vue" {
  import type { DefineComponent } from "vue";

  const component: DefineComponent;
  export default component;
}
