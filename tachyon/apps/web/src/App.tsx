import { type Component } from "solid-js";
import { Route, Router, A } from "@solidjs/router";
import { authStore } from "@/lib/auth";
import { AuthGuard } from "@/components/AuthGuard";
import { AppShell } from "@/components/AppShell";
import LoginPage from "@/pages/Login";
import Dashboard from "@/pages/Dashboard";

/**
 * Route table — mirrors the Leptos app's 34 routes (ADR-113 parity).
 * Phase B replaces <Migrating/> stubs with real pages one by one; the
 * parity gate (control-plane screenshot matrix + e2e) retires Leptos routes.
 */

function Migrating(props: { route: string }) {
  return (
    <div class="p-8">
      <h1 class="font-display text-2xl font-bold">{props.route}</h1>
      <p class="mt-2 max-w-lg text-sm text-gray-500 dark:text-gray-400">
        This page is scheduled for migration in Phase B (ADR-113). The Leptos
        implementation remains the source of truth until the parity gate passes.
      </p>
      <code class="mt-4 inline-block rounded bg-gray-100 px-2 py-1 text-xs dark:bg-gray-800">
        {props.route}
      </code>
    </div>
  );
}

function Home() {
  return (
    <div class="p-8">
      <h1 class="font-display text-3xl font-bold">Tachyon</h1>
      <p class="mt-1 text-gray-500 dark:text-gray-400">
        Knowledge Management Platform — SolidJS build (ADR-113)
      </p>
      {authStore.isAuthenticated() ? (
        <A
          href="/dashboard"
          class="mt-4 inline-block rounded-none bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
        >
          Go to Dashboard
        </A>
      ) : (
        <A
          href="/login"
          class="mt-4 inline-block rounded-none bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
        >
          Sign in
        </A>
      )}
    </div>
  );
}

/** Wrap a page in the auth guard + app shell. */
function shell(page: Component): Component {
  return () => (
    <AuthGuard>
      <AppShell>
        {page({})}
      </AppShell>
    </AuthGuard>
  );
}

const migrating = (route: string): Component => shell(() => <Migrating route={route} />);

const App: Component = () => (
  <Router>
    <Route path="/" component={Home} />
    <Route path="/login" component={LoginPage} />
    {/* Phase B migration stubs — each replaced by its real page */}
    <Route path="/dashboard" component={shell(Dashboard)} />
    <Route path="/documents" component={migrating("/documents")} />
    <Route path="/documents/:id" component={migrating("/documents/:id")} />
    <Route path="/documents/:id/edit" component={migrating("/documents/:id/edit")} />
    <Route path="/spaces" component={migrating("/spaces")} />
    <Route path="/graph" component={migrating("/graph")} />
    <Route path="/tags" component={migrating("/tags")} />
    <Route path="/teams" component={migrating("/teams")} />
    <Route path="/teams/:team_id" component={migrating("/teams/:team_id")} />
    <Route path="/search" component={migrating("/search")} />
    <Route path="/import" component={migrating("/import")} />
    <Route path="/catalog" component={migrating("/catalog")} />
    <Route path="/templates" component={migrating("/templates")} />
    <Route path="/plugins" component={migrating("/plugins")} />
    <Route path="/ssg" component={migrating("/ssg")} />
    <Route path="/billing" component={migrating("/billing")} />
    <Route path="/audit" component={migrating("/audit")} />
    <Route path="/admin/roles" component={migrating("/admin/roles")} />
    <Route path="/settings" component={migrating("/settings")} />
    <Route path="/settings/branding" component={migrating("/settings/branding")} />
    <Route path="/settings/webhooks" component={migrating("/settings/webhooks")} />
    <Route path="/settings/notifications" component={migrating("/settings/notifications")} />
    <Route path="/profile" component={migrating("/profile")} />
    <Route path="/local" component={migrating("/local")} />
    <Route path="/servers" component={migrating("/servers")} />
    <Route path="/versions" component={migrating("/versions")} />
    <Route path="/flashcards" component={migrating("/flashcards")} />
    <Route path="/daily" component={migrating("/daily")} />
    <Route path="/daily/:date" component={migrating("/daily/:date")} />
    <Route path="/journal" component={migrating("/journal")} />
    <Route path="/journal/:date" component={migrating("/journal/:date")} />
    <Route path="/canvas" component={migrating("/canvas")} />
    <Route path="/canvas/:id" component={migrating("/canvas/:id")} />
    <Route path="/onboarding" component={migrating("/onboarding")} />
    <Route path="/blog" component={migrating("/blog")} />
    <Route path="/blog/:slug" component={migrating("/blog/:slug")} />
    <Route path="/blog/admin/new" component={migrating("/blog/admin/new")} />
    <Route path="*" component={() => <div class="p-8">404 — Not Found</div>} />
  </Router>
);

export default App;
