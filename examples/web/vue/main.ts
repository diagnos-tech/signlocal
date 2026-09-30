import { createApp } from "vue";
import { setUpFake } from "../shared/fake";
import App from "./App.vue";

// The fake goes in before mounting, so the first status() already sees it.
await setUpFake();
createApp(App).mount("#app");
