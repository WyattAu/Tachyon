// Knowledge Graph Page
// Visual exploration of the knowledge graph with stats and node/edge browsing

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api::ApiClient;
use serde::{Deserialize, Serialize};

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
    Stats,
    Nodes,
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

/// Knowledge graph exploration page
#[component]
pub fn GraphPage() -> impl IntoView {
    let (view, set_view) = signal(GraphView::Stats);
    let (stats, set_stats) = signal(None::<GraphStats>);
    let (nodes, set_nodes) = signal(Vec::<GraphNode>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Load stats
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

    // Load nodes
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

    // View switching
    let switch_to_stats = move |_| { set_view.set(GraphView::Stats); load_stats(); };
    let switch_to_nodes = move |_| { set_view.set(GraphView::Nodes); load_nodes(); };

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Knowledge Graph"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Explore nodes and relationships in your knowledge base."
                </p>
            </div>

            // Error display
            <Show when=move || error.get().is_some()>
                <div class="mb-6 p-4 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded-lg">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // View tabs
            <div class="mb-6 border-b border-gray-200 dark:border-gray-700">
                <nav class="flex space-x-8">
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

            // Loading overlay
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
