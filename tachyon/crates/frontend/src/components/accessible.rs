use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn VisuallyHidden(
    children: Children,
) -> impl IntoView {
    view! {
        <span class="sr-only">
            {children()}
        </span>
    }
}

#[component]
pub fn LiveRegion(
    #[prop(default = "polite".to_string())]
    aria_live: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="status"
            aria-live=aria_live
            class="sr-only"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn AccessibleDialog(
    open: Signal<bool>,
    title: String,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let dialog_ref = create_node_ref::<html::Dialog>();
    let previous_focus = StoredValue::new(None::<web_sys::Element>);

    create_effect(move |_| {
        if let Some(dialog) = dialog_ref.get() {
            if open.get() {
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    previous_focus.set_value(doc.active_element());
                }
                let _ = dialog.show_modal();
                let _ = dialog.focus();
            } else {
                let _ = dialog.close();
                if let Some(el) = previous_focus.get_value() {
                    let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
                }
            }
        }
    });

    view! {
        <dialog
            node_ref=dialog_ref
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 rounded-lg shadow-xl p-6 max-w-lg w-full"
            on:close=move |_| on_close.call(())
            aria-labelledby="dialog-title"
        >
            <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full">
                <header class="flex justify-between items-center mb-4">
                    <h2 id="dialog-title" class="text-lg font-semibold">{title}</h2>
                    <button
                        on:click=move |_| on_close.call(())
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                        aria-label="Close dialog"
                    >
                        "\u{00d7}"
                    </button>
                </header>
                <div>
                    {children()}
                </div>
            </div>
        </dialog>
    }
}
