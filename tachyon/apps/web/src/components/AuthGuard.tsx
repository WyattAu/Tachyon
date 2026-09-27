import { type ParentProps, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authStore } from "@/lib/auth";

/**
 * Route guard: defers rendering until auth state resolves, then either
 * renders the children or bounces to /login (preserving the return route,
 * mirroring the Leptos app's ?return= contract).
 */
export function AuthGuard(props: ParentProps) {
  const navigate = useNavigate();
  return (
    <Show
      when={authStore.state() !== "unknown"}
      fallback={
        <div class="flex h-screen items-center justify-center">
          <div class="h-6 w-6 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />
        </div>
      }
    >
      <Show
        when={authStore.isAuthenticated()}
        keyed={false}
      >
        {props.children}
      </Show>
      <Show when={!authStore.isAuthenticated() && authStore.state() === "anonymous"}>
        {(() => {
          navigate("/login", { replace: true });
          return null;
        })()}
      </Show>
    </Show>
  );
}
