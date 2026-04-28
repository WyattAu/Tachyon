#![allow(dead_code)]
use tachyon_editor::Editor;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SplitMode {
    Edit,
    Preview,
    Split,
}

#[component]
pub fn EditorSplit(
    editor: RwSignal<Editor>,
    #[prop(default = SplitMode::Edit)]
    mode: SplitMode,
    #[prop(default = String::new())]
    document_id: String,
    #[prop(default = "Start writing...".to_string())]
    placeholder: String,
    #[prop(default = Callback::new(|_: String| {}))]
    on_change: Callback<String>,
) -> impl IntoView {
    let (current_mode, set_current_mode) = signal(mode);

    let mode_class = move || match current_mode.get() {
        SplitMode::Edit => "split-edit",
        SplitMode::Preview => "split-preview",
        SplitMode::Split => "split-both",
    };

    let _toggle_mode = move |_: leptos::ev::MouseEvent| {
        let next = match current_mode.get() {
            SplitMode::Edit => SplitMode::Split,
            SplitMode::Split => SplitMode::Preview,
            SplitMode::Preview => SplitMode::Edit,
        };
        set_current_mode.set(next);
    };

    let set_mode = move |new_mode: SplitMode| {
        set_current_mode.set(new_mode);
    };

    view! {
        <div class="editor-split-container">
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
                            view! {
                                <>
                                    <div class="editor-pane editor-pane-left">
                                        <crate::components::NativeEditor
                                            content={String::new()}
                                            document_id={document_id.clone()}
                                            placeholder={placeholder.clone()}
                                             on_change={on_change}
                                         />
                                    </div>
                                    <div class="editor-split-divider"></div>
                                    <div class="editor-pane editor-pane-right">
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
