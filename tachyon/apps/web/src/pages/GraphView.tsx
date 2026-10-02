import { onCleanup, onMount, Show, createSignal, createEffect } from "solid-js";
import Graph from "graphology";
import forceAtlas2 from "graphology-layout-forceatlas2";
import Sigma from "sigma";
import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Knowledge Graph — Solid port (Phase B, sigma.js per roadmap decision).
 * Data: GET /nodes + GET /edges (typed). Layout: ForceAtlas2.
 * Time-travel/query endpoints render in a later pass (spec: feature-
 * complete-v1.md §3).
 */

interface GNode {
  id: string;
  name: string;
  node_type: string | null;
  weight: number | null;
}
interface GEdge {
  id: string;
  source_id: string;
  target_id: string;
  label: string | null;
}

export default function GraphView() {
  const [graph, setGraph] = createSignal<Graph | null>(null);
  const [stats, setStats] = createSignal({ nodes: 0, edges: 0 });
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      // Runtime read routes: /graph/nodes + /graph/edges (typed /nodes exists,
      // but GET /edges is 405 — the read alias lives at /graph/edges and is
      // not yet in the OpenAPI spec: hand-typed, tracked for annotation).
      const gh = { Authorization: `Bearer ${getToken() ?? ""}` };
      const [nodesRes, edgesRes] = await Promise.all([
        fetch(`${resolveBaseUrl()}/graph/nodes?page=1&page_size=200`, { headers: gh }),
        fetch(`${resolveBaseUrl()}/graph/edges`, { headers: gh }),
      ]);
      const nodes =
        ((await nodesRes.json()) as unknown as { nodes?: GNode[] }).nodes ?? [];
      const edges =
        ((await edgesRes.json()) as unknown as { edges?: GEdge[] }).edges ?? [];
      console.log("[graph] fetched", nodes.length, "nodes,", edges.length, "edges; token:", getToken() ? "present" : "MISSING");

      const graph = new Graph({ multi: true });
      const nodeIds = new Set<string>();
      for (const n of nodes) {
        if (!nodeIds.has(n.id)) {
          graph.addNode(n.id, {
            label: n.name,
            size: 4 + Math.min(10, n.weight ?? 0),
            color: colorFor(n.node_type),
            x: Math.random(),
            y: Math.random(),
          });
          nodeIds.add(n.id);
        }
      }
      let kept = 0;
      for (const e of edges) {
        if (graph.hasNode(e.source_id) && graph.hasNode(e.target_id) && !graph.hasEdge(e.id)) {
          graph.addEdge(e.source_id, e.target_id, { label: e.label ?? "", size: 1, color: "#64748b" });
          kept++;
        }
      }

      if (graph.order === 0) {
        setStats({ nodes: 0, edges: 0 });
        setLoading(false);
        return;
      }

      forceAtlas2.assign(graph, {
        iterations: 60,
        settings: {
          gravity: 1,
          scalingRatio: 8,
          barnesHutOptimize: graph.order > 150,
        },
      });
      // Normalize into [0,1] so sigma's default camera (centered 0.5,0.5)
      // frames the whole graph without manual zoom.
      let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
      graph.forEachNode((node, attrs) => {
        minX = Math.min(minX, attrs.x); maxX = Math.max(maxX, attrs.x);
        minY = Math.min(minY, attrs.y); maxY = Math.max(maxY, attrs.y);
      });
      const spanX = maxX - minX || 1;
      const spanY = maxY - minY || 1;
      graph.forEachNode((node, attrs) => {
        graph.setNodeAttribute(node, "x", 0.15 + ((attrs.x - minX) / spanX) * 0.7);
        graph.setNodeAttribute(node, "y", 0.15 + ((attrs.y - minY) / spanY) * 0.7);
      });
      setGraph(graph);
      setStats({ nodes: graph.order, edges: kept });
      setLoading(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setLoading(false);
    }
  });

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
            Knowledge Graph
          </h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Explore nodes and relationships in your knowledge base.
          </p>
        </div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          {stats().nodes} nodes · {stats().edges} edges
        </div>
      </div>

      <Show when={!loading()} fallback={<div class="h-[500px] animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={!error()}
          fallback={<p class="text-sm text-red-600 dark:text-red-400">Graph error: {error()}</p>}
        >
          <Show
            when={stats().nodes > 0}
            fallback={
              <div class="flex h-[500px] items-center justify-center rounded-none border border-gray-900 bg-white dark:border-gray-100 dark:bg-gray-800">
                <div class="text-center">
                  <p class="text-lg font-medium text-gray-700 dark:text-gray-300">No graph data available</p>
                  <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                    Add nodes and edges (or wikilinks between documents) to see the visualization.
                  </p>
                </div>
              </div>
            }
          >
            <Show when={graph()}><GraphCanvas graph={graph()!} /></Show>
          </Show>
        </Show>
      </Show>
    </div>
  );
}

function colorFor(nodeType: string | null): string {
  switch (nodeType) {
    case "document": return "#2563eb";
    case "person": return "#16a34a";
    case "project": return "#9333ea";
    case "concept": return "#f59e0b";
    default: return "#0ea5e9";
  }
}

/** Sigma host: constructs the renderer after its own container mounts
 * (Solid refs in conditionally-rendered branches are not reactive — an
 * effect keyed on graph() ran before this div existed and skipped forever). */
function GraphCanvas(props: { graph: Graph }) {
  let container!: HTMLDivElement;

  onMount(() => {
    const renderer = new Sigma(props.graph, container, {
      allowInvalidContainer: true,
      minCameraRatio: 0.1,
      maxCameraRatio: 10,
      defaultEdgeType: "line",
      labelRenderedSizeThreshold: 0,
      defaultDrawNodeLabel: () => {},
      defaultDrawNodeHover: () => {},
    });
    (window as unknown as { __sigmaDebug?: unknown }).__sigmaDebug = renderer;
    onCleanup(() => renderer.kill());
  });

  return (
    <div
      ref={container}
      class="h-[600px] rounded-none border-2 border-gray-900 bg-white dark:border-gray-100 dark:bg-gray-950"
    />
  );
}
