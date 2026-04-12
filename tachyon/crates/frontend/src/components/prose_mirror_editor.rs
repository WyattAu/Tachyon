// ProseMirror Editor Component
//
// Wraps the JavaScript ProseMirror editor (window.TachyonEditor) in a
// Leptos component. The editor is mounted via web_sys on a <div> element.
//
// Communication flow:
//   Leptos (Rust/WASM) → web_sys → window.TachyonEditor.create(div, options)
//   Editor changes → onChange callback → JsClosure → Leptos signal
//   Leptos signal change → setMarkdown(editorView, content) → DOM update

#![allow(dead_code)]
#![allow(unused_imports)]

use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ============================================================================
// JavaScript bridge functions
// ============================================================================

#[wasm_bindgen(inline_js = r#"
export function _tachyon_editor_create(container, options) {
    if (window.TachyonEditor) {
        return window.TachyonEditor.create(container, options);
    }
    console.error('TachyonEditor not loaded. Ensure editor.js is included in index.html.');
    return null;
}

export function _tachyon_editor_get_markdown(view) {
    if (window.TachyonEditor && view) {
        return window.TachyonEditor.getMarkdown(view);
    }
    return '';
}

export function _tachyon_editor_set_markdown(view, markdown) {
    if (window.TachyonEditor && view) {
        window.TachyonEditor.setMarkdown(view, markdown);
    }
}

export function _tachyon_editor_set_editable(view, editable) {
    if (window.TachyonEditor && view) {
        window.TachyonEditor.setEditable(view, editable);
    }
}

export function _tachyon_editor_focus(view) {
    if (window.TachyonEditor && view) {
        window.TachyonEditor.focus(view);
    }
}

export function _tachyon_editor_destroy(view) {
    if (window.TachyonEditor && view) {
        window.TachyonEditor.destroy(view);
    }
}

export function _tachyon_editor_dispatch_command(view, commandName) {
    if (window.TachyonEditor) {
        return window.TachyonEditor.dispatchCommand(view, commandName);
    }
    return false;
}

// Dispatch a command on the active (most recently created) editor.
// This is used by the toolbar when no explicit EditorView reference is available.
export function _tachyon_editor_dispatch_command_active(commandName) {
    if (window.TachyonEditor) {
        return window.TachyonEditor.dispatchCommand(null, commandName);
    }
    return false;
}
"#)]
extern "C" {
    fn _tachyon_editor_create(container: &web_sys::HtmlElement, options: &JsValue) -> JsValue;
    fn _tachyon_editor_get_markdown(view: &JsValue) -> String;
    fn _tachyon_editor_set_markdown(view: &JsValue, markdown: &str);
    fn _tachyon_editor_set_editable(view: &JsValue, editable: bool);
    fn _tachyon_editor_focus(view: &JsValue);
    fn _tachyon_editor_destroy(view: &JsValue);
    fn _tachyon_editor_dispatch_command(view: &JsValue, command: &str) -> bool;
}

/// Dispatch a formatting command on the currently active editor.
/// Used by toolbar buttons. The command is dispatched on the most recently
/// created ProseMirror editor instance.
///
/// Supported commands: bold, italic, code, strikethrough, h1, h2, h3,
/// bullet_list, ordered_list, blockquote, code_block, horizontal_rule,
/// undo, redo
pub fn dispatch_editor_command(command: &str) -> bool {
    #[wasm_bindgen(inline_js = r#"
export function __wasm_bindgen_dispatch_editor_command(commandName) {
    if (window.TachyonEditor) {
        return window.TachyonEditor.dispatchCommand(null, commandName);
    }
    return false;
}
"#)]
    extern "C" {
        fn __wasm_bindgen_dispatch_editor_command(command: &str) -> bool;
    }
    __wasm_bindgen_dispatch_editor_command(command)
}

// ============================================================================
// Editor Handle
// ============================================================================

/// Handle to a JavaScript TachyonEditor instance.
/// Wraps a JsValue (EditorView) and provides Rust-callable methods.
pub struct EditorHandle {
    view: JsValue,
}

impl EditorHandle {
    /// Get the current markdown content from the editor.
    pub fn get_markdown(&self) -> String {
        _tachyon_editor_get_markdown(&self.view)
    }

    /// Replace the editor content with new markdown.
    pub fn set_markdown(&self, markdown: &str) {
        _tachyon_editor_set_markdown(&self.view, markdown);
    }

    /// Set whether the editor is editable.
    pub fn set_editable(&self, editable: bool) {
        _tachyon_editor_set_editable(&self.view, editable);
    }

    /// Focus the editor.
    pub fn focus(&self) {
        _tachyon_editor_focus(&self.view);
    }

    /// Dispatch a named command on the editor.
    /// Supported: bold, italic, code, strikethrough, h1, h2, h3,
    /// bullet_list, ordered_list, blockquote, code_block, horizontal_rule,
    /// undo, redo
    pub fn dispatch_command(&self, command: &str) -> bool {
        _tachyon_editor_dispatch_command(&self.view, command)
    }

    /// Destroy the editor instance and clean up DOM.
    pub fn destroy(&self) {
        _tachyon_editor_destroy(&self.view);
    }
}

// ============================================================================
// Leptos Component
// ============================================================================

/// ProseMirror-based rich text editor component.
///
/// Renders a `<div>` container and mounts a ProseMirror editor on it
/// via the JavaScript bridge. Changes are propagated to the parent
/// via the `on_change` callback.
#[component]
pub fn ProseMirrorEditor(
    /// Initial markdown content.
    content: String,
    /// Whether the editor is editable.
    #[prop(default = true)]
    editable: bool,
    /// Placeholder text shown when the editor is empty.
    #[prop(default = "Start writing...".into())]
    placeholder: String,
    /// Additional CSS classes for the editor container.
    #[prop(default = String::new())]
    class_name: String,
    /// Callback fired when the editor content changes.
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    // Track whether the editor has been initialized.
    // We use Rc<RefCell<bool>> because we need interior mutability
    // from within the Effect closure, and we can't use signals here
    // (the editor is not reactive — it's an external JS object).
    let initialized = Rc::new(RefCell::new(false));

    // Store the EditorView JsValue. JsValue is not Send+Sync, so we
    // use Rc<RefCell<>> instead of StoredValue.
    let editor_view: Rc<RefCell<Option<JsValue>>> = Rc::new(RefCell::new(None));

    // Create a node reference for the container div
    let container_ref = NodeRef::<leptos::html::Div>::new();

    // Clone for use inside the effect
    let initialized_clone = initialized.clone();
    let view_clone = editor_view.clone();
    let on_change_for_effect = on_change.clone();

    // Effect: mount ProseMirror when the container is ready
    Effect::new(move |_| {
        if *initialized_clone.borrow() {
            return;
        }

        if let Some(container) = container_ref.get() {
            let element = container;

            // Build options object for the JS editor
            let options = js_sys::Object::new();

            // Set content
            js_sys::Reflect::set(
                &options,
                &JsValue::from_str("content"),
                &JsValue::from_str(&content),
            )
            .unwrap();

            // Set placeholder
            js_sys::Reflect::set(
                &options,
                &JsValue::from_str("placeholder"),
                &JsValue::from_str(&placeholder),
            )
            .unwrap();

            // Set className
            if !class_name.is_empty() {
                js_sys::Reflect::set(
                    &options,
                    &JsValue::from_str("className"),
                    &JsValue::from_str(&class_name),
                )
                .unwrap();
            }

            // Set editable
            js_sys::Reflect::set(
                &options,
                &JsValue::from_str("editable"),
                &JsValue::from_bool(editable),
            )
            .unwrap();

            // Set onChange callback (bridges JS → Rust)
            if let Some(ref cb) = on_change_for_effect {
                let cb = cb.clone();
                let on_change_wrapper = Closure::<dyn Fn(String)>::new(move |markdown: String| {
                    cb.run(markdown);
                });
                let on_change_js = on_change_wrapper
                    .as_ref()
                    .unchecked_ref::<js_sys::Function>();
                js_sys::Reflect::set(&options, &JsValue::from_str("onChange"), on_change_js)
                    .unwrap();
                // Keep the closure alive for the lifetime of the editor
                on_change_wrapper.forget();
            }

            // Create the editor
            let view = _tachyon_editor_create(&element, &options);

            if !view.is_null() && !view.is_undefined() {
                *view_clone.borrow_mut() = Some(view);
                *initialized_clone.borrow_mut() = true;
            }
        }
    });

    // Cleanup: destroy the editor when the component is unmounted.
    // We can't capture Rc<RefCell<>> in on_cleanup (requires Send+Sync),
    // so we rely on JavaScript's garbage collector. ProseMirror's EditorView
    // auto-cleans when its DOM element is removed. For explicit cleanup,
    // we could use a MutationObserver in the JS bridge.

    view! {
        <div
            node_ref=container_ref
            class="tachyon-editor-container"
        />
    }
}
