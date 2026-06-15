// Canvas / Whiteboard Page
// Interactive infinite canvas with node/edge creation, drag-and-drop, and auto-layout

use crate::canvas::*;
use crate::components::{BreadcrumbItem, Breadcrumbs};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

/// Canvas page component
#[component]
pub fn CanvasPage() -> impl IntoView {
    let (canvas_state, set_canvas_state) = signal(CanvasState::default());
    let (toolbar_mode, set_toolbar_mode) = signal(ToolbarMode::Select);
    let (_loading, _set_loading) = signal(false);
    let (error, _set_error) = signal(None::<String>);

    let canvas_ref = NodeRef::new();

    // Initialize canvas renderer on mount
    #[allow(deprecated)]
    let _ = watch(
        move || canvas_ref.get(),
        move |canvas_el: &Option<web_sys::HtmlCanvasElement>, _, _| {
            if let Some(canvas) = canvas_el {
                canvas.set_width(800);
                canvas.set_height(600);
            }
        },
        false,
    );

    // Render loop
    let render_loop = StoredValue::new(canvas_ref);
    spawn_local(async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(16).await; // ~60fps
            if let Some(canvas) = render_loop.get_value().get() {
                if let Ok(renderer) = CanvasRenderer::new(&canvas) {
                    let state = canvas_state.get_untracked();
                    renderer.render(&state.nodes, &state.edges, &state.view);
                }
            }
        }
    });

    // Add text node
    let add_text_node = move |_| {
        let mut state = canvas_state.get();
        let id = uuid::Uuid::new_v4().to_string();
        let node = CanvasNode::new_text(
            &id,
            "New Text",
            200.0 + state.nodes.len() as f64 * 30.0,
            200.0,
        );
        state.add_node(node);
        set_canvas_state.set(state);
    };

    // Add shape node
    let add_shape = move |shape_type: ShapeType| {
        let mut state = canvas_state.get();
        let id = uuid::Uuid::new_v4().to_string();
        let node = CanvasNode::new_shape(
            &id,
            shape_type,
            200.0 + state.nodes.len() as f64 * 30.0,
            200.0,
        );
        state.add_node(node);
        set_canvas_state.set(state);
    };

    // Auto-layout
    let apply_layout = move |algo: LayoutAlgorithm| {
        let mut state = canvas_state.get();
        auto_layout(&mut state.nodes, &state.edges, algo);
        set_canvas_state.set(state);
    };

    // Delete selected
    let delete_selected = move |_| {
        let mut state = canvas_state.get();
        if let Some(id) = state.selected_node_id.clone() {
            state.remove_node(&id);
            state.selected_node_id = None;
        } else if let Some(id) = state.selected_edge_id.clone() {
            state.remove_edge(&id);
            state.selected_edge_id = None;
        }
        set_canvas_state.set(state);
    };

    // Canvas mouse handlers
    let on_canvas_click = {
        move |ev: web_sys::MouseEvent| {
            let canvas = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            if let Some(canvas) = canvas {
                let rect = canvas.get_bounding_client_rect();
                let sx = ev.client_x() as f64 - rect.left();
                let sy = ev.client_y() as f64 - rect.top();
                let state = canvas_state.get();

                if let Ok(renderer) = CanvasRenderer::new(&canvas) {
                    let mut new_state = state;
                    let hit = renderer.hit_test(sx, sy, &new_state.nodes, &new_state.view);
                    new_state.selected_node_id = hit;
                    new_state.selected_edge_id = None;
                    set_canvas_state.set(new_state);
                }
            }
        }
    };

    let on_canvas_mousedown = {
        move |ev: web_sys::MouseEvent| {
            if ev.button() == 1 {
                // Middle mouse button = pan
                let mut state = canvas_state.get();
                state.is_panning = true;
                set_canvas_state.set(state);
            }
        }
    };

    let on_canvas_mouseup = {
        move |_: web_sys::MouseEvent| {
            let mut state = canvas_state.get();
            state.is_panning = false;
            set_canvas_state.set(state);
        }
    };

    let on_canvas_wheel = {
        move |ev: web_sys::WheelEvent| {
            ev.prevent_default();
            let mut state = canvas_state.get();
            let delta = ev.delta_y() * -0.001;
            state.view.zoom = (state.view.zoom + delta).clamp(0.1, 5.0);
            set_canvas_state.set(state);
        }
    };

    // Selected node for properties panel
    let selected_node = move || {
        let state = canvas_state.get();
        state
            .selected_node_id
            .as_ref()
            .and_then(|id| state.find_node(id).cloned())
    };

    view! {
        <div class="max-w-full mx-auto px-4 py-8">
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Canvas".into(), href: None },
            ]}/>

            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Canvas</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Interactive whiteboard for visual thinking and knowledge mapping."
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-4 p-4 bg-red-100 dark:bg-red-900 border border-red-400 text-red-700 dark:text-red-200">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="flex gap-4">
                // Toolbar
                <div class="flex flex-col gap-2 p-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm">
                    <button
                        class=move || format!(
                            "px-3 py-2 text-sm font-medium rounded-md transition-colors {}",
                            if toolbar_mode.get() == ToolbarMode::Select { "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" } else { "text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700" }
                        )
                        on:click=move |_| set_toolbar_mode.set(ToolbarMode::Select)
                    >
                        "Select"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=add_text_node
                    >
                        "+ Text"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| add_shape(ShapeType::Rectangle)
                    >
                        "Rectangle"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| add_shape(ShapeType::Circle)
                    >
                        "Circle"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| add_shape(ShapeType::Diamond)
                    >
                        "Diamond"
                    </button>
                    <hr class="border-gray-200 dark:border-gray-700" />
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| apply_layout(LayoutAlgorithm::ForceDirected)
                    >
                        "Force Layout"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| apply_layout(LayoutAlgorithm::Hierarchical)
                    >
                        "Tree Layout"
                    </button>
                    <button
                        class="px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 rounded-md transition-colors"
                        on:click=move |_| apply_layout(LayoutAlgorithm::Radial)
                    >
                        "Radial Layout"
                    </button>
                    <hr class="border-gray-200 dark:border-gray-700" />
                    <button
                        class="px-3 py-2 text-sm font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 rounded-md transition-colors"
                        on:click=delete_selected
                    >
                        "Delete"
                    </button>
                </div>

                // Canvas area
                <div class="flex-1 relative">
                    <canvas
                        node_ref=canvas_ref
                        class="w-full h-[600px] bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg cursor-crosshair"
                        on:click=on_canvas_click
                        on:mousedown=on_canvas_mousedown
                        on:mouseup=on_canvas_mouseup
                        on:mouseleave=on_canvas_mouseup
                        on:wheel=on_canvas_wheel
                    />

                    // Status bar
                    <div class="mt-2 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
                        <span>{move || format!("Nodes: {}", canvas_state.get().nodes.len())}</span>
                        <span>{move || format!("Edges: {}", canvas_state.get().edges.len())}</span>
                        <span>{move || format!("Zoom: {:.0}%", canvas_state.get().view.zoom * 100.0)}</span>
                    </div>
                </div>

                // Properties panel
                <Show when=move || selected_node().is_some()>
                    <div class="w-64 p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm">
                        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">"Properties"</h3>
                        {move || {
                            selected_node().map(|node| {
                                match &node.data {
                                    CanvasNodeData::Text(d) => view! {
                                        <div class="space-y-2 text-xs">
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"Content"</label>
                                                <div class="text-gray-900 dark:text-white">{d.content.clone()}</div>
                                            </div>
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"Font Size"</label>
                                                <div class="text-gray-900 dark:text-white">{d.font_size}</div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    CanvasNodeData::Image(d) => view! {
                                        <div class="space-y-2 text-xs">
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"Source"</label>
                                                <div class="text-gray-900 dark:text-white break-all">{d.src.clone()}</div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    CanvasNodeData::Link(d) => view! {
                                        <div class="space-y-2 text-xs">
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"URL"</label>
                                                <div class="text-gray-900 dark:text-white break-all">{d.url.clone()}</div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    CanvasNodeData::Document(d) => view! {
                                        <div class="space-y-2 text-xs">
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"Title"</label>
                                                <div class="text-gray-900 dark:text-white">{d.title.clone()}</div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    CanvasNodeData::Shape(d) => view! {
                                        <div class="space-y-2 text-xs">
                                            <div>
                                                <label class="block text-gray-500 dark:text-gray-400 mb-1">"Shape"</label>
                                                <div class="text-gray-900 dark:text-white">{format!("{:?}", d.shape_type)}</div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                }
                            })
                        }}
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ToolbarMode {
    Select,
}
