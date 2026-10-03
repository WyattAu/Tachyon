import { createSignal, Show } from "solid-js";
import { A } from "@solidjs/router";
import { api } from "@/lib/api/client";

/**
 * Forgot password — Solid port (Phase C). Request half of the reset flow:
 * POST /auth/password-reset/request {email}. Anti-enumeration: the server
 * always answers with the same generic message.
 */
export default function ForgotPassword() {
  const [email, setEmail] = createSignal("");
  const [sent, setSent] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const submit = async (e: SubmitEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await api.POST("/auth/password-reset/request", {
        body: { email: email() } as never,
      });
      setSent(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="flex min-h-screen items-center justify-center p-4">
      <div class="w-full max-w-md">
        <h1 class="font-display mb-6 text-center text-2xl font-bold">Reset your password</h1>

        <Show
          when={!sent()}
          fallback={
            <div class="rounded-none border border-green-300 bg-green-50 p-6 text-center dark:border-green-700 dark:bg-green-900/20">
              <p class="font-medium text-green-700 dark:text-green-300">
                Check your email
              </p>
              <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                If an account with that email exists, a password reset link has been sent.
                The link expires in 1 hour.
              </p>
              <A
                href="/login"
                class="mt-4 inline-block rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700"
              >
                Back to sign in
              </A>
            </div>
          }
        >
          <form
            class="rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800"
            onSubmit={submit}
          >
            <Show when={error()}>
              <p class="mb-3 text-sm text-red-600 dark:text-red-400">{error()}</p>
            </Show>
            <label class="mb-1 block text-sm font-medium" for="email">
              Email
            </label>
            <input
              id="email"
              type="email"
              required
              placeholder="you@example.com"
              class="mb-4 w-full rounded-none border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              value={email()}
              onInput={(e) => setEmail(e.currentTarget.value)}
            />
            <button
              type="submit"
              disabled={busy()}
              class="w-full rounded-none bg-blue-600 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
            >
              {busy() ? "Sending…" : "Send reset link"}
            </button>
            <p class="mt-4 text-center text-xs text-gray-400">
              <A href="/login" class="hover:underline">
                ← Back to sign in
              </A>
            </p>
          </form>
        </Show>
      </div>
    </div>
  );
}
