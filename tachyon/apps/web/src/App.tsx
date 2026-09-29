import { type Component, ErrorBoundary } from "solid-js";
import { Route, Router, A } from "@solidjs/router";
import { authStore } from "@/lib/auth";
import { AuthGuard } from "@/components/AuthGuard";
import { AppShell } from "@/components/AppShell";
import LoginPage from "@/pages/Login";
import Dashboard from "@/pages/Dashboard";
import DocumentsList from "@/pages/DocumentsList";
import DocumentEditor from "@/pages/DocumentEditor";
import Spaces from "@/pages/Spaces";
import Tags from "@/pages/Tags";
import Search from "@/pages/Search";
import Teams from "@/pages/Teams";
import Templates from "@/pages/Templates";
import Billing from "@/pages/Billing";
import Plugins from "@/pages/Plugins";
import Catalog from "@/pages/Catalog";
import Audit from "@/pages/Audit";
import Ssg from "@/pages/Ssg";
import Settings from "@/pages/Settings";
import Profile from "@/pages/Profile";
import Servers from "@/pages/Servers";
import Local from "@/pages/Local";
import Import from "@/pages/Import";

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
  <ErrorBoundary fallback={(err) => (
    <div class="p-8">
      <h1 class="text-xl font-bold text-red-600">Render error</h1>
      <pre class="mt-2 overflow-auto text-xs">{String(err?.message ?? err)}</pre>
    </div>
  )}>
  <Router>
    <Route path="/" component={Home} />
    <Route path="/login" component={LoginPage} />
    {/* Phase B migration stubs — each replaced by its real page */}
    <Route path="/dashboard" component={shell(Dashboard)} />
    <Route path="/documents" component={shell(DocumentsList)} />
    <Route path="/documents/:id" component={migrating("/documents/:id")} />
    <Route path="/documents/:id/edit" component={shell(DocumentEditor)} />
    <Route path="/spaces" component={shell(Spaces)} />
    <Route path="/graph" component={migrating("/graph")} />
    <Route path="/tags" component={shell(Tags)} />
    <Route path="/teams" component={shell(Teams)} />
    <Route path="/teams/:team_id" component={migrating("/teams/:team_id")} />
    <Route path="/search" component={shell(Search)} />
    <Route path="/import" component={shell(Import)} />
    <Route path="/catalog" component={shell(Catalog)} />
    <Route path="/templates" component={shell(Templates)} />
    <Route path="/plugins" component={shell(Plugins)} />
    <Route path="/ssg" component={shell(Ssg)} />
    <Route path="/billing" component={shell(Billing)} />
    <Route path="/audit" component={shell(Audit)} />
    <Route path="/admin/roles" component={migrating("/admin/roles")} />
    <Route path="/settings" component={shell(Settings)} />
    <Route path="/settings/branding" component={migrating("/settings/branding")} />
    <Route path="/settings/webhooks" component={migrating("/settings/webhooks")} />
    <Route path="/settings/notifications" component={migrating("/settings/notifications")} />
    <Route path="/profile" component={shell(Profile)} />
    <Route path="/local" component={shell(Local)} />
    <Route path="/servers" component={shell(Servers)} />
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
  </ErrorBoundary>
);

export default App;
