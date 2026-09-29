import { Show, createResource, createSignal } from "solid-js";
import { useParams } from "@solidjs/router";
import { api } from "@/lib/api/client";

/** Document view (read mode) — Solid port (Phase B). Rendered markdown. */

interface DocumentData {
  id: string;
  title: string;
  content: string;
  status: string;
  updated_at: string | null;
  tags: string[] | null;
}

export default function DocumentView() {
  const params = useParams();
  const docId = () => params.id ?? "";
  const [editError, setEditError] = createSignal(false);

  const [doc] = createResource(docId, async (id) => {
    const { data, error, response } = await api.GET("/documents/{document_id}", {
      params: { path: { document_id: id } },
    });
    if (error || !data) throw new Error(`load failed (${response.status})`);
    return data as unknown as DocumentData;
  });

  const [html] = createResource(doc, async (d) => renderMarkdown(d?.content ?? ""));

  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <Show
        when={doc()}
        fallback={
          <Show when={!doc.loading} fallback={<div class="h-32 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
            <p class="text-sm text-gray-500">Document not found.</p>
          </Show>
        }
      >
        {(d) => (
          <>
            <div class="mb-4 flex items-center justify-between">
              <h1 class="font-display text-3xl font-bold text-gray-900 dark:text-white">
                {d().title}
              </h1>
              <Show when={!editError()} fallback={<span class="text-xs text-gray-400">edit unavailable</span>}>
                <a
                  href={`/documents/${docId()}/edit`}
                  class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
                >
                  Edit
                </a>
              </Show>
            </div>
            <div class="mb-4 flex gap-2 text-xs text-gray-400">
              <span class="rounded-full bg-gray-100 px-2 py-0.5 dark:bg-gray-800">{d().status}</span>
              <Show when={d().updated_at}>
                <span>updated {new Date(d().updated_at as string).toLocaleDateString()}</span>
              </Show>
            </div>
            {/* markdown render — engine-wasm parity path (Phase C ties to SSG renderer) */}
            <article
              class="prose max-w-none dark:prose-invert prose-headings:font-display"
              innerHTML={html() ?? "<p></p>"}
            />
          </>
        )}
      </Show>
    </div>
  );
}

function renderMarkdown(content: string): string {
  // Interim client render: escape + minimal md blocks until engine-wasm
  // render_markdown is wired (Phase C, exact parity with docs-pipeline).
  const esc = content
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
  return esc
    .replace(/^### (.+)$/gm, "<h3>$1</h3>")
    .replace(/^## (.+)$/gm, "<h2>$1</h2>")
    .replace(/^# (.+)$/gm, "<h1>$1</h1>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`(.+?)`/g, "<code>$1</code>")
    .replace(/\n\n/g, "<br/><br/>")
    .replace(/\n/g, "<br/>");
}
