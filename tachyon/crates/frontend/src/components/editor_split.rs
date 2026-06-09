#![allow(dead_code)]
use leptos::prelude::*;
use tachyon_editor::Editor;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum SplitMode {
    Edit,
    Preview,
    Split,
}

#[component]
pub fn EditorSplit(
    editor: RwSignal<Editor>,
    #[prop(default = SplitMode::Edit)] mode: SplitMode,
    #[prop(default = String::new())] document_id: String,
    #[prop(default = "Start writing...".to_string())] placeholder: String,
    #[prop(default = Callback::new(|_: String| {}))] on_change: Callback<String>,
) -> impl IntoView {
    let (current_mode, set_current_mode) = signal(mode);
    let (split_ratio, set_split_ratio) = signal(0.5f64); // 0.0 = all editor, 1.0 = all preview
    let is_dragging = RwSignal::new(false);
    let (_editor_scroll_pct, set_editor_scroll_pct) = signal(0.0f64);

    let mode_class = move || match current_mode.get() {
        SplitMode::Edit => "split-edit",
        SplitMode::Preview => "split-preview",
        SplitMode::Split => "split-both",
    };

    let set_mode = move |new_mode: SplitMode| {
        set_current_mode.set(new_mode);
    };

    // Drag handler for the divider
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let set_ratio = set_split_ratio;
    let dragging = is_dragging;
    let start_drag = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        dragging.set(true);
        let container = container_ref;
        let set_r = set_ratio;
        let drag_sig = dragging;

        // Mouse move handler
        let move_closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
            move |ev: web_sys::Event| {
                if !drag_sig.get() {
                    return;
                }
                let mouse_event = ev.dyn_into::<web_sys::MouseEvent>().unwrap();
                if let Some(container_el) = container.get() {
                    let rect = container_el.get_bounding_client_rect();
                    let x = mouse_event.client_x() as f64 - rect.left();
                    let width = rect.width();
                    if width > 0.0 {
                        let ratio = (x / width).clamp(0.15, 0.85);
                        set_r.set(ratio);
                    }
                }
            },
        );

        // Mouse up handler
        let up_dragging = drag_sig;
        let up_closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
            move |_ev: web_sys::Event| {
                up_dragging.set(false);
            },
        );

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "mousemove",
                move_closure.as_ref().unchecked_ref(),
            );
            let _ = window
                .add_event_listener_with_callback("mouseup", up_closure.as_ref().unchecked_ref());
            move_closure.forget();
            up_closure.forget();
        }
    };

    // Synced scrolling: track editor scroll percentage
    let _editor_scroll_cb = set_editor_scroll_pct;

    view! {
        <div class="editor-split-container" node_ref=container_ref>
            // Mode toggle buttons
            <div class="editor-split-controls">
                <button
                    class="editor-split-btn"
                    class:active={move || current_mode.get() == SplitMode::Edit}
                    on:click={move |_| set_mode(SplitMode::Edit)}
                    title="Editor only"
                >
                    {"Edit"}
                </button>
                <button
                    class="editor-split-btn"
                    class:active={move || current_mode.get() == SplitMode::Split}
                    on:click={move |_| set_mode(SplitMode::Split)}
                    title="Split view"
                >
                    {"Split"}
                </button>
                <button
                    class="editor-split-btn"
                    class:active={move || current_mode.get() == SplitMode::Preview}
                    on:click={move |_| set_mode(SplitMode::Preview)}
                    title="Preview only"
                >
                    {"Preview"}
                </button>
            </div>

            // Content area
            <div class={move || format!("editor-split-content {}", mode_class())}>
                {move || {
                    match current_mode.get() {
                        SplitMode::Edit => {
                            view! {
                                <div class="editor-pane editor-pane-full">
                                    <crate::components::NativeEditor
                                        content={String::new()}
                                        document_id={document_id.clone()}
                                        placeholder={placeholder.clone()}
                                        on_change={on_change}
                                    />
                                </div>
                            }.into_any()
                        }
                        SplitMode::Preview => {
                            view! {
                                <div class="editor-pane editor-pane-full">
                                    <crate::components::EditorPreview
                                        editor={editor}
                                        document_id={document_id.clone()}
                                    />
                                </div>
                            }.into_any()
                        }
                        SplitMode::Split => {
                            let editor_width = move || format!("{}%", split_ratio.get() * 100.0);
                            let preview_width = move || format!("{}%", (1.0 - split_ratio.get()) * 100.0);

                            view! {
                                <>
                                    <div class="editor-pane editor-pane-left"
                                         style:width={editor_width}>
                                        <crate::components::NativeEditor
                                            content={String::new()}
                                            document_id={document_id.clone()}
                                            placeholder={placeholder.clone()}
                                            on_change={on_change}
                                        />
                                    </div>
                                    <div
                                        class="editor-split-divider"
                                        class:dragging={move || is_dragging.get()}
                                        on:mousedown={start_drag}
                                    ></div>
                                    <div class="editor-pane editor-pane-right"
                                         style:width={preview_width}>
                                        <crate::components::EditorPreview
                                            editor={editor}
                                            document_id={document_id.clone()}
                                        />
                                    </div>
                                </>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </div>
    }
}
