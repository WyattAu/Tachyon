use leptos::*;

#[component]
pub fn FocusTrap(
    active: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let container_ref = create_node_ref::<html::Div>();

    create_effect(move |_| {
        if !active.get() {
            return;
        }

        let _ = container_ref.get().and_then(|el| {
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

#[component]
pub fn KeyboardShortcut(
    key: String,
    on_press: Callback<web_sys::KeyboardEvent>,
    #[prop(default = true)]
    active: bool,
) -> impl IntoView {
    let key_combo = key.clone();

    leptos::window_event_listener(ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if !active {
            return;
        }

        let parts: Vec<&str> = key_combo.split('+').collect();
        let target_key = parts.last().unwrap_or(&"");

        let ctrl = parts.contains(&"ctrl") || parts.contains(&"mod");
        let shift = parts.contains(&"shift");
        let alt = parts.contains(&"alt");

        let key_match = ev.key().to_lowercase() == *target_key
            || ev.code().to_lowercase() == *target_key;

        if key_match
            && (ctrl == ev.ctrl_key() || ctrl == ev.meta_key())
            && shift == ev.shift_key()
            && alt == ev.alt_key()
        {
            ev.prevent_default();
            on_press.call(ev);
        }
    });

    view! {}
}
