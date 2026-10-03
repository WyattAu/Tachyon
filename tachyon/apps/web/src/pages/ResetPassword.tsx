import { createSignal, Show } from "solid-js";
import { A, useNavigate, useSearchParams } from "@solidjs/router";
import { api } from "@/lib/api/client";

/**
 * Reset password — Solid port (Phase C). Confirm half of the reset flow:
 * POST /auth/password-reset/confirm {token, new_password}. The token
 * arrives by email as /reset-password?token=...
 */
export default function ResetPassword() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const token = () => (searchParams.token as string) ?? "";

  const [password, setPassword] = createSignal("");
  const [confirm, setConfirm] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);
  const [done, setDone] = createSignal(false);
  const [busy, setBusy] = createSignal(false);

  const submit = async (e: SubmitEvent) => {
    e.preventDefault();
    if (password() !== confirm()) {
      setError("Passwords do not match");
      return;
    }
    if (password().length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const { error: e, response } = await api.POST("/auth/password-reset/confirm", {
        body: { token: token(), new_password: password() } as never,
      });
      if (e) throw new Error(`reset failed (${response.status})`);
      setDone(true);
      setTimeout(() => navigate("/login", { replace: true }), 2500);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="flex min-h-screen items-center justify-center p-4">
      <div class="w-full max-w-md">
        <h1 class="font-display mb-6 text-center text-2xl font-bold">Choose a new password</h1>

        <Show when={token()} fallback={
          <div class="rounded-none border border-red-300 bg-red-50 p-6 text-center dark:border-red-700 dark:bg-red-900/20">
            <p class="text-sm text-red-700 dark:text-red-300">
              Missing reset token. Use the link from your email.
            </p>
            <A href="/forgot-password" class="mt-3 inline-block text-sm underline">
              Request a new link
            </A>
          </div>
        }>
          <Show
            when={!done()}
            fallback={
              <div class="rounded-none border border-green-300 bg-green-50 p-6 text-center dark:border-green-700 dark:bg-green-900/20">
                <p class="font-medium text-green-700 dark:text-green-300">
                  Password updated
                </p>
                <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                  Redirecting to sign in…
                </p>
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
              <label class="mb-1 block text-sm font-medium" for="new-password">
                New password
              </label>
              <input
                id="new-password"
                type="password"
                required
                minlength={8}
                class="mb-3 w-full rounded-none border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
              />
              <label class="mb-1 block text-sm font-medium" for="confirm-password">
                Confirm password
              </label>
              <input
                id="confirm-password"
                type="password"
                required
                class="mb-4 w-full rounded-none border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                value={confirm()}
                onInput={(e) => setConfirm(e.currentTarget.value)}
              />
              <button
                type="submit"
                disabled={busy()}
                class="w-full rounded-none bg-blue-600 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              >
                {busy() ? "Updating…" : "Update password"}
              </button>
            </form>
          </Show>
        </Show>
      </div>
    </div>
  );
}
