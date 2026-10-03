import { render } from "solid-js/web";
import App from "./App";
import { authStore } from "./lib/auth";
import { themeStore } from "./lib/theme";

themeStore.init();

// Restore auth before first paint of guarded routes (token -> /auth/me).
void authStore.restore();

const root = document.getElementById("root");
if (!root) throw new Error("#root missing");
render(() => <App />, root);
