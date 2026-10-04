import { Show, createResource, createSignal, onCleanup } from "solid-js";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * PDF document view — Solid port (Phase C). Parity with the Leptos viewer:
 * browser-native PDF display in an iframe (#page/#zoom fragments) with page
 * and zoom controls. The PDF is fetched with auth and served to the iframe
 * via an object URL (iframes cannot send Authorization headers).
 *
 * Annotation toolbar parity is UI-scaffold only — Leptos kept annotations
 * in memory too (persistence is a tracked Phase E item, feature-complete-
 * v1.md §2).
 */

interface Attachment {
  id: string;
  filename: string;
  mime_type: string;
}

export function PdfView(props: { docId: string; attachment: Attachment; title: string }) {
  const [page, setPage] = createSignal(1);
  const [zoom, setZoom] = createSignal(1);
  const [objectUrl, setObjectUrl] = createSignal<string | null>(null);

  const load = async () => {
    const res = await fetch(
      `${resolveBaseUrl()}/documents/${props.docId}/attachments/${props.attachment.id}`,
      { headers: { Authorization: `Bearer ${getToken() ?? ""}` } }
    );
    if (!res.ok) return;
    const blob = await res.blob();
    setObjectUrl(URL.createObjectURL(blob));
  };
  void load();
  onCleanup(() => {
    const u = objectUrl();
    if (u) URL.revokeObjectURL(u);
  });

  return (
    <div>
      <div class="mb-2 flex items-center justify-end gap-2 text-sm">
        <button class="rounded-none border border-gray-300 px-2 py-1 dark:border-gray-600" onClick={() => setPage((v) => Math.max(1, v - 1))} aria-label="Previous page">
          ‹
        </button>
        <span class="font-mono text-xs">p. {page()}</span>
        <button class="rounded-none border border-gray-300 px-2 py-1 dark:border-gray-600" onClick={() => setPage((v) => v + 1)} aria-label="Next page">
          ›
        </button>
        <button class="rounded-none border border-gray-300 px-2 py-1 dark:border-gray-600" onClick={() => setZoom((z) => Math.max(0.5, +(z - 0.25).toFixed(2)))} aria-label="Zoom out">
          −
        </button>
        <span class="font-mono text-xs">{Math.round(zoom() * 100)}%</span>
        <button class="rounded-none border border-gray-300 px-2 py-1 dark:border-gray-600" onClick={() => setZoom((z) => Math.min(3, +(z + 0.25).toFixed(2)))} aria-label="Zoom in">
          +
        </button>
      </div>
      <div class="overflow-auto rounded-none border-2 border-gray-900 bg-gray-800 dark:border-gray-100" style={{ height: "78vh" }}>
        <Show when={objectUrl()} fallback={<p class="p-6 text-sm text-gray-400">Loading PDF…</p>}>
          <iframe
            src={`${objectUrl()}#page=${page()}`}
            class="h-full w-full border-0"
            style={{ "min-height": "78vh" }}
            title={props.title}
          />
        </Show>
      </div>
    </div>
  );
}

/** Detect a PDF attachment for a document (null when none). */
export async function findPdfAttachment(docId: string): Promise<Attachment | null> {
  try {
    const res = await fetch(`${resolveBaseUrl()}/documents/${docId}/attachments`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return null;
    // server returns a BARE ARRAY (untyped route); tolerate an envelope too
    const body = (await res.json()) as unknown as
      | { attachments?: Attachment[] }
      | Attachment[];
    const list = Array.isArray(body) ? body : (body.attachments ?? []);
    return list.find((a) => a.filename.toLowerCase().endsWith(".pdf")) ?? null;
  } catch {
    return null;
  }
}
