use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn FocusTrap(active: Signal<bool>, children: Children) -> impl IntoView {
    let container_ref = NodeRef::new();

    Effect::new(move |_| {
        if !active.get() {
            return;
        }

        let _ =
            container_ref.get().and_then(|el: web_sys::HtmlDivElement| {
                let focusable = el.query_selector(
                "button, [href], input, select, textarea, [tabindex]:not([tabindex=\"-1\"])"
            ).ok()??;
                let elem: web_sys::HtmlElement = focusable.dyn_into().ok()?;
                elem.focus().ok()
            });
    });

    view! {
        <div node_ref=container_ref>
            {children()}
        </div>
    }
}
