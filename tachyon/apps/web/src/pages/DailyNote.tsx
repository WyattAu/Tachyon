import { Show, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { api } from "@/lib/api/client";

/**
 * Daily note — Solid port (Phase B). Finds (or creates) the document for a
 * date, then hands off to the WASM editor. Journal = continuous log;
 * Daily = one note per day (both resolve to dated documents).
 */

export default function DailyNote(props: { kind: "daily" | "journal" }) {
  const navigate = useNavigate();
  const [state, setState] = createSignal<"resolving" | "error">("resolving");
  const [error, setError] = createSignal("");

  const date = new Date().toISOString().slice(0, 10);
  const title =
    props.kind === "daily" ? `Daily — ${date}` : `Journal — ${date}`;

  const resolve = async (): Promise<void> => {
    setState("resolving");
    try {
      // find existing
      const { data } = await api.GET("/documents", {
        params: { query: { search: title, page: 1, page_size: 5 } },
      });
      const results =
        ((data as unknown as { results?: Array<{ id: string; title: string }> })
          ?.results ?? []);
      const existing = results.find((d) => d.title === title);
      if (existing) {
        navigate(`/documents/${existing.id}/edit`, { replace: true });
        return;
      }
      // create
      const created = (await api.POST("/documents", {
        body: {
          title,
          content: `# ${title}\n\n`,
        } as never,
      })).data as unknown as { id: string };
      navigate(`/documents/${created.id}/edit`, { replace: true });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setState("error");
    }
  };

  void resolve();

  return (
    <div class="flex min-h-screen items-center justify-center">
      <Show
        when={state() !== "error"}
        fallback={
          <div class="text-center">
            <p class="text-sm text-red-600">{error()}</p>
            <button class="mt-2 text-sm underline" onClick={() => void resolve()}>
              Retry
            </button>
          </div>
        }
      >
        <div class="h-6 w-6 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />
      </Show>
    </div>
  );
}
