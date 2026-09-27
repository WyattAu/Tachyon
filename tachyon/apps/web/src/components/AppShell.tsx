import { For, Show, type ParentProps } from "solid-js";
import { A } from "@solidjs/router";
import { authStore } from "@/lib/auth";

/**
 * App shell — sidebar + topbar, ported from the Leptos dashboard layout.
 * Navigation items mirror the 34-route table (ADR-113 parity tracking).
 * Phase B fills pages in; unwired routes render <Migrating/>.
 */

export interface NavItem {
  label: string;
  href: string;
  section: "main" | "server" | "local" | "admin" | "account";
}

export const NAV: NavItem[] = [
  { label: "Home", href: "/", section: "main" },
  { label: "Dashboard", href: "/dashboard", section: "main" },
  { label: "Documents", href: "/documents", section: "server" },
  { label: "Spaces", href: "/spaces", section: "server" },
  { label: "Graph", href: "/graph", section: "server" },
  { label: "Tags", href: "/tags", section: "server" },
  { label: "Teams", href: "/teams", section: "server" },
  { label: "Search", href: "/search", section: "server" },
  { label: "Import", href: "/import", section: "server" },
  { label: "Local Documents", href: "/local", section: "local" },
  { label: "Server Connections", href: "/servers", section: "local" },
  { label: "Catalog", href: "/catalog", section: "admin" },
  { label: "Templates", href: "/templates", section: "admin" },
  { label: "Plugins", href: "/plugins", section: "admin" },
  { label: "Static Site", href: "/ssg", section: "admin" },
  { label: "Billing", href: "/billing", section: "admin" },
  { label: "Audit Log", href: "/audit", section: "admin" },
  { label: "Admin", href: "/admin/roles", section: "admin" },
  { label: "Settings", href: "/settings", section: "account" },
  { label: "Profile", href: "/profile", section: "account" },
];

const SECTIONS: Array<NavItem["section"] | "main"> = [
  "main",
  "server",
  "local",
  "admin",
  "account",
];

const SECTION_LABEL: Record<string, string> = {
  main: "",
  server: "Server",
  local: "Local",
  admin: "Admin",
  account: "Account",
};

export function AppShell(props: ParentProps) {
  return (
    <div class="flex min-h-screen">
      <aside class="hidden w-64 flex-shrink-0 flex-col border-r border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-950 md:flex">
        <div class="flex items-center gap-2 border-b border-gray-200 p-4 dark:border-gray-700">
          <div class="flex h-8 w-8 items-center justify-center rounded bg-blue-600 font-bold text-white">
            T
          </div>
          <span class="font-display text-lg font-semibold">Tachyon</span>
        </div>
        <nav class="flex-1 overflow-y-auto p-2">
          <For each={SECTIONS}>
            {(section) => (
              <div class="mb-2">
                <Show when={SECTION_LABEL[section]}>
                  <p class="px-3 pt-3 pb-1 text-[10px] font-bold tracking-widest text-gray-400 uppercase">
                    {SECTION_LABEL[section]}
                  </p>
                </Show>
                <For each={NAV.filter((n) => n.section === section)}>
                  {(item) => (
                    <A href={item.href} class="block rounded-none px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800" activeClass="bg-blue-50 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300" end={item.href === "/"}>
                      {item.label}
                    </A>
                  )}
                </For>
              </div>
            )}
          </For>
        </nav>
        <div class="border-t border-gray-200 p-2 dark:border-gray-700">
          <button
            class="w-full rounded-none px-3 py-2 text-left text-sm text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800"
            onClick={() => {
              authStore.logout();
              location.href = "/login";
            }}
          >
            Sign Out ({authStore.user()?.username ?? "?"})
          </button>
        </div>
      </aside>
      <main class="min-w-0 flex-1">{props.children}</main>
    </div>
  );
}
