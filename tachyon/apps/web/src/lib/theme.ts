import { createSignal } from "solid-js";

/**
 * Theme store (Phase C): dark/light/system + persisted to localStorage.
 * Applied via the `dark` class on <html> (Tailwind strategy, same as the
 * rest of the app). The index.html boot script reads the same key before
 * paint to avoid flashes.
 */

export type ThemeMode = "dark" | "light" | "system";

const KEY = "tachyon_theme";

function initial(): ThemeMode {
  if (typeof localStorage === "undefined") return "dark";
  const v = localStorage.getItem(KEY) as ThemeMode | null;
  return v === "light" || v === "system" ? v : "dark";
}

const [mode, setMode] = createSignal<ThemeMode>(initial());

function apply(m: ThemeMode): void {
  const dark = m === "dark" || (m === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.classList.toggle("dark", dark);
}

export const themeStore = {
  mode,
  apply,

  init(): void {
    apply(mode());
    // follow OS changes while in system mode
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (mode() === "system") apply("system");
    });
  },

  set(m: ThemeMode): void {
    setMode(m);
    localStorage.setItem(KEY, m);
    apply(m);
  },
};
