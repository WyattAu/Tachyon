import { createSignal, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authStore } from "@/lib/auth";

/**
 * Login page — Solid port of the Leptos login (Phase B first migrated page).
 * Server URL field matches the Leptos contract (placeholder org input);
 * guest login supported. OAuth buttons render disabled until Phase C wires
 * the real OAuth2 redirect flow (server routes exist; UI pending).
 */
export default function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  const submit = async (e: SubmitEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await authStore.login(username(), password());
      navigate("/dashboard", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "login failed");
    } finally {
      setBusy(false);
    }
  };

  const guest = async () => {
    setBusy(true);
    setError(null);
    try {
      await authStore.guestLogin();
      navigate("/dashboard", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "guest login failed");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="flex min-h-screen items-center justify-center p-4">
      <div class="w-full max-w-md">
        <h1 class="font-display mb-1 text-center text-3xl font-bold">Tachyon</h1>
        <p class="mb-6 text-center text-sm text-gray-500 dark:text-gray-400">
          Knowledge Management Platform
        </p>

        <form
          class="rounded-none border border-gray-900 bg-white p-6 shadow-spatial dark:border-gray-100 dark:bg-gray-800"
          onSubmit={submit}
        >
          <h2 class="mb-4 text-lg font-semibold">Sign in to your account</h2>

          <label class="mb-1 block text-sm font-medium" for="username">
            Username or Email
          </label>
          <input
            id="username"
            name="username"
            type="text"
            required
            placeholder="Enter your username"
            class="mb-3 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />

          <label class="mb-1 block text-sm font-medium" for="password">
            Password
          </label>
          <input
            id="password"
            name="password"
            type="password"
            required
            placeholder="Enter your password"
            class="mb-3 w-full rounded-none border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
          />

          <Show when={error()}>
            <p class="mb-3 rounded-none border border-red-300 bg-red-50 p-2 text-sm text-red-700 dark:border-red-700 dark:bg-red-900/30 dark:text-red-300">
              {error()}
            </p>
          </Show>

          <button
            type="submit"
            disabled={busy()}
            class="w-full rounded-none bg-blue-600 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          >
            {busy() ? "Signing in…" : "Sign in"}
          </button>

          <div class="my-4 flex items-center gap-2 text-xs text-gray-400">
            <span class="h-px flex-1 bg-gray-200 dark:bg-gray-600" />
            Or
            <span class="h-px flex-1 bg-gray-200 dark:bg-gray-600" />
          </div>

          <button
            type="button"
            onClick={guest}
            disabled={busy()}
            class="w-full rounded-none border border-gray-300 py-2 text-sm hover:bg-gray-100 disabled:opacity-50 dark:border-gray-600 dark:hover:bg-gray-700"
          >
            Continue as Guest
          </button>

          <p class="mt-4 text-center text-xs text-gray-400">
            OAuth (Google/GitHub) lands in Phase C ·{" "}
            <a class="text-blue-600 hover:underline dark:text-blue-400" href="/register">
              Create account
            </a>
          </p>
        </form>
      </div>
    </div>
  );
}
