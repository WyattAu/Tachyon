use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub name: String,
    pub brand_primary: String,
    pub brand_secondary: String,
    pub bg_primary: String,
    pub bg_secondary: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub border_color: String,
    pub radius_sm: String,
    pub radius_md: String,
    pub radius_lg: String,
    pub shadow_sm: String,
    pub shadow_md: String,
    pub shadow_lg: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            brand_primary: "#3B82F6".to_string(),
            brand_secondary: "#10B981".to_string(),
            bg_primary: "#ffffff".to_string(),
            bg_secondary: "#f9fafb".to_string(),
            text_primary: "#111827".to_string(),
            text_secondary: "#6b7280".to_string(),
            border_color: "#e5e7eb".to_string(),
            radius_sm: "0px".to_string(),
            radius_md: "0px".to_string(),
            radius_lg: "0px".to_string(),
            shadow_sm: "0 1px 2px rgba(0,0,0,0.05)".to_string(),
            shadow_md: "0 4px 6px rgba(0,0,0,0.1)".to_string(),
            shadow_lg: "0 10px 15px rgba(0,0,0,0.1)".to_string(),
        }
    }
}

fn load_theme_from_storage() -> ThemeConfig {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item("tachyon-custom-theme") {
                if let Ok(config) = serde_json::from_str::<ThemeConfig>(&json) {
                    return config;
                }
            }
        }
    }
    ThemeConfig::default()
}

fn save_theme_to_storage(config: &ThemeConfig) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(config) {
                let _ = storage.set_item("tachyon-custom-theme", &json);
            }
        }
    }
}

fn apply_theme_css(config: &ThemeConfig) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let css = format!(
                r#"
:root {{
    --brand-primary: {bp};
    --brand-secondary: {bs};
    --bg-primary: {bgp};
    --bg-secondary: {bgs};
    --text-primary: {tp};
    --text-secondary: {ts};
    --border-color: {bc};
    --radius-sm: {rs};
    --radius-md: {rm};
    --radius-lg: {rl};
    --shadow-sm: {ss};
    --shadow-md: {sm};
    --shadow-lg: {sl};
}}
"#,
                bp = config.brand_primary,
                bs = config.brand_secondary,
                bgp = config.bg_primary,
                bgs = config.bg_secondary,
                tp = config.text_primary,
                ts = config.text_secondary,
                bc = config.border_color,
                rs = config.radius_sm,
                rm = config.radius_md,
                rl = config.radius_lg,
                ss = config.shadow_sm,
                sm = config.shadow_md,
                sl = config.shadow_lg,
            );

            let style_id = "tachyon-theme-override";
            if let Some(existing) = document.get_element_by_id(style_id) {
                if let Some(parent) = existing.parent_node() {
                    let _ = parent.remove_child(&existing);
                }
            }

            let style_el = document.create_element("style").unwrap();
            style_el.set_id(style_id);
            style_el.set_text_content(Some(&css));
            if let Some(head) = document.head() {
                let _ = head.append_child(&style_el);
            }
        }
    }
}

fn export_theme_json(config: &ThemeConfig) -> String {
    serde_json::to_string_pretty(config).unwrap_or_default()
}

fn import_theme_json(json: &str) -> Option<ThemeConfig> {
    serde_json::from_str(json).ok()
}

#[component]
pub fn ThemesPage() -> impl IntoView {
    let initial = load_theme_from_storage();
    let (config, set_config) = signal(initial.clone());
    let (theme_name, set_theme_name) = signal(initial.name.clone());
    let (import_json, set_import_json) = signal(String::new());
    let (msg, set_msg) = signal(String::new());
    let (saving, set_saving) = signal(false);

    let on_apply = move |_| {
        set_saving.set(true);
        set_msg.set(String::new());
        let mut new_config = config.get();
        new_config.name = theme_name.get();
        apply_theme_css(&new_config);
        save_theme_to_storage(&new_config);
        set_config.set(new_config);
        set_msg.set("Theme applied successfully.".to_string());
        set_saving.set(false);
    };

    let on_export = move |_| {
        let mut export_config = config.get();
        export_config.name = theme_name.get();
        let json = export_theme_json(&export_config);
        if let Some(_window) = web_sys::window() {
            let bag = web_sys::BlobPropertyBag::new();
            bag.set_type("application/json");
            let blob = web_sys::Blob::new_with_str_sequence_and_options(
                &js_sys::Array::of1(&JsValue::from_str(&json)),
                &bag,
            )
            .unwrap();
            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let a: web_sys::HtmlAnchorElement =
                document().create_element("a").unwrap().unchecked_into();
            a.set_href(&url);
            a.set_download(&format!("{}.json", theme_name.get()));
            a.click();
            let _ = web_sys::Url::revoke_object_url(&url);
        }
        set_msg.set("Theme exported.".to_string());
    };

    let on_import = move |_| {
        let json = import_json.get();
        if let Some(imported) = import_theme_json(&json) {
            set_theme_name.set(imported.name.clone());
            set_config.set(imported);
            set_import_json.set(String::new());
            set_msg.set("Theme imported. Click Apply to activate.".to_string());
        } else {
            set_msg.set("Invalid theme JSON.".to_string());
        }
    };

    let on_reset = move |_| {
        let default = ThemeConfig::default();
        set_theme_name.set(default.name.clone());
        set_config.set(default);
        set_msg.set("Reset to defaults. Click Apply to activate.".to_string());
    };

    let on_import_file = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let set_json = set_import_json;
                let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::ProgressEvent)>::new(
                    move |_: web_sys::ProgressEvent| {
                        if let Ok(val) = reader_clone.result() {
                            if let Some(s) = val.as_string() {
                                set_json.set(s);
                            }
                        }
                    },
                );
                reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
                let _ = reader.read_as_text(&file);
            }
        }
    };

    let preview_config = config;

    view! {
        <div class="max-w-5xl">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Theme Editor"</h1>
                <div class="flex items-center gap-2">
                    <button on:click=on_reset
                        class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors">
                        "Reset"
                    </button>
                    <button on:click=on_export
                        class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors">
                        "Export JSON"
                    </button>
                    <label class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors cursor-pointer">
                        "Import JSON"
                        <input type="file" accept=".json" class="hidden" on:change=on_import_file />
                    </label>
                    <button on:click=on_apply disabled=move || saving.get()
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white text-sm font-medium rounded-none transition-colors">
                        {move || if saving.get() { "Applying..." } else { "Apply Theme" }}
                    </button>
                </div>
            </div>

            {move || if !msg.get().is_empty() {
                view! {
                    <div class="mb-4 px-4 py-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-800 dark:text-green-300 text-sm rounded-none">
                        {msg.get()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                // Editor
                <div class="space-y-6">
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Theme Name"</h2>
                        <input type="text" prop:value={move || theme_name.get()}
                            on:input=move |ev| set_theme_name.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            placeholder="My Theme" />
                    </div>

                    <ColorSection
                        title="Brand Colors"
                        config=config
                        set_config=set_config
                        fields=vec![
                            ("brand_primary".to_string(), "Primary Color".to_string()),
                            ("brand_secondary".to_string(), "Secondary Color".to_string()),
                        ]
                    />

                    <ColorSection
                        title="Background Colors"
                        config=config
                        set_config=set_config
                        fields=vec![
                            ("bg_primary".to_string(), "Primary Background".to_string()),
                            ("bg_secondary".to_string(), "Secondary Background".to_string()),
                        ]
                    />

                    <ColorSection
                        title="Text Colors"
                        config=config
                        set_config=set_config
                        fields=vec![
                            ("text_primary".to_string(), "Primary Text".to_string()),
                            ("text_secondary".to_string(), "Secondary Text".to_string()),
                            ("border_color".to_string(), "Border Color".to_string()),
                        ]
                    />

                    <RadiusSection config=config set_config=set_config />

                    <ShadowSection config=config set_config=set_config />

                    // Import textarea
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Import from JSON"</h2>
                        <textarea
                            prop:value={move || import_json.get()}
                            on:input=move |ev| set_import_json.set(event_target_value(&ev))
                            rows="6"
                            placeholder="{\"name\":\"My Theme\",\"brand_primary\":\"#3B82F6\",...}"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                        ></textarea>
                        <button on:click=on_import
                            class="mt-3 px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm rounded-none transition-colors">
                            "Import"
                        </button>
                    </div>
                </div>

                // Preview
                <div class="space-y-6">
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Live Preview"</h2>
                        <div class="border border-gray-200 dark:border-gray-700 rounded-none overflow-hidden">
                            // Header preview
                            <div class="p-4 flex items-center gap-3" style={move || format!("background-color: {}", preview_config.get().brand_primary)}>
                                <div class="h-8 w-8 rounded flex items-center justify-center bg-white/20">
                                    <span class="text-white font-bold text-sm">"T"</span>
                                </div>
                                <span class="text-white font-semibold text-lg">{move || theme_name.get()}</span>
                            </div>
                            // Body preview
                            <div class="p-4" style={move || format!("background-color: {}", preview_config.get().bg_primary)}>
                                <div class="flex gap-2 mb-3">
                                    <button class="px-3 py-1 text-white text-sm font-medium rounded-none" style={move || format!("background-color: {}", preview_config.get().brand_primary)}>
                                        "Primary Action"
                                    </button>
                                    <button class="px-3 py-1 text-white text-sm font-medium rounded-none" style={move || format!("background-color: {}", preview_config.get().brand_secondary)}>
                                        "Secondary Action"
                                    </button>
                                </div>
                                <p class="text-sm" style={move || format!("color: {}", preview_config.get().text_primary)}>
                                    "This is how your theme will look across the application."
                                </p>
                                <p class="text-xs mt-1" style={move || format!("color: {}", preview_config.get().text_secondary)}>
                                    "Secondary text appears lighter for supporting content."
                                </p>
                                <div class="mt-3 p-3 border rounded-none" style={move || format!("border-color: {}; background-color: {}", preview_config.get().border_color, preview_config.get().bg_secondary)}>
                                    <p class="text-sm" style={move || format!("color: {}", preview_config.get().text_primary)}>
                                        "Card content with secondary background."
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // CSS output
                    <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"CSS Variables"</h2>
                        <pre class="p-3 bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-none overflow-x-auto text-xs font-mono" style={move || format!("color: {}", preview_config.get().text_primary)}>{move || {
                            let c = preview_config.get();
                            format!(
r#":root {{
  --brand-primary: {};
  --brand-secondary: {};
  --bg-primary: {};
  --bg-secondary: {};
  --text-primary: {};
  --text-secondary: {};
  --border-color: {};
  --radius-sm: {};
  --radius-md: {};
  --radius-lg: {};
  --shadow-sm: {};
  --shadow-md: {};
  --shadow-lg: {};
}}"#,
                                c.brand_primary, c.brand_secondary,
                                c.bg_primary, c.bg_secondary,
                                c.text_primary, c.text_secondary,
                                c.border_color,
                                c.radius_sm, c.radius_md, c.radius_lg,
                                c.shadow_sm, c.shadow_md, c.shadow_lg,
                            )
                        }}</pre>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ColorSection(
    title: &'static str,
    config: ReadSignal<ThemeConfig>,
    set_config: WriteSignal<ThemeConfig>,
    fields: Vec<(String, String)>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{title}</h2>
            <div class="space-y-4">
                {fields.into_iter().map(|(field, label)| {
                    let field_clone = field.clone();
                    let setter = set_config;
                    let current_val = move || {
                        let c = config.get();
                        match field_clone.as_str() {
                            "brand_primary" => c.brand_primary,
                            "brand_secondary" => c.brand_secondary,
                            "bg_primary" => c.bg_primary,
                            "bg_secondary" => c.bg_secondary,
                            "text_primary" => c.text_primary,
                            "text_secondary" => c.text_secondary,
                            "border_color" => c.border_color,
                            _ => "#000000".to_string(),
                        }
                    };
                    let field_clone2 = field.clone();
                    let on_input = move |ev: web_sys::Event| {
                        let val = event_target_value(&ev);
                        let mut c = config.get();
                        match field_clone2.as_str() {
                            "brand_primary" => c.brand_primary = val,
                            "brand_secondary" => c.brand_secondary = val,
                            "bg_primary" => c.bg_primary = val,
                            "bg_secondary" => c.bg_secondary = val,
                            "text_primary" => c.text_primary = val,
                            "text_secondary" => c.text_secondary = val,
                            "border_color" => c.border_color = val,
                            _ => {}
                        }
                        setter.set(c);
                    };
                    view! {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{label}</label>
                            <div class="flex items-center gap-3">
                                <input type="color" prop:value=current_val.clone()
                                    on:input=on_input.clone()
                                    class="h-10 w-14 border border-gray-300 dark:border-gray-600 rounded cursor-pointer" />
                                <input type="text" prop:value=current_val
                                    on:input=on_input
                                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono" />
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn RadiusSection(
    config: ReadSignal<ThemeConfig>,
    set_config: WriteSignal<ThemeConfig>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Border Radius"</h2>
            <div class="space-y-4">
                {vec![
                    ("sm".to_string(), "Small".to_string()),
                    ("md".to_string(), "Medium".to_string()),
                    ("lg".to_string(), "Large".to_string()),
                ].into_iter().map(|(size, label)| {
                    let size_clone = size.clone();
                    let setter = set_config;
                    let current_val = move || {
                        let c = config.get();
                        match size_clone.as_str() {
                            "sm" => c.radius_sm,
                            "md" => c.radius_md,
                            "lg" => c.radius_lg,
                            _ => "0px".to_string(),
                        }
                    };
                    let size_clone2 = size.clone();
                    let on_input = move |ev: web_sys::Event| {
                        let val = event_target_value(&ev);
                        let mut c = config.get();
                        match size_clone2.as_str() {
                            "sm" => c.radius_sm = val,
                            "md" => c.radius_md = val,
                            "lg" => c.radius_lg = val,
                            _ => {}
                        }
                        setter.set(c);
                    };
                    view! {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{label}</label>
                            <input type="text" prop:value=current_val
                                on:input=on_input
                                placeholder="0px"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono" />
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn ShadowSection(
    config: ReadSignal<ThemeConfig>,
    set_config: WriteSignal<ThemeConfig>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Shadows"</h2>
            <div class="space-y-4">
                {vec![
                    ("sm".to_string(), "Small".to_string()),
                    ("md".to_string(), "Medium".to_string()),
                    ("lg".to_string(), "Large".to_string()),
                ].into_iter().map(|(size, label)| {
                    let size_clone = size.clone();
                    let setter = set_config;
                    let current_val = move || {
                        let c = config.get();
                        match size_clone.as_str() {
                            "sm" => c.shadow_sm,
                            "md" => c.shadow_md,
                            "lg" => c.shadow_lg,
                            _ => "none".to_string(),
                        }
                    };
                    let size_clone2 = size.clone();
                    let on_input = move |ev: web_sys::Event| {
                        let val = event_target_value(&ev);
                        let mut c = config.get();
                        match size_clone2.as_str() {
                            "sm" => c.shadow_sm = val,
                            "md" => c.shadow_md = val,
                            "lg" => c.shadow_lg = val,
                            _ => {}
                        }
                        setter.set(c);
                    };
                    view! {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{label}</label>
                            <input type="text" prop:value=current_val
                                on:input=on_input
                                placeholder="0 1px 2px rgba(0,0,0,0.05)"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono" />
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
