#![allow(dead_code)]
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "tachyon-editor-settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettingsData {
    pub font_size: u8,
    pub font_family: String,
    pub tab_size: u8,
    pub word_wrap: bool,
    pub line_numbers: bool,
    pub minimap: bool,
    pub auto_save: bool,
    pub auto_save_delay_ms: u32,
    pub bracket_matching: bool,
    pub auto_indent: bool,
    pub smart_home: bool,
    pub render_whitespace: bool,
    pub cursor_blink: bool,
    pub cursor_style: String,
    pub scroll_beyond_last_line: bool,
    pub highlight_active_line: bool,
    pub theme: String,
    pub keybindings: String,
    pub show_toolbar: bool,
    pub show_status_bar: bool,
    pub show_minimap: bool,
}

impl Default for EditorSettingsData {
    fn default() -> Self {
        Self {
            font_size: 14,
            font_family: "monospace".to_string(),
            tab_size: 4,
            word_wrap: true,
            line_numbers: true,
            minimap: false,
            auto_save: true,
            auto_save_delay_ms: 3000,
            bracket_matching: true,
            auto_indent: true,
            smart_home: true,
            render_whitespace: false,
            cursor_blink: true,
            cursor_style: "line".to_string(),
            scroll_beyond_last_line: true,
            highlight_active_line: true,
            theme: "dark".to_string(),
            keybindings: "default".to_string(),
            show_toolbar: true,
            show_status_bar: true,
            show_minimap: false,
        }
    }
}

impl EditorSettingsData {
    /// Load settings from localStorage.
    ///
    /// Reserved for future use: persistent editor preferences.
    fn load() -> Self {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Self::default(),
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return Self::default(),
        };
        match storage.get_item(STORAGE_KEY) {
            Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
            _ => Self::default(),
        }
    }

    /// Save settings to localStorage.
    ///
    /// Reserved for future use: persistent editor preferences.
    fn save(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };
        if let Ok(json) = serde_json::to_string(self) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}

/// Editor settings panel.
///
/// Reserved for future use: editor preference configuration UI.
#[component]
pub fn EditorSettings(
    visible: RwSignal<bool>,
    settings: RwSignal<EditorSettingsData>,
) -> impl IntoView {
    Effect::new(move |_| {
        let loaded = EditorSettingsData::load();
        settings.set(loaded);
    });

    let persist = move || {
        settings.with(|s| s.save());
    };

    let close = move |_: leptos::ev::MouseEvent| {
        visible.set(false);
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            visible.set(false);
        }
    };

    let dispatch_settings_changed = move || {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Ok(event) = doc.create_event("CustomEvent") {
                    event.init_event_with_bubbles_and_cancelable("tachyon:settings-changed", true, true);
                    let _ = window.dispatch_event(&event);
                }
            }
        }
    };

    let update_font_size = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev).parse::<u8>().unwrap_or(14);
        let clamped = val.clamp(12, 24);
        settings.update(|s| s.font_size = clamped);
        persist();
        dispatch_settings_changed();
    };

    let update_font_family = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        settings.update(|s| s.font_family = val);
        persist();
        dispatch_settings_changed();
    };

    let update_tab_size = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev).parse::<u8>().unwrap_or(4);
        let clamped = match val {
            2 => 2,
            8 => 8,
            _ => 4,
        };
        settings.update(|s| s.tab_size = clamped);
        persist();
        dispatch_settings_changed();
    };

    let update_cursor_style = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        settings.update(|s| s.cursor_style = val);
        persist();
        dispatch_settings_changed();
    };

    let update_theme = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        settings.update(|s| s.theme = val);
        persist();
        dispatch_settings_changed();
    };

    let update_keybindings = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        settings.update(|s| s.keybindings = val);
        persist();
        dispatch_settings_changed();
    };

    let update_auto_save_delay = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev).parse::<u32>().unwrap_or(3000);
        let clamped = val.clamp(1000, 10000);
        settings.update(|s| s.auto_save_delay_ms = clamped);
        persist();
        dispatch_settings_changed();
    };

    let toggle_bool = move |key: fn(&mut EditorSettingsData, bool)| move |ev: leptos::ev::Event| {
        let val = event_target_checked(&ev);
        settings.update(|s| key(s, val));
        persist();
        dispatch_settings_changed();
    };

    let toggle_word_wrap = toggle_bool(|s, v| s.word_wrap = v);
    let toggle_line_numbers = toggle_bool(|s, v| s.line_numbers = v);
    let toggle_auto_save = toggle_bool(|s, v| s.auto_save = v);
    let toggle_bracket_matching = toggle_bool(|s, v| s.bracket_matching = v);
    let toggle_auto_indent = toggle_bool(|s, v| s.auto_indent = v);
    let toggle_smart_home = toggle_bool(|s, v| s.smart_home = v);
    let toggle_render_whitespace = toggle_bool(|s, v| s.render_whitespace = v);
    let toggle_cursor_blink = toggle_bool(|s, v| s.cursor_blink = v);
    let toggle_scroll_beyond = toggle_bool(|s, v| s.scroll_beyond_last_line = v);
    let toggle_highlight_line = toggle_bool(|s, v| s.highlight_active_line = v);
    let toggle_show_toolbar = toggle_bool(|s, v| s.show_toolbar = v);
    let toggle_show_status_bar = toggle_bool(|s, v| s.show_status_bar = v);
    let toggle_show_minimap = toggle_bool(|s, v| s.show_minimap = v);

    view! {
        {move || if visible.get() {
            view! {
                <div class="editor-settings-overlay" on:keydown={on_keydown}>
                    <div class="editor-settings-backdrop" on:click={close}></div>
                    <div class="editor-settings-panel">
                        <div class="editor-settings-header">
                            <h2 class="editor-settings-title">{"Editor Settings"}</h2>
                            <button class="editor-settings-close" on:click={close} title="Close (Esc)">
                                { "\u{2715}" }
                            </button>
                        </div>

                        <div class="editor-settings-body">

                            // Appearance
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"Appearance"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Font Size"}</label>
                                    <div class="editor-settings-control editor-settings-range">
                                        <input
                                            type="range"
                                            min="12"
                                            max="24"
                                            prop:value={move || settings.with(|s| s.font_size.to_string())}
                                            on:input={update_font_size}
                                        />
                                        <span class="editor-settings-range-value">
                                            {move || settings.with(|s| s.font_size.to_string())}
                                            {"px"}
                                        </span>
                                    </div>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Font Family"}</label>
                                    <select
                                        class="editor-settings-select"
                                        prop:value={move || settings.with(|s| s.font_family.clone())}
                                        on:change={update_font_family}
                                    >
                                        <option value="monospace" selected={move || settings.with(|s| s.font_family == "monospace")}>
                                            {"Monospace"}
                                        </option>
                                        <option value="sans-serif" selected={move || settings.with(|s| s.font_family == "sans-serif")}>
                                            {"Sans Serif"}
                                        </option>
                                        <option value="serif" selected={move || settings.with(|s| s.font_family == "serif")}>
                                            {"Serif"}
                                        </option>
                                    </select>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Theme"}</label>
                                    <select
                                        class="editor-settings-select"
                                        prop:value={move || settings.with(|s| s.theme.clone())}
                                        on:change={update_theme}
                                    >
                                        <option value="dark" selected={move || settings.with(|s| s.theme == "dark")}>
                                            {"Dark"}
                                        </option>
                                        <option value="light" selected={move || settings.with(|s| s.theme == "light")}>
                                            {"Light"}
                                        </option>
                                        <option value="solarized" selected={move || settings.with(|s| s.theme == "solarized")}>
                                            {"Solarized"}
                                        </option>
                                        <option value="monokai" selected={move || settings.with(|s| s.theme == "monokai")}>
                                            {"Monokai"}
                                        </option>
                                    </select>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Cursor Style"}</label>
                                    <select
                                        class="editor-settings-select"
                                        prop:value={move || settings.with(|s| s.cursor_style.clone())}
                                        on:change={update_cursor_style}
                                    >
                                        <option value="line" selected={move || settings.with(|s| s.cursor_style == "line")}>
                                            {"Line"}
                                        </option>
                                        <option value="block" selected={move || settings.with(|s| s.cursor_style == "block")}>
                                            {"Block"}
                                        </option>
                                        <option value="underline" selected={move || settings.with(|s| s.cursor_style == "underline")}>
                                            {"Underline"}
                                        </option>
                                    </select>
                                </div>
                            </section>

                            // Editor Behavior
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"Editor Behavior"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Tab Size"}</label>
                                    <select
                                        class="editor-settings-select"
                                        prop:value={move || settings.with(|s| s.tab_size.to_string())}
                                        on:change={update_tab_size}
                                    >
                                        <option value="2" selected={move || settings.with(|s| s.tab_size == 2)}>
                                            {"2 spaces"}
                                        </option>
                                        <option value="4" selected={move || settings.with(|s| s.tab_size == 4)}>
                                            {"4 spaces"}
                                        </option>
                                        <option value="8" selected={move || settings.with(|s| s.tab_size == 8)}>
                                            {"8 spaces"}
                                        </option>
                                    </select>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Word Wrap"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.word_wrap)}
                                            on:change={toggle_word_wrap}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Line Numbers"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.line_numbers)}
                                            on:change={toggle_line_numbers}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Auto Indent"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.auto_indent)}
                                            on:change={toggle_auto_indent}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Smart Home"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.smart_home)}
                                            on:change={toggle_smart_home}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Bracket Matching"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.bracket_matching)}
                                            on:change={toggle_bracket_matching}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>
                            </section>

                            // Save
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"Save"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Auto Save"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.auto_save)}
                                            on:change={toggle_auto_save}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Auto Save Delay"}</label>
                                    <div class="editor-settings-control editor-settings-range">
                                        <input
                                            type="range"
                                            min="1000"
                                            max="10000"
                                            step="500"
                                            prop:value={move || settings.with(|s| s.auto_save_delay_ms.to_string())}
                                            on:input={update_auto_save_delay}
                                        />
                                        <span class="editor-settings-range-value">
                                            {move || {
                                                let ms = settings.with(|s| s.auto_save_delay_ms);
                                                if ms >= 1000 && ms % 1000 == 0 {
                                                    format!("{}s", ms / 1000)
                                                } else {
                                                    format!("{}ms", ms)
                                                }
                                            }}
                                        </span>
                                    </div>
                                </div>
                            </section>

                            // Advanced
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"Advanced"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Render Whitespace"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.render_whitespace)}
                                            on:change={toggle_render_whitespace}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Scroll Beyond Last Line"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.scroll_beyond_last_line)}
                                            on:change={toggle_scroll_beyond}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Highlight Active Line"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.highlight_active_line)}
                                            on:change={toggle_highlight_line}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Cursor Blink"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.cursor_blink)}
                                            on:change={toggle_cursor_blink}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>
                            </section>

                            // Keybindings
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"Keybindings"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Preset"}</label>
                                    <select
                                        class="editor-settings-select"
                                        prop:value={move || settings.with(|s| s.keybindings.clone())}
                                        on:change={update_keybindings}
                                    >
                                        <option value="default" selected={move || settings.with(|s| s.keybindings == "default")}>
                                            {"Default"}
                                        </option>
                                        <option value="vim" selected={move || settings.with(|s| s.keybindings == "vim")}>
                                            {"Vim"}
                                        </option>
                                        <option value="emacs" selected={move || settings.with(|s| s.keybindings == "emacs")}>
                                            {"Emacs"}
                                        </option>
                                    </select>
                                </div>
                            </section>

                            // UI Layout
                            <section class="editor-settings-section">
                                <h3 class="editor-settings-section-title">{"UI Layout"}</h3>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Show Toolbar"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.show_toolbar)}
                                            on:change={toggle_show_toolbar}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Show Status Bar"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.show_status_bar)}
                                            on:change={toggle_show_status_bar}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>

                                <div class="editor-settings-row">
                                    <label class="editor-settings-label">{"Show Minimap"}</label>
                                    <label class="editor-settings-toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked={move || settings.with(|s| s.show_minimap)}
                                            on:change={toggle_show_minimap}
                                        />
                                        <span class="editor-settings-toggle-slider"></span>
                                    </label>
                                </div>
                            </section>

                        </div>
                    </div>
                </div>
            }.into_any()
        } else {
            ().into_any()
        }}
    }
}
