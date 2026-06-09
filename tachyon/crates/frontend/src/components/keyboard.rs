use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

const FOCUSABLE: &str = "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"]):not([disabled])";

/// Focus trap component that cycles Tab/Shift+Tab within container.
/// When `active` is true, keyboard focus cannot escape the container.
#[component]
pub fn FocusTrap(active: Signal<bool>, children: Children) -> impl IntoView {
    let container_ref = NodeRef::new();

    // Focus first focusable element when trap activates
    Effect::new(move |_| {
        if !active.get() {
            return;
        }

        let _ = container_ref.get().and_then(|el: web_sys::HtmlDivElement| {
            let focusable = el.query_selector(FOCUSABLE).ok()??;
            let elem: web_sys::HtmlElement = focusable.dyn_into().ok()?;
            elem.focus().ok()
        });
    });

    // Trap Tab key within container
    Effect::new(move |_| {
        if !active.get() {
            return;
        }

        let Some(container) = container_ref.get() else {
            return;
        };

        let closure =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if ev.key() != "Tab" {
                    return;
                }

                let Ok(node_list) = container.query_selector_all(FOCUSABLE) else {
                    return;
                };

                let length = node_list.length();
                if length == 0 {
                    return;
                }

                let active_el: web_sys::Element = match document().active_element() {
                    Some(el) => el,
                    _ => return,
                };

                // Find the index of the currently focused element
                let mut current_idx: i32 = -1;
                for i in 0..length {
                    if let Some(node) = node_list.item(i) {
                        if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                            if active_el.is_same_node(Some(&el)) {
                                current_idx = i as i32;
                                break;
                            }
                        }
                    }
                }

                if current_idx < 0 {
                    return;
                }

                let is_shift = ev.shift_key();
                let next_idx = if is_shift {
                    if current_idx == 0 {
                        length as i32 - 1
                    } else {
                        current_idx - 1
                    }
                } else if current_idx >= length as i32 - 1 {
                    0
                } else {
                    current_idx + 1
                };

                ev.prevent_default();

                if let Some(node) = node_list.item(next_idx as u32) {
                    if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                }
            });

        let _ = document()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        // Prevent Closure from being dropped (would invalidate the listener)
        closure.forget();
    });

    view! {
        <div node_ref=container_ref>
            {children()}
        </div>
    }
}
