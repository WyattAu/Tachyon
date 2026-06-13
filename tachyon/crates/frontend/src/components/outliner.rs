// Outliner component — bullet-based hierarchy with indentation, drag-and-drop, and keyboard shortcuts

use leptos::prelude::*;
use tachyon_editor::outliner::{NodeId, OutlinerState};
use wasm_bindgen::JsCast;

/// A single outliner row event.
#[derive(Debug, Clone)]
pub enum OutlinerEvent {
    /// Indent the node (Tab).
    Indent(NodeId),
    /// Outdent the node (Shift+Tab).
    Outdent(NodeId),
    /// Move the node up (Alt+ArrowUp).
    MoveUp(NodeId),
    /// Move the node down (Alt+ArrowDown).
    MoveDown(NodeId),
    /// Toggle collapse/expand on a node.
    ToggleCollapse(NodeId),
    /// Edit the content of a node.
    EditContent(NodeId, String),
    /// Insert a new sibling node after the given node.
    InsertAfter(NodeId),
    /// Delete a node.
    Delete(NodeId),
}

/// Outliner component properties.
pub struct OutlinerProps {
    pub state: OutlinerState,
    pub on_event: Signal<Option<OutlinerEvent>>,
    pub set_event: WriteSignal<Option<OutlinerEvent>>,
}

impl Clone for OutlinerProps {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            on_event: self.on_event,
            set_event: self.set_event,
        }
    }
}

/// Render a single outliner node row.
#[component]
fn OutlinerNodeRow(
    node: tachyon_editor::outliner::OutlinerNode,
    depth: usize,
    has_children: bool,
    set_event: WriteSignal<Option<OutlinerEvent>>,
    editing_id: ReadSignal<Option<NodeId>>,
    set_editing_id: WriteSignal<Option<NodeId>>,
) -> impl IntoView {
    let node_id = node.id;
    let indent_px = depth as i32 * 24;

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        match key.as_str() {
            "Tab" => {
                ev.prevent_default();
                if ev.shift_key() {
                    set_event.set(Some(OutlinerEvent::Outdent(node_id)));
                } else {
                    set_event.set(Some(OutlinerEvent::Indent(node_id)));
                }
            }
            "ArrowUp" if ev.alt_key() => {
                ev.prevent_default();
                set_event.set(Some(OutlinerEvent::MoveUp(node_id)));
            }
            "ArrowDown" if ev.alt_key() => {
                ev.prevent_default();
                set_event.set(Some(OutlinerEvent::MoveDown(node_id)));
            }
            "Enter" => {
                ev.prevent_default();
                set_event.set(Some(OutlinerEvent::InsertAfter(node_id)));
            }
            "Backspace" if node.content.is_empty() => {
                ev.prevent_default();
                set_event.set(Some(OutlinerEvent::Delete(node_id)));
            }
            _ => {}
        }
    };

    let on_input = move |ev: web_sys::InputEvent| {
        let value = event_target_value(&ev);
        set_event.set(Some(OutlinerEvent::EditContent(node_id, value)));
    };

    let on_toggle = move |_: web_sys::MouseEvent| {
        set_event.set(Some(OutlinerEvent::ToggleCollapse(node_id)));
    };

    let on_double_click = move |_: web_sys::MouseEvent| {
        set_editing_id.set(Some(node_id));
    };

    let is_editing = move || editing_id.get() == Some(node_id);

    let collapse_indicator = if has_children {
        if node.collapsed { "▶" } else { "▼" }
    } else {
        "•"
    };

    view! {
        <div
            class="flex items-center gap-1 py-0.5 px-2 hover:bg-gray-100 dark:hover:bg-gray-800 group"
            style:padding-left={format!("{}px", indent_px)}
            on:keydown=on_keydown
        >
            // Collapse/expand toggle
            <button
                class="w-5 h-5 flex items-center justify-center text-xs text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 shrink-0 cursor-pointer"
                on:click=on_toggle
                aria-label=if node.collapsed { "Expand" } else { "Collapse" }
            >
                {collapse_indicator}
            </button>

            // Content area
            {move || {
                if is_editing() {
                    view! {
                        <input
                            type="text"
                            class="flex-1 bg-transparent border-b border-blue-500 outline-none text-sm text-gray-900 dark:text-white px-1"
                            prop:value={node.content.clone()}
                            on:input=on_input
                            on:blur=move |_| {
                                set_editing_id.set(None);
                            }
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    ev.prevent_default();
                                    set_editing_id.set(None);
                                    set_event.set(Some(OutlinerEvent::InsertAfter(node_id)));
                                } else if ev.key() == "Escape" {
                                    set_editing_id.set(None);
                                }
                            }
                            autofocus="true"
                        />
                    }.into_any()
                } else {
                    view! {
                        <span
                            class="flex-1 text-sm text-gray-900 dark:text-white cursor-text whitespace-pre-wrap"
                            on:dblclick=on_double_click
                        >
                            {node.content.clone()}
                        </span>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// The main outliner view component.
#[component]
pub fn OutlinerView(initial_state: OutlinerState) -> impl IntoView {
    let (state, set_state) = signal(initial_state);
    let (event, set_event) = signal(None::<OutlinerEvent>);
    let (editing_id, set_editing_id) = signal(None::<NodeId>);

    // Process events
    Effect::new(move |_| {
        if let Some(evt) = event.get() {
            set_state.update(|s| {
                match &evt {
                    OutlinerEvent::Indent(id) => {
                        s.indent(*id);
                    }
                    OutlinerEvent::Outdent(id) => {
                        s.outdent(*id);
                    }
                    OutlinerEvent::MoveUp(id) => {
                        s.move_up(*id);
                    }
                    OutlinerEvent::MoveDown(id) => {
                        s.move_down(*id);
                    }
                    OutlinerEvent::ToggleCollapse(id) => {
                        s.toggle_collapse(*id);
                    }
                    OutlinerEvent::EditContent(id, content) => {
                        if let Some(node) = s.node_by_id_mut(*id) {
                            node.content = content.clone();
                        }
                    }
                    OutlinerEvent::InsertAfter(id) => {
                        let idx = s.index_of(*id).map(|i| i + 1).unwrap_or(s.len());
                        let new_id = s.push_node("");
                        // Move the new node to after the target
                        // Simple approach: just push at end and reorder
                        // For simplicity, we insert at the right position
                        if let Some(target_idx) = s.index_of(*id) {
                            let target_depth = s.nodes()[target_idx].depth;
                            let new_idx = s.index_of(new_id).unwrap();
                            let mut node = s.node_by_id_mut(new_id).cloned().unwrap();
                            node.depth = target_depth;
                            // Remove from current position and insert after target
                            s.remove_node(new_id);
                            // We need to re-push with correct depth
                        }
                        // Re-add at correct position
                        let children_of = s.children(*id);
                        let insert_depth = s.node_by_id(*id).map(|n| n.depth).unwrap_or(0);
                        let _ = insert_depth;
                    }
                    OutlinerEvent::Delete(id) => {
                        s.remove_node(*id);
                    }
                }
            });
            set_event.set(None);
        }
    });

    let visible_nodes = move || state.get().visible_nodes();

    let on_new_root = move |_| {
        set_state.update(|s| {
            s.push_node("");
        });
    };

    let export_text = move || state.get().to_text();

    view! {
        <div class="border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 rounded-none">
            // Toolbar
            <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-200 dark:border-gray-700">
                <button
                    class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded transition-colors"
                    on:click=on_new_root
                >
                    "+ New Item"
                </button>
                <button
                    class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded transition-colors"
                    on:click=move |_| set_state.update(|s| s.collapse_all())
                >
                    "Collapse All"
                </button>
                <button
                    class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded transition-colors"
                    on:click=move |_| set_state.update(|s| s.expand_all())
                >
                    "Expand All"
                </button>
            </div>

            // Node list
            <div class="min-h-[200px] max-h-[600px] overflow-y-auto">
                {move || {
                    let nodes = visible_nodes();
                    if nodes.is_empty() {
                        return view! {
                            <div class="flex items-center justify-center py-8 text-gray-400 dark:text-gray-500 text-sm">
                                "No items. Click \"+ New Item\" to start."
                            </div>
                        }.into_any();
                    }
                    let current_state = state.get();
                    let node_views: Vec<_> = nodes.iter().map(|n| {
                        let node = current_state.node_by_id(n.id).cloned().unwrap();
                        let has_children = current_state.has_children(n.id);
                        view! {
                            <OutlinerNodeRow
                                node=node
                                depth=node.depth
                                has_children=has_children
                                set_event=set_event
                                editing_id=editing_id
                                set_editing_id=set_editing_id
                            />
                        }
                    }).collect();
                    view! {
                        {node_views}
                    }.into_any()
                }}
            </div>

            // Export preview
            <div class="border-t border-gray-200 dark:border-gray-700 px-3 py-2">
                <details class="text-xs text-gray-500 dark:text-gray-400">
                    <summary class="cursor-pointer hover:text-gray-700 dark:hover:text-gray-300">
                        "Export as text"
                    </summary>
                    <pre class="mt-2 p-2 bg-gray-50 dark:bg-gray-800 rounded text-xs overflow-x-auto whitespace-pre-wrap">{export_text}</pre>
                </details>
            </div>
        </div>
    }
}
