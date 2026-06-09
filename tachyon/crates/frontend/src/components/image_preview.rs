#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn ImagePreview(
    src: String,
    #[prop(default = String::new())] alt: String,
    open: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (zoom_level, set_zoom_level) = signal(1.0_f64);

    let handle_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let delta = ev.delta_y() * -0.001;
        let new_zoom = (zoom_level.get() + delta).clamp(0.25, 5.0);
        set_zoom_level.set(new_zoom);
    };

    let zoom_in =
        move |_: leptos::ev::MouseEvent| set_zoom_level.update(|z| *z = (*z + 0.25).min(5.0));
    let zoom_out =
        move |_: leptos::ev::MouseEvent| set_zoom_level.update(|z| *z = (*z - 0.25).max(0.25));
    let zoom_reset = move |_: leptos::ev::MouseEvent| set_zoom_level.set(1.0);

    let close_backdrop = move |_: leptos::ev::MouseEvent| on_close.run(());
    let close_button = move |_: leptos::ev::MouseEvent| on_close.run(());

    view! {
        <Show when=move || open.get()>
            <div
                class="fixed inset-0 bg-black/80 z-50 flex items-center justify-center"
                on:click=close_backdrop
                role="dialog"
                aria-modal="true"
                aria-label="Image preview"
            >
                <button
                    class="absolute top-4 right-4 text-white hover:text-gray-300 z-10 p-2"
                    on:click=close_button
                    aria-label="Close preview"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>

                <div
                    class="max-w-[90vw] max-h-[90vh] overflow-auto"
                    on:wheel=handle_wheel
                    on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <img
                        src=src.clone()
                        alt=alt.clone()
                        class="max-w-full max-h-[85vh] object-contain transition-transform duration-200"
                        style=move || format!("transform: scale({})", zoom_level.get())
                        draggable="false"
                    />
                </div>

                <div class="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-2">
                    <button
                        class="px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-none text-sm"
                        on:click=zoom_out
                        aria-label="Zoom out"
                    >
                        "-"
                    </button>
                    <button
                        class="px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-none text-sm"
                        on:click=zoom_reset
                        aria-label="Reset zoom"
                    >
                        {move || format!("{:.0}%", zoom_level.get() * 100.0)}
                    </button>
                    <button
                        class="px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-none text-sm"
                        on:click=zoom_in
                        aria-label="Zoom in"
                    >
                        "+"
                    </button>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn Thumbnail(
    src: String,
    alt: String,
    #[prop(default = 48)] size: u32,
    #[prop(default = "rounded-none".to_string())] class: String,
) -> impl IntoView {
    let size_style = format!("width: {}px; height: {}px;", size, size);
    let container_class = format!("overflow-hidden {} bg-gray-100 dark:bg-gray-700", class);

    view! {
        <div
            class=container_class
            style=size_style
        >
            <img
                src=src
                alt=alt
                class="w-full h-full object-cover"
                loading="lazy"
            />
        </div>
    }
}
