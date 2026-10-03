/**
 * i18n scaffold (Phase C — per feature-complete-v1.md §9: framework
 * adopted, EN-only ship, locale #2 added later).
 *
 * Usage: t("nav.documents") — falls back to the key when a translation is
 * missing (which is every key until strings are extracted; the dictionaries
 * seed the pattern with the app chrome).
 */
import { createSignal } from "solid-js";

export type Locale = "en";

const DICTS: Record<Locale, Record<string, string>> = {
  en: {
    "nav.home": "Home",
    "nav.dashboard": "Dashboard",
    "nav.documents": "Documents",
    "nav.spaces": "Spaces",
    "nav.graph": "Graph",
    "nav.tags": "Tags",
    "nav.teams": "Teams",
    "nav.search": "Search",
    "nav.import": "Import",
    "nav.local": "Local Documents",
    "nav.servers": "Server Connections",
    "nav.catalog": "Catalog",
    "nav.templates": "Templates",
    "nav.plugins": "Plugins",
    "nav.ssg": "Static Site",
    "nav.billing": "Billing",
    "nav.audit": "Audit Log",
    "nav.admin": "Admin",
    "nav.settings": "Settings",
    "nav.profile": "Profile",
    "app.signout": "Sign Out",
    "app.signin": "Sign in",
    "app.newDocument": "+ New Document",
  },
};

const [locale, setLocale] = createSignal<Locale>("en");

export const i18n = {
  locale,
  setLocale(l: Locale) {
    setLocale(l);
  },

  t(key: string, vars?: Record<string, string | number>): string {
    const dict = DICTS[locale()] ?? DICTS.en;
    let s = dict[key] ?? DICTS.en[key] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        s = s.replaceAll(`{${k}}`, String(v));
      }
    }
    return s;
  },
};
