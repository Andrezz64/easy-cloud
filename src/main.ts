import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import pinia from "./store";
import "./assets/styles.css";

// Disable default browser context menu globally
document.addEventListener("contextmenu", (e) => e.preventDefault());

const app = createApp(App);

app.use(pinia);
app.use(router);

app.mount("#app");
