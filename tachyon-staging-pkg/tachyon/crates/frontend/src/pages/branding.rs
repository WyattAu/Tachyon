use crate::api::ApiClient;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn BrandingPage() -> impl IntoView {
    let (company_name, set_company_name) = signal(String::new());
    let (primary_color, set_primary_color) = signal("#2563eb".to_string());
    let (secondary_color, set_secondary_color) = signal("#10b981".to_string());
    let (logo_url, set_logo_url) = signal(Option::<String>::None);
    let (logo_data_url, set_logo_data_url) = signal(Option::<String>::None);
    let (loaded, set_loaded) = signal(false);
    let (saving, set_saving) = signal(false);
    let (msg, set_msg) = signal(String::new());
    let (drag_over, set_drag_over) = signal(false);

    Effect::new(move |_| {
        let api = ApiClient::default();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_brand_settings().await {
                Ok(val) => {
                    set_company_name.set(
                        val.get("company_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    );
                    set_primary_color.set(
                        val.get("primary_color")
                            .and_then(|v| v.as_str())
                            .unwrap_or("#2563eb")
                            .to_string(),
                    );
                    set_secondary_color.set(
                        val.get("secondary_color")
                            .and_then(|v| v.as_str())
                            .unwrap_or("#10b981")
                            .to_string(),
                    );
                    set_logo_url.set(
                        val.get("logo_url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    );
                    set_loaded.set(true);
                }
                Err(_) => {
                    set_loaded.set(true);
                }
            }
        });
    });

    let on_file_select = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let set_data = set_logo_data_url;
                let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::ProgressEvent)>::new(
                    move |_: web_sys::ProgressEvent| {
                        if let Ok(val) = reader_clone.result() {
                            if let Some(s) = val.as_string() {
                                set_data.set(Some(s));
                            }
                        }
                    },
                );
                reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
                let _ = reader.read_as_data_url(&file);
            }
        }
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_drag_over.set(false);
        if let Some(data_transfer) = ev.data_transfer() {
            if let Some(files) = data_transfer.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let set_data = set_logo_data_url;
                    let closure =
                        wasm_bindgen::closure::Closure::<dyn Fn(web_sys::ProgressEvent)>::new(
                            move |_: web_sys::ProgressEvent| {
                                if let Ok(val) = reader_clone.result() {
                                    if let Some(s) = val.as_string() {
                                        set_data.set(Some(s));
                                    }
                                }
                            },
                        );
                    reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                    let _ = reader.read_as_data_url(&file);
                }
            }
        }
    };

    let on_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_drag_over.set(true);
    };

    let on_drag_leave = move |_: web_sys::DragEvent| {
        set_drag_over.set(false);
    };

    let on_save = move |_| {
        let client = ApiClient::default();
        let name = company_name.get();
        let primary = primary_color.get();
        let secondary = secondary_color.get();
        let logo = logo_data_url.get().or_else(|| logo_url.get());
        set_saving.set(true);
        set_msg.set(String::new());
        let set_msg_ref = set_msg;
        let set_saving_ref = set_saving;
        wasm_bindgen_futures::spawn_local(async move {
            match client
                .update_brand_settings(&name, &primary, &secondary, logo.as_deref())
                .await
            {
                Ok(_) => set_msg_ref.set("Branding saved successfully.".to_string()),
                Err(e) => set_msg_ref.set(format!("Failed to save: {}", e)),
            }
            set_saving_ref.set(false);
        });
    };

    let primary_preview = move || primary_color.get();
    let secondary_preview = move || secondary_color.get();
    let company_preview = move || {
        let name = company_name.get();
        if name.is_empty() {
            "Your Company".to_string()
        } else {
            name
        }
    };

    view! {
        <div class="max-w-4xl">
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Branding Settings"</h1>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                // Form
                <div class="space-y-6">
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Brand Configuration"</h2>
                        <div class="space-y-4">
                            <div>
                                <label for="brand-company-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Company Name"</label>
                                <input id="brand-company-name" type="text" prop:value={move || company_name.get()}
                                    on:input=move |ev| set_company_name.set(event_target_value(&ev))
                                    placeholder="Acme Corp"
                                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                    disabled=move || !loaded.get() />
                            </div>
                            <div>
                                <label for="brand-primary-color" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Primary Color"</label>
                                <div class="flex items-center gap-3">
                                    <input id="brand-primary-color" type="color" prop:value={move || primary_color.get()}
                                        on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                                        class="h-10 w-14 border border-gray-300 dark:border-gray-600 rounded cursor-pointer" />
                                    <input type="text" prop:value={move || primary_color.get()}
                                        on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                                        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono"
                                        disabled=move || !loaded.get() />
                                </div>
                            </div>
                            <div>
                                <label for="brand-secondary-color" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Secondary Color"</label>
                                <div class="flex items-center gap-3">
                                    <input id="brand-secondary-color" type="color" prop:value={move || secondary_color.get()}
                                        on:input=move |ev| set_secondary_color.set(event_target_value(&ev))
                                        class="h-10 w-14 border border-gray-300 dark:border-gray-600 rounded cursor-pointer" />
                                    <input type="text" prop:value={move || secondary_color.get()}
                                        on:input=move |ev| set_secondary_color.set(event_target_value(&ev))
                                        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono"
                                        disabled=move || !loaded.get() />
                                </div>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Logo"</label>
                                <div class="mt-1"
                                    on:drop=on_drop
                                    on:dragover=on_drag_over
                                    on:dragleave=on_drag_leave
                                >
                                    <label for="brand-logo-input" class={
                                        move || if drag_over.get() {
                                            "flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-blue-500 bg-blue-50 dark:bg-blue-900/20 cursor-pointer transition-colors"
                                        } else {
                                            "flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700/50 cursor-pointer hover:border-gray-400 dark:hover:border-gray-500 transition-colors"
                                        }
                                    }>
                                        {move || if logo_data_url.get().is_some() || logo_url.get().is_some() {
                                            view! {
                                                <div class="p-2 text-center">
                                                    <svg class="mx-auto h-8 w-8 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                                    </svg>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">"Logo selected. Drop another to replace."</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="p-2 text-center">
                                                    <svg class="mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                    </svg>
                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Drop logo here or click to browse"</p>
                                                </div>
                                            }.into_any()
                                        }}
                                    </label>
                                    <input id="brand-logo-input" type="file" accept="image/*" class="hidden" on:change=on_file_select />
                                </div>
                            </div>
                            <div class="flex items-center gap-3 pt-2">
                                <button on:click=on_save disabled=move || saving.get() || !loaded.get()
                                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-none transition-colors">
                                    {move || if saving.get() { "Saving..." } else { "Save Branding" }}
                                </button>
                                <span class="text-sm text-gray-600 dark:text-gray-400">{move || msg.get()}</span>
                            </div>
                        </div>
                    </div>
                </div>

                // Preview
                <div class="space-y-6">
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Preview"</h2>
                        <div class="border border-gray-200 dark:border-gray-700 rounded-none overflow-hidden">
                            // Header preview
                            <div class="p-4 flex items-center gap-3" style={move || format!("background-color: {}", primary_preview())}>
                                {move || {
                                    if let Some(ref data_url) = logo_data_url.get() {
                                        view! { <img src=data_url class="h-8 w-8 object-contain" alt="Logo" /> }.into_any()
                                    } else if let Some(ref url) = logo_url.get() {
                                        view! { <img src=url class="h-8 w-8 object-contain" alt="Logo" /> }.into_any()
                                    } else {
                                        view! {
                                            <div class="h-8 w-8 rounded flex items-center justify-center bg-white/20">
                                                <span class="text-white font-bold text-sm">{
                                                    move || company_preview().chars().next().unwrap_or('C').to_uppercase().to_string()
                                                }</span>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                                <span class="text-white font-semibold text-lg">{company_preview}</span>
                            </div>
                            // Body preview
                            <div class="p-4 bg-gray-50 dark:bg-gray-900">
                                <div class="flex gap-2 mb-3">
                                    <button class="px-3 py-1 text-white text-sm font-medium rounded-none" style={move || format!("background-color: {}", primary_preview())}>
                                        "Primary Action"
                                    </button>
                                    <button class="px-3 py-1 text-white text-sm font-medium rounded-none" style={move || format!("background-color: {}", secondary_preview())}>
                                        "Secondary Action"
                                    </button>
                                </div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    "This is a preview of how your branding will appear across the application."
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
