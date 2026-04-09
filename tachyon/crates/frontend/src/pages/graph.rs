// Knowledge Graph Page
// Visual exploration of the knowledge graph with stats, node/edge browsing, and force-directed visualization

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::api::ApiClient;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    #[serde(default)]
    pub node_count: i64,
    #[serde(default)]
    pub edge_count: i64,
    #[serde(default)]
    pub nodes_by_type: serde_json::Value,
    #[serde(default)]
    pub edges_by_type: serde_json::Value,
    #[serde(default)]
    pub avg_degree: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub node_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub visibility: String,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            node_type: String::new(),
            name: String::new(),
            slug: None,
            description: None,
            visibility: String::new(),
            weight: 0.0,
            is_active: false,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GraphEdge {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub source_id: String,
    #[serde(default)]
    pub target_id: String,
    #[serde(default)]
    pub edge_type: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphView {
    Visual,
    Stats,
    Nodes,
}

// ============================================================================
// Force-Directed Layout
// ============================================================================

#[derive(Debug, Clone)]
struct SimNode {
    index: usize,
    id: String,
    label: String,
    node_type: String,
    description: Option<String>,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fixed: bool,
}

#[derive(Debug, Clone)]
struct SimEdge {
    source: usize,
    target: usize,
    #[allow(dead_code)]
    weight: f64,
}

fn node_color(node_type: &str) -> &'static str {
    match node_type {
        "document" => "#3B82F6",
        "component" => "#10B981",
        "project" => "#8B5CF6",
        "person" => "#F59E0B",
        _ => "#6B7280",
    }
}

fn run_simulation(nodes: &mut [SimNode], edges: &[SimEdge], iterations: usize) {
    if nodes.is_empty() {
        return;
    }

    let repulsion_strength = 5000.0;
    let attraction_strength = 0.01;
    let center_gravity = 0.005;
    let damping = 0.85;
    let max_force = 100.0;
    let n = nodes.len();

    let center_x = 500.0;
    let center_y = 350.0;

    for _ in 0..iterations {
        let mut fx = vec![0.0f64; n];
        let mut fy = vec![0.0f64; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = nodes[i].x - nodes[j].x;
                let dy = nodes[i].y - nodes[j].y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(1.0);
                let force = (repulsion_strength / dist_sq).min(max_force);
                let ux = dx / dist;
                let uy = dy / dist;
                fx[i] += force * ux;
                fy[i] += force * uy;
                fx[j] -= force * ux;
                fy[j] -= force * uy;
            }
        }

        for edge in edges {
            if edge.source >= n || edge.target >= n {
                continue;
            }
            let dx = nodes[edge.target].x - nodes[edge.source].x;
            let dy = nodes[edge.target].y - nodes[edge.source].y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let force = attraction_strength * dist;
            let ux = dx / dist;
            let uy = dy / dist;
            fx[edge.source] += force * ux;
            fy[edge.source] += force * uy;
            fx[edge.target] -= force * ux;
            fy[edge.target] -= force * uy;
        }

        for i in 0..n {
            fx[i] += (center_x - nodes[i].x) * center_gravity;
            fy[i] += (center_y - nodes[i].y) * center_gravity;
        }

        for i in 0..n {
            if nodes[i].fixed {
                continue;
            }
            nodes[i].vx = (nodes[i].vx + fx[i]) * damping;
            nodes[i].vy = (nodes[i].vy + fy[i]) * damping;
            nodes[i].x += nodes[i].vx;
            nodes[i].y += nodes[i].vy;
        }
    }
}

fn compute_view_box(nodes: &[SimNode]) -> (f64, f64, f64, f64) {
    if nodes.is_empty() {
        return (0.0, 0.0, 1000.0, 700.0);
    }

    let padding = 100.0;
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for node in nodes {
        min_x = min_x.min(node.x);
        max_x = max_x.max(node.x);
        min_y = min_y.min(node.y);
        max_y = max_y.max(node.y);
    }

    min_x -= padding;
    max_x += padding;
    min_y -= padding;
    max_y += padding;

    let width = (max_x - min_x).max(200.0);
    let height = (max_y - min_y).max(200.0);

    (min_x, min_y, width, height)
}

fn svg_event_to_coords(ev: &web_sys::MouseEvent) -> Option<(f64, f64)> {
    let target = ev.target()?;
    let el: web_sys::Element = target.dyn_into().ok()?;
    let svg_el: web_sys::Element = el.closest("svg").ok().flatten()?;

    let vb_str = svg_el.get_attribute("data-vb")?;
    let parts: Vec<&str> = vb_str.split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    let vb_x: f64 = parts[0].parse().ok()?;
    let vb_y: f64 = parts[1].parse().ok()?;
    let vb_w: f64 = parts[2].parse().ok()?;
    let vb_h: f64 = parts[3].parse().ok()?;

    let svg_js: &wasm_bindgen::JsValue = &svg_el.into();
    let get_bcr = js_sys::Reflect::get(svg_js, &js_sys::JsString::from("getBoundingClientRect"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let rect = get_bcr.call0(svg_js).ok()?;
    let left = js_sys::Reflect::get(&rect, &js_sys::JsString::from("left")).ok()?.as_f64()?;
    let top = js_sys::Reflect::get(&rect, &js_sys::JsString::from("top")).ok()?.as_f64()?;
    let width = js_sys::Reflect::get(&rect, &js_sys::JsString::from("width")).ok()?.as_f64()?;
    let height = js_sys::Reflect::get(&rect, &js_sys::JsString::from("height")).ok()?.as_f64()?;

    let scale_x = vb_w / width;
    let scale_y = vb_h / height;
    let mx = (ev.client_x() as f64 - left) * scale_x + vb_x;
    let my = (ev.client_y() as f64 - top) * scale_y + vb_y;

    Some((mx, my))
}

// ============================================================================
// Node Card Component
// ============================================================================

#[component]
fn NodeCard(node: GraphNode) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-5 hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between mb-2">
                <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">
                    {node.name.clone()}
                </h3>
                <span class="text-xs px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 flex-shrink-0 ml-2">
                    {node.node_type.clone()}
                </span>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-400 truncate">
                {node.description.clone().unwrap_or_else(|| "-".to_string())}
            </p>
            <div class="mt-3 flex items-center justify-between text-xs text-gray-400">
                <span>{node.visibility.clone()}</span>
                <span>{node.created_at.clone()}</span>
            </div>
        </div>
    }
}

// ============================================================================
// Page Component
// ============================================================================

#[component]
pub fn GraphPage() -> impl IntoView {
    let (view, set_view) = signal(GraphView::Visual);
    let (stats, set_stats) = signal(None::<GraphStats>);
    let (nodes, set_nodes) = signal(Vec::<GraphNode>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let (sim_nodes, set_sim_nodes) = signal(Vec::<SimNode>::new());
    let (sim_edges, set_sim_edges) = signal(Vec::<SimEdge>::new());
    let (selected_node, set_selected_node) = signal(None::<usize>);
    let (hovered_node, set_hovered_node) = signal(None::<usize>);
    let (dragging, set_dragging) = signal(None::<(usize, f64, f64)>);
    let (view_box_str, set_view_box_str) = signal("0 0 1000 700".to_string());

    let load_stats = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let client = ApiClient::default();
            match client.get_graph_stats().await {
                Ok(data) => {
                    let s: GraphStats = serde_json::from_value(data).unwrap_or(GraphStats {
                        node_count: 0, edge_count: 0,
                        nodes_by_type: serde_json::json!({}),
                        edges_by_type: serde_json::json!({}),
                        avg_degree: 0.0,
                    });
                    set_stats.set(Some(s));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load graph stats: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    let load_nodes = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let client = ApiClient::default();
            match client.list_graph_nodes(None, None, Some(1), Some(50)).await {
                Ok(data) => {
                    if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
                        let parsed: Vec<GraphNode> = items
                            .iter()
                            .filter_map(|item| serde_json::from_value(item.clone()).ok())
                            .collect();
                        set_nodes.set(parsed);
                    }
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load nodes: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    let load_visual = move || {
        set_loading.set(true);
        set_error.set(None);
        set_selected_node.set(None);
        set_hovered_node.set(None);
        set_dragging.set(None);
        spawn_local(async move {
            let client = ApiClient::default();

            let all_nodes = match client.list_graph_nodes(None, None, Some(1), Some(200)).await {
                Ok(data) => {
                    if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
                        items
                            .iter()
                            .filter_map(|item| serde_json::from_value::<GraphNode>(item.clone()).ok())
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load graph: {}", e)));
                    set_loading.set(false);
                    return;
                }
            };

            let id_to_idx: HashMap<String, usize> = all_nodes
                .iter()
                .enumerate()
                .map(|(i, n)| (n.id.clone(), i))
                .collect();

            let mut edge_set: HashSet<(String, String)> = HashSet::new();
            let mut collected_edges = Vec::new();

            for node in &all_nodes {
                if let Ok(data) = client.get_node_edges(&node.id).await {
                    if let Some(items) = data.as_array() {
                        for item in items {
                            if let Ok(edge) = serde_json::from_value::<GraphEdge>(item.clone()) {
                                let key = if edge.source_id < edge.target_id {
                                    (edge.source_id.clone(), edge.target_id.clone())
                                } else {
                                    (edge.target_id.clone(), edge.source_id.clone())
                                };
                                if edge_set.insert(key) {
                                    if let (Some(&src_idx), Some(&tgt_idx)) =
                                        (id_to_idx.get(&edge.source_id), id_to_idx.get(&edge.target_id))
                                    {
                                        if src_idx != tgt_idx {
                                            collected_edges.push(SimEdge {
                                                source: src_idx,
                                                target: tgt_idx,
                                                weight: edge.weight.max(1.0),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut sim: Vec<SimNode> = all_nodes
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let angle = (i as f64) * 2.0 * std::f64::consts::PI / (all_nodes.len() as f64).max(1.0);
                    let radius = 200.0 + (i as f64 % 10.0) * 20.0;
                    SimNode {
                        index: i,
                        id: n.id.clone(),
                        label: n.name.clone(),
                        node_type: n.node_type.clone(),
                        description: n.description.clone(),
                        x: 500.0 + radius * angle.cos(),
                        y: 350.0 + radius * angle.sin(),
                        vx: 0.0,
                        vy: 0.0,
                        fixed: false,
                    }
                })
                .collect();

            run_simulation(&mut sim, &collected_edges, 300);

            let (vb_x, vb_y, vb_w, vb_h) = compute_view_box(&sim);
            set_view_box_str.set(format!("{} {} {} {}", vb_x, vb_y, vb_w, vb_h));

            set_sim_nodes.set(sim);
            set_sim_edges.set(collected_edges);
            set_loading.set(false);
        });
    };

    let switch_to_visual = move |_| {
        set_view.set(GraphView::Visual);
        load_visual();
    };
    let switch_to_stats = move |_| {
        set_view.set(GraphView::Stats);
        load_stats();
    };
    let switch_to_nodes = move |_| {
        set_view.set(GraphView::Nodes);
        load_nodes();
    };

    let on_svg_mouse_move = move |ev: web_sys::MouseEvent| {
        let drag_info = dragging.get();
        let (idx, offset_x, offset_y) = match drag_info {
            Some(d) => d,
            None => return,
        };

        let coords = match svg_event_to_coords(&ev) {
            Some(c) => c,
            None => return,
        };
        let (mx, my) = coords;

        let mut updated = sim_nodes.get();
        if idx < updated.len() {
            updated[idx].x = mx - offset_x;
            updated[idx].y = my - offset_y;
            updated[idx].fixed = true;
            let (vb_x, vb_y, vb_w, vb_h) = compute_view_box(&updated);
            set_view_box_str.set(format!("{} {} {} {}", vb_x, vb_y, vb_w, vb_h));
            set_sim_nodes.set(updated);
        }
    };

    let on_svg_mouse_up = move |_: web_sys::MouseEvent| {
        if let Some((idx, _, _)) = dragging.get() {
            let mut updated = sim_nodes.get();
            if idx < updated.len() {
                updated[idx].fixed = false;
                set_sim_nodes.set(updated);
            }
        }
        set_dragging.set(None);
    };

    let close_popup = move |_: web_sys::MouseEvent| {
        set_selected_node.set(None);
    };

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Knowledge Graph"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Explore nodes and relationships in your knowledge base."
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-6 p-4 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded-lg">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="mb-6 border-b border-gray-200 dark:border-gray-700">
                <nav class="flex space-x-8">
                    <button
                        class="pb-3 border-b-2 font-medium text-sm transition-colors"
                        class=("border-blue-500 text-blue-600 dark:text-blue-400", move || view.get() == GraphView::Visual)
                        class=("border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400", move || view.get() != GraphView::Visual)
                        on:click=switch_to_visual
                    >
                        "Visual"
                    </button>
                    <button
                        class="pb-3 border-b-2 font-medium text-sm transition-colors"
                        class=("border-blue-500 text-blue-600 dark:text-blue-400", move || view.get() == GraphView::Stats)
                        class=("border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400", move || view.get() != GraphView::Stats)
                        on:click=switch_to_stats
                    >
                        "Overview"
                    </button>
                    <button
                        class="pb-3 border-b-2 font-medium text-sm transition-colors"
                        class=("border-blue-500 text-blue-600 dark:text-blue-400", move || view.get() == GraphView::Nodes)
                        class=("border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400", move || view.get() != GraphView::Nodes)
                        on:click=switch_to_nodes
                    >
                        "Nodes"
                    </button>
                </nav>
            </div>

            // Visual view
            <Show when=move || view.get() == GraphView::Visual>
                <div class="relative">
                    <Show when=move || sim_nodes.get().is_empty() && !loading.get()>
                        <div class="text-center py-16 text-gray-400 dark:text-gray-500">
                            <p class="text-lg">"No graph data available"</p>
                            <p class="text-sm mt-1">"Add some nodes and edges to see the visualization."</p>
                        </div>
                    </Show>

                    <Show when=move || !sim_nodes.get().is_empty()>
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                            <svg
                                class="w-full"
                                style="min-height: 500px;"
                                style=("cursor", move || if dragging.get().is_some() { "grabbing" } else { "grab" })
                                viewBox=view_box_str
                                attr:data-vb=view_box_str
                                preserveAspectRatio="xMidYMid meet"
                                on:mousemove=on_svg_mouse_move
                                on:mouseup=on_svg_mouse_up
                                on:mouseleave=on_svg_mouse_up
                            >
                                // Edges
                                <For
                                    each=move || { sim_edges.get() }
                                    key=|e| format!("e-{}-{}", e.source, e.target)
                                    let:edge
                                >
                                    {
                                        let sim_nodes = sim_nodes;
                                        let hovered_node = hovered_node;
                                        let sim_edges_inner = sim_edges;
                                        view! {
                                            <line
                                                x1=move || sim_nodes.with(|ns| ns.get(edge.source).map(|n| n.x).unwrap_or(0.0))
                                                y1=move || sim_nodes.with(|ns| ns.get(edge.source).map(|n| n.y).unwrap_or(0.0))
                                                x2=move || sim_nodes.with(|ns| ns.get(edge.target).map(|n| n.x).unwrap_or(0.0))
                                                y2=move || sim_nodes.with(|ns| ns.get(edge.target).map(|n| n.y).unwrap_or(0.0))
                                                stroke=move || {
                                                    let h = hovered_node.get();
                                                    if let Some(h_idx) = h {
                                                        let is_connected = sim_edges_inner.with(|edges| {
                                                            edges.iter().any(|e| {
                                                                (e.source == h_idx && e.target == edge.target)
                                                                    || (e.target == h_idx && e.source == edge.source)
                                                            })
                                                        }) || edge.source == h_idx || edge.target == h_idx;
                                                        if is_connected { "#94A3B8" } else { "#E2E8F0" }
                                                    } else {
                                                        "#CBD5E1"
                                                    }
                                                }
                                                stroke-width=move || {
                                                    let h = hovered_node.get();
                                                    if let Some(h_idx) = h {
                                                        if edge.source == h_idx || edge.target == h_idx { 2.0 } else { 0.5 }
                                                    } else {
                                                        1.0
                                                    }
                                                }
                                                fill="none"
                                            />
                                        }
                                    }
                                </For>

                                // Nodes
                                <For
                                    each=move || { sim_nodes.get() }
                                    key=|n| n.id.clone()
                                    let:node
                                >
                                    {
                                        let hovered_node = hovered_node;
                                        let selected_node = selected_node;
                                        let sim_edges_inner = sim_edges;
                                        let set_dragging = set_dragging;
                                        let set_selected_node = set_selected_node;
                                        let set_hovered_node = set_hovered_node;
                                        let node_color_val: &'static str = node_color(&node.node_type);
                                        let node_idx = node.index;
                                        view! {
                                            <g
                                                transform=move || format!("translate({}, {})", node.x, node.y)
                                                class="cursor-pointer"
                                                on:mousedown=move |ev: web_sys::MouseEvent| {
                                                    ev.stop_propagation();
                                                    ev.prevent_default();
                                                    let coords = svg_event_to_coords(&ev);
                                                    if let Some((mx, my)) = coords {
                                                        set_dragging.set(Some((node_idx, mx - node.x, my - node.y)));
                                                    }
                                                }
                                                on:click=move |ev: web_sys::MouseEvent| {
                                                    ev.stop_propagation();
                                                    set_selected_node.set(Some(node_idx));
                                                }
                                                on:mouseenter=move |_| set_hovered_node.set(Some(node_idx))
                                                on:mouseleave=move |_| set_hovered_node.set(None)
                                            >
                                                <Show when=move || {
                                                    let h = hovered_node.get();
                                                    match h {
                                                        None => false,
                                                        Some(h_idx) => {
                                                            if h_idx == node_idx { return true; }
                                                            sim_edges_inner.with(|edges| {
                                                                edges.iter().any(|e| {
                                                                    (e.source == h_idx && e.target == node_idx)
                                                                        || (e.target == h_idx && e.source == node_idx)
                                                                })
                                                            })
                                                        }
                                                    }
                                                }>
                                                    <circle
                                                        cx=0 cy=0 r=18
                                                        fill="none"
                                                        stroke="#3B82F6"
                                                        stroke-width=2
                                                        stroke-opacity=0.4
                                                    />
                                                </Show>

                                                <Show when=move || selected_node.get() == Some(node_idx)>
                                                    <circle
                                                        cx=0 cy=0 r=18
                                                        fill="none"
                                                        stroke=node_color_val
                                                        stroke-width=2.5
                                                        stroke-opacity=0.8
                                                    />
                                                </Show>

                                                <circle
                                                    cx=0 cy=0 r=12
                                                    fill=node_color_val
                                                    stroke="white"
                                                    stroke-width=2
                                                />

                                                <text
                                                    x=0 y=24
                                                    text-anchor="middle"
                                                    class="fill-gray-600 dark:fill-gray-300 select-none"
                                                    style="font-size: 10px; pointer-events: none;"
                                                >
                                                    {
                                                        let label = node.label.clone();
                                                        if label.len() > 20 {
                                                            format!("{}...", &label[..17])
                                                        } else {
                                                            label
                                                        }
                                                    }
                                                </text>
                                            </g>
                                        }
                                    }
                                </For>
                            </svg>

                            <div class="flex flex-wrap gap-4 px-4 py-3 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400">
                                <div class="flex items-center gap-1.5">
                                    <span class="inline-block w-3 h-3 rounded-full" style="background-color: #3B82F6;"></span>
                                    "Document"
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <span class="inline-block w-3 h-3 rounded-full" style="background-color: #10B981;"></span>
                                    "Component"
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <span class="inline-block w-3 h-3 rounded-full" style="background-color: #8B5CF6;"></span>
                                    "Project"
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <span class="inline-block w-3 h-3 rounded-full" style="background-color: #F59E0B;"></span>
                                    "Person"
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <span class="inline-block w-3 h-3 rounded-full" style="background-color: #6B7280;"></span>
                                    "Other"
                                </div>
                            </div>
                        </div>

                        // Node detail popup
                        <Show when=move || selected_node.get().is_some()>
                            {
                                let close_popup = close_popup;
                                view! {
                                    <div
                                        class="fixed inset-0 z-40"
                                        on:click=close_popup
                                    ></div>
                                    <div class="absolute top-4 right-4 z-50 w-72 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-4">
                                        <div class="flex items-start justify-between mb-3">
                                            <h3 class="text-sm font-semibold text-gray-900 dark:text-white pr-2 break-words">
                                                {move || selected_node.get().and_then(|i| sim_nodes.with(|ns| ns.get(i).map(|n| n.label.clone()))).unwrap_or_default()}
                                            </h3>
                                            <button
                                                class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 flex-shrink-0"
                                                on:click=close_popup
                                            >
                                                "x"
                                            </button>
                                        </div>
                                        <div class="space-y-2 text-xs">
                                            <div class="flex items-center gap-2">
                                                <span class="inline-block w-2.5 h-2.5 rounded-full flex-shrink-0"
                                                    style=move || format!("background-color: {};", selected_node.get().and_then(|i| sim_nodes.with(|ns| ns.get(i).map(|n| node_color(&n.node_type)))).unwrap_or("#6B7280"))
                                                ></span>
                                                <span class="text-gray-500 dark:text-gray-400">
                                                    {move || selected_node.get().and_then(|i| sim_nodes.with(|ns| ns.get(i).map(|n| n.node_type.clone()))).unwrap_or_default()}
                                                </span>
                                            </div>
                                            <p class="text-gray-600 dark:text-gray-300 break-words">
                                                {move || selected_node.get().and_then(|i| sim_nodes.with(|ns| ns.get(i).and_then(|n| n.description.clone()))).unwrap_or_else(|| "No description".to_string())}
                                            </p>
                                            <div class="pt-2 border-t border-gray-100 dark:border-gray-700 text-gray-400">
                                                {move || {
                                                    let count = selected_node.get()
                                                        .map(|i| {
                                                            sim_edges.with(|edges| {
                                                                edges.iter().filter(|e| e.source == i || e.target == i).count()
                                                            })
                                                        })
                                                        .unwrap_or(0);
                                                    format!("{} connection{}", count, if count == 1 { "" } else { "s" })
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        </Show>
                    </Show>
                </div>
            </Show>

            // Stats view
            <Show when=move || view.get() == GraphView::Stats>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="text-3xl font-bold text-blue-600 dark:text-blue-400">
                            {move || stats.get().map(|s| s.node_count.to_string()).unwrap_or_default()}
                        </div>
                        <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Total Nodes"</div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="text-3xl font-bold text-green-600 dark:text-green-400">
                            {move || stats.get().map(|s| s.edge_count.to_string()).unwrap_or_default()}
                        </div>
                        <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Total Edges"</div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="text-3xl font-bold text-purple-600 dark:text-purple-400">
                            {move || stats.get().map(|s| format!("{:.2}", s.avg_degree).to_string()).unwrap_or_default()}
                        </div>
                        <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Avg Degree"</div>
                    </div>
                </div>
            </Show>

            // Nodes view
            <Show when=move || view.get() == GraphView::Nodes>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For
                        each=move || nodes.get()
                        key=|node| node.id.clone()
                        let:node
                    >
                        <NodeCard node=node />
                    </For>
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="fixed inset-0 bg-black bg-opacity-20 flex items-center justify-center z-50">
                    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
                        <p class="mt-3 text-sm text-gray-600 dark:text-gray-400">"Loading..."</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
