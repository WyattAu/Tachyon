import { createEffect, createSignal, onMount } from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Canvas editor — Solid port (Phase B). 2D canvas render + drag + connect
 * + persistence (the 13 canvas API methods wired natively — the Leptos
 * version kept everything in memory; this one persists every mutation).
 */

interface CNode {
  id: string;
  node_type: string;
  data: { label?: string } | null;
  position_x: number;
  position_y: number;
}
interface CEdge {
  id: string;
  source_id: string;
  target_id: string;
  edge_type: string;
}
interface CanvasMeta {
  id: string;
  title: string;
}

const auth = () => ({ Authorization: `Bearer ${getToken() ?? ""}` });
const json = () => ({ "Content-Type": "application/json", ...auth() });

const NODE_W = 140;
const NODE_H = 44;

export default function CanvasEditor() {
  const params = useParams();
  const navigate = useNavigate();
  const canvasId = () => params.id ?? "";

  let canvasEl!: HTMLCanvasElement;
  const [meta, setMeta] = createSignal<CanvasMeta | null>(null);
  const [nodes, setNodes] = createSignal<CNode[]>([]);
  const [edges, setEdges] = createSignal<CEdge[]>([]);
  const [drag, setDrag] = createSignal<{ id: string; dx: number; dy: number } | null>(null);
  const [connectFrom, setConnectFrom] = createSignal<string | null>(null);
  const [selected, setSelected] = createSignal<string | null>(null);
  const [mouse, setMouse] = createSignal({ x: 0, y: 0 });
  const [saveState, setSaveState] = createSignal("");

  // --- load: canvas meta + nodes + edges (3 of the 13 methods) ---
  onMount(async () => {
    const base = resolveBaseUrl();
    const [c, n, e] = await Promise.all([
      fetch(`${base}/canvases/${canvasId()}`, { headers: auth() }),
      fetch(`${base}/canvases/${canvasId()}/nodes`, { headers: auth() }),
      fetch(`${base}/canvases/${canvasId()}/edges`, { headers: auth() }),
    ]);
    if (c.ok) setMeta(((await c.json()) as unknown as { canvas: CanvasMeta }).canvas);
    if (n.ok) setNodes((await n.json()) as unknown as CNode[]);
    if (e.ok) setEdges((await e.json()) as unknown as CEdge[]);
  });

  // --- render loop (2D canvas) ---
  const draw = () => {
    const ctx = canvasEl?.getContext("2d");
    if (!ctx) return;
    const { width, height } = canvasEl;
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "#0f172a";
    ctx.fillRect(0, 0, width, height);

    // edges
    ctx.strokeStyle = "#64748b";
    ctx.lineWidth = 1.5;
    for (const e of edges()) {
      const a = nodes().find((n) => n.id === e.source_id);
      const b = nodes().find((n) => n.id === e.target_id);
      if (!a || !b) continue;
      ctx.beginPath();
      ctx.moveTo(a.position_x + NODE_W / 2, a.position_y + NODE_H / 2);
      ctx.lineTo(b.position_x + NODE_W / 2, b.position_y + NODE_H / 2);
      ctx.stroke();
    }
    // connect-mode rubber band
    const from = connectFrom();
    if (from) {
      const a = nodes().find((n) => n.id === from);
      if (a) {
        ctx.strokeStyle = "#f59e0b";
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        ctx.moveTo(a.position_x + NODE_W / 2, a.position_y + NODE_H / 2);
        ctx.lineTo(mouse().x, mouse().y);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
    // nodes
    for (const n of nodes()) {
      const sel = selected() === n.id;
      ctx.fillStyle = n.node_type === "note" ? "#1e293b" : "#312e81";
      ctx.strokeStyle = sel ? "#f59e0b" : "#3b82f6";
      ctx.lineWidth = sel ? 2.5 : 1.5;
      ctx.beginPath();
      ctx.roundRect(n.position_x, n.position_y, NODE_W, NODE_H, 6);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = "#e2e8f0";
      ctx.font = "12px 'JetBrains Mono', monospace";
      const label = (n.data?.label ?? n.node_type).slice(0, 18);
      ctx.fillText(label, n.position_x + 10, n.position_y + NODE_H / 2 + 4);
    }
  };

  // redraw on any state change
  [nodes, edges, drag, connectFrom, selected, mouse].forEach((sig) => {
    createEffect(() => {
      sig();
      if (canvasEl) draw();
    });
  });

  const hitTest = (x: number, y: number): CNode | null =>
    nodes().find(
      (n) => x >= n.position_x && x <= n.position_x + NODE_W && y >= n.position_y && y <= n.position_y + NODE_H
    ) ?? null;

  const toLocal = (e: MouseEvent) => {
    const r = canvasEl.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  // --- drag handling (position persisted on drop: PUT node) ---
  const onDown = (e: MouseEvent) => {
    const { x, y } = toLocal(e);
    const hit = hitTest(x, y);
    setSelected(hit?.id ?? null);
    if (connectFrom() && hit && hit.id !== connectFrom()) {
      void connect(hit.id);
      return;
    }
    if (hit) setDrag({ id: hit.id, dx: x - hit.position_x, dy: y - hit.position_y });
  };

  const onMove = (e: MouseEvent) => {
    const { x, y } = toLocal(e);
    setMouse({ x, y });
    const d = drag();
    if (!d) return;
    setNodes((ns) => ns.map((n) => (n.id === d.id ? { ...n, position_x: x - d.dx, position_y: y - d.dy } : n)));
  };

  const onUp = async () => {
    const d = drag();
    setDrag(null);
    if (!d) return;
    const n = nodes().find((n) => n.id === d.id);
    if (!n) return;
    await fetch(`${resolveBaseUrl()}/canvases/nodes/${n.id}`, {
      method: "PUT",
      headers: json(),
      body: JSON.stringify({
        node_type: n.node_type,
        data: n.data,
        position_x: n.position_x,
        position_y: n.position_y,
      }),
    });
    setSaveState(`saved ${new Date().toLocaleTimeString()}`);
  };

  // --- mutations (the remaining wired methods) ---
  const addNode = async () => {
    const res = await fetch(`${resolveBaseUrl()}/canvases/${canvasId()}/nodes`, {
      method: "POST",
      headers: json(),
      body: JSON.stringify({
        node_type: "note",
        data: { label: `Node ${nodes().length + 1}` },
        position_x: 60 + Math.random() * 500,
        position_y: 60 + Math.random() * 320,
      }),
    });
    if (res.ok) {
      const body = (await res.json()) as unknown as { node: CNode };
      setNodes((ns) => [...ns, body.node]);
      setSelected(body.node.id);
    }
  };

  const connect = async (targetId: string) => {
    const from = connectFrom();
    setConnectFrom(null);
    if (!from || from === targetId) return;
    const res = await fetch(`${resolveBaseUrl()}/canvases/${canvasId()}/edges`, {
      method: "POST",
      headers: json(),
      body: JSON.stringify({ source_id: from, target_id: targetId, edge_type: "default" }),
    });
    if (res.ok) {
      const body = (await res.json()) as unknown as { edge: CEdge };
      setEdges((es) => [...es, body.edge]);
    }
  };

  const deleteSelected = async () => {
    const id = selected();
    if (!id) return;
    // delete edges touching the node first (server FK), then the node
    for (const e of edges().filter((e) => e.source_id === id || e.target_id === id)) {
      await fetch(`${resolveBaseUrl()}/canvases/edges/${e.id}`, { method: "DELETE", headers: auth() });
    }
    setEdges((es) => es.filter((e) => e.source_id !== id && e.target_id !== id));
    await fetch(`${resolveBaseUrl()}/canvases/nodes/${id}`, { method: "DELETE", headers: auth() });
    setNodes((ns) => ns.filter((n) => n.id !== id));
    setSelected(null);
  };

  const relabel = async () => {
    const id = selected();
    if (!id) return;
    const label = prompt("Label:");
    if (label === null) return;
    const n = nodes().find((n) => n.id === id);
    if (!n) return;
    const data = { ...n.data, label };
    setNodes((ns) => ns.map((x) => (x.id === id ? { ...x, data } : x)));
    await fetch(`${resolveBaseUrl()}/canvases/nodes/${id}`, {
      method: "PUT",
      headers: json(),
      body: JSON.stringify({ node_type: n.node_type, data, position_x: n.position_x, position_y: n.position_y }),
    });
  };

  return (
    <div class="mx-auto max-w-6xl px-4 py-4">
      <div class="mb-3 flex items-center justify-between">
        <h1 class="font-display text-xl font-bold text-gray-900 dark:text-white">
          {meta()?.title ?? "Canvas"}
        </h1>
        <div class="flex items-center gap-2">
          <button class="rounded-none bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700" onClick={() => void addNode()}>
            + Node
          </button>
          <button
            class={`rounded-none px-3 py-1.5 text-sm ${connectFrom() ? "bg-amber-500 text-white" : "border border-gray-300 hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"}`}
            onClick={() => setConnectFrom(connectFrom() ? null : selected())}
          >
            {connectFrom() ? "Click target…" : "Connect"}
          </button>
          <button
            class="rounded-none border border-gray-300 px-3 py-1.5 text-sm hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"
            onClick={() => void relabel()}
            disabled={!selected()}
          >
            Rename
          </button>
          <button
            class="rounded-none border border-red-300 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 disabled:opacity-40 dark:border-red-800"
            onClick={() => void deleteSelected()}
            disabled={!selected()}
          >
            Delete
          </button>
          <button class="text-sm text-gray-500 hover:underline" onClick={() => navigate("/canvas")}>
            ← canvases
          </button>
        </div>
      </div>

      <canvas
        ref={canvasEl}
        width={1100}
        height={620}
        class="w-full cursor-move rounded-none border-2 border-gray-900 dark:border-gray-100"
        data-testid="canvas-surface"
        onMouseDown={onDown}
        onMouseMove={onMove}
        onMouseUp={() => void onUp()}
        onMouseLeave={() => void onUp()}
      />
      <p class="mt-1 text-xs text-gray-400">
        Drag nodes to move (auto-saves) · Connect: press Connect, click source, click target ·{" "}
        {saveState() || "changes persist on drop"}
      </p>
    </div>
  );
}
