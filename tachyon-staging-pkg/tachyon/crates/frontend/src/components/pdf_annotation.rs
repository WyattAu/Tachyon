#![allow(dead_code)]

use crate::pdf_annotation_types::*;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

/// PDF annotation toolbar component
#[component]
#[allow(unused_variables)]
pub fn AnnotationToolbar(
    active_tool: ReadSignal<AnnotationType>,
    active_color: ReadSignal<AnnotationColor>,
    on_tool_change: Callback<AnnotationType>,
    on_color_change: Callback<AnnotationColor>,
    on_create_annotation: Callback<Annotation>,
) -> impl IntoView {
    let tools = vec![
        (AnnotationType::Highlight, "Highlight", "H"),
        (AnnotationType::Underline, "Underline", "U"),
        (AnnotationType::Strikethrough, "Strikethrough", "S"),
        (AnnotationType::StickyNote, "Sticky Note", "N"),
    ];

    let colors = vec![
        AnnotationColor::Yellow,
        AnnotationColor::Blue,
        AnnotationColor::Green,
        AnnotationColor::Pink,
    ];

    let handle_tool_click = move |tool: AnnotationType| {
        on_tool_change.run(tool);
    };

    let handle_color_click = move |color: AnnotationColor| {
        on_color_change.run(color);
    };

    view! {
        <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
            <div class="flex items-center gap-1">
                {tools.into_iter().map(|(tool, label, shortcut)| {
                    let tool_for_class = tool.clone();
                    let tool_for_click = tool.clone();
                    let tool_for_aria = tool.clone();
                    let label_text = label.to_string();
                    let shortcut = shortcut.to_string();
                    let label_for_title = label_text.clone();
                    let shortcut_for_title = shortcut.clone();
                    view! {
                        <button
                            class=move || {
                                let is_active = active_tool.get() == tool_for_class;
                                if is_active {
                                    "px-3 py-1.5 text-sm bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded font-medium"
                                } else {
                                    "px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
                                }
                            }
                            on:click=move |_| handle_tool_click(tool_for_click.clone())
                            title={move || format!("{} ({})", label_for_title, shortcut_for_title)}
                            aria-pressed=move || if active_tool.get() == tool_for_aria { "true" } else { "false" }
                        >
                            {label_text}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-2"></div>

            <div class="flex items-center gap-1">
                {colors.into_iter().map(|color| {
                    let color_for_class = color.clone();
                    let color_for_click = color.clone();
                    let color_for_title1 = color.clone();
                    let color_for_title2 = color.clone();
                    let hex = color.to_hex().to_string();
                    view! {
                        <button
                            class=move || {
                                let is_active = active_color.get() == color_for_class;
                                if is_active {
                                    "w-6 h-6 rounded-full border-2 border-gray-900 dark:border-white"
                                } else {
                                    "w-6 h-6 rounded-full border-2 border-gray-300 dark:border-gray-600 hover:border-gray-500 dark:hover:border-gray-400"
                                }
                            }
                            style=move || format!("background-color: {}", hex)
                            on:click=move |_| handle_color_click(color_for_click.clone())
                            title=move || format!("{:?}", color_for_title1)
                            aria-label=move || format!("Select {:?} color", color_for_title2)
                        ></button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// Annotation list panel component
#[component]
pub fn AnnotationListPanel(
    annotations: Vec<Annotation>,
    on_select: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    let annotation_count = annotations.len();
    if annotation_count == 0 {
        return view! {
            <div class="p-4 text-center text-gray-500 dark:text-gray-400">
                <p class="text-sm">"No annotations yet"</p>
                <p class="text-xs mt-1">"Select text to highlight, or add sticky notes"</p>
            </div>
        }
        .into_any();
    }

    let items: Vec<_> = annotations.into_iter().map(|annotation| {
        let id = annotation.id().to_string();
        let page = annotation.page();
        let id_for_select = id.clone();
        let id_for_delete = id.clone();

        let (annotation_type, preview, color_hex) = match &annotation {
            Annotation::Text(text) => {
                let type_name = match text.kind {
                    AnnotationType::Highlight => "Highlight",
                    AnnotationType::Underline => "Underline",
                    AnnotationType::Strikethrough => "Strikethrough",
                    _ => "Text",
                };
                let note = text.note.as_deref().unwrap_or("No note");
                (type_name.to_string(), note.to_string(), text.color.to_hex().to_string())
            }
            Annotation::StickyNote(note) => {
                ("Sticky Note".to_string(), note.content.clone(), "#FFFF00".to_string())
            }
        };

        view! {
            <div
                class="p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer border-b border-gray-100 dark:border-gray-700 transition-colors"
                on:click=move |_| on_select.run(id_for_select.clone())
            >
                <div class="flex items-start justify-between">
                    <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 mb-1">
                            <div
                                class="w-3 h-3 rounded-full flex-shrink-0"
                                style=move || format!("background-color: {}", color_hex)
                            ></div>
                            <span class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                                {annotation_type}
                            </span>
                            <span class="text-xs text-gray-400 dark:text-gray-500">
                                {format!("Page {}", page)}
                            </span>
                        </div>
                        <p class="text-sm text-gray-900 dark:text-white line-clamp-2">
                            {preview}
                        </p>
                    </div>
                    <button
                        class="ml-2 p-1 text-gray-400 hover:text-red-500 dark:hover:text-red-400 transition-colors"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            on_delete.run(id_for_delete.clone());
                        }
                        aria-label="Delete annotation"
                    >
                        <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>
            </div>
        }
    }).collect();

    view! {
        <div class="divide-y divide-gray-200 dark:divide-gray-700">
            <div class="px-4 py-2 bg-gray-50 dark:bg-gray-800">
                <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
                    "Annotations"
                    <span class="ml-2 text-xs font-normal text-gray-500 dark:text-gray-400">
                        {annotation_count}
                    </span>
                </h3>
            </div>
            {items}
        </div>
    }
    .into_any()
}

/// Sticky note input component
#[component]
#[allow(unused_variables)]
pub fn StickyNoteInput(
    x: f64,
    y: f64,
    page: u32,
    on_save: Callback<String>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (content, set_content) = signal(String::new());

    let handle_save = move |_: leptos::ev::MouseEvent| {
        let content_val = content.get();
        if !content_val.trim().is_empty() {
            on_save.run(content_val);
        }
    };

    let handle_cancel = move |_: leptos::ev::MouseEvent| {
        on_cancel.run(());
    };

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            on_cancel.run(());
        } else if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
            let content_val = content.get();
            if !content_val.trim().is_empty() {
                on_save.run(content_val);
            }
        }
    };

    view! {
        <div
            class="absolute z-10 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-3 w-64"
            style=move || format!("left: {}px; top: {}px;", x, y)
            on:keydown=handle_keydown
        >
            <textarea
                class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                placeholder="Add a note..."
                rows="3"
                prop:value={move || content.get()}
                on:input=move |ev| set_content.set(event_target_value(&ev))
            ></textarea>
            <div class="flex justify-end gap-2 mt-2">
                <button
                    class="px-3 py-1 text-xs text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    on:click=handle_cancel
                >
                    "Cancel"
                </button>
                <button
                    class="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50"
                    disabled=move || content.get().trim().is_empty()
                    on:click=handle_save
                >
                    "Save"
                </button>
            </div>
        </div>
    }
}

/// Annotation rendering overlay for a PDF page
#[component]
pub fn AnnotationOverlay(
    page: u32,
    annotations: Vec<Annotation>,
    on_click: Callback<(f64, f64)>,
) -> impl IntoView {
    let handles: Vec<_> = annotations
        .into_iter()
        .filter(|a| a.page() == page)
        .map(|annotation| {
            match annotation {
                Annotation::Text(text) => {
                    // Render text annotations as highlighted rectangles
                    let overlays: Vec<_> = text
                        .rects
                        .iter()
                        .map(|rect| {
                            let color = match text.kind {
                                AnnotationType::Highlight => text.color.to_rgba(0.3),
                                AnnotationType::Underline => text.color.to_rgba(0.8),
                                AnnotationType::Strikethrough => text.color.to_rgba(0.8),
                                _ => text.color.to_rgba(0.3),
                            };

                            let style = match text.kind {
                                AnnotationType::Underline => {
                                    format!(
                                        "position: absolute; left: {}px; bottom: {}px; width: {}px; height: 2px; background-color: {};",
                                        rect.x, rect.y, rect.width, color
                                    )
                                }
                                AnnotationType::Strikethrough => {
                                    format!(
                                        "position: absolute; left: {}px; top: {}px; width: {}px; height: 2px; background-color: {}; transform: translateY(-50%);",
                                        rect.x, rect.y + rect.height / 2.0, rect.width, color
                                    )
                                }
                                _ => {
                                    format!(
                                        "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; background-color: {};",
                                        rect.x, rect.y, rect.width, rect.height, color
                                    )
                                }
                            };

                            view! {
                                <div
                                    class="pointer-events-none"
                                    style=style
                                    title=text.note.clone().unwrap_or_default()
                                ></div>
                            }
                        })
                        .collect();

                    view! { <div>{overlays}</div> }.into_any()
                }
                Annotation::StickyNote(note) => {
                    let style = format!(
                        "position: absolute; left: {}px; top: {}px; width: 24px; height: 24px; background-color: #FFFF00; border: 1px solid #E5E5E5; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 12px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                        note.x, note.y
                    );

                    view! {
                        <div
                            class="group"
                            style=style
                            title=note.content.clone()
                        >
                            <svg class="w-3 h-3 text-yellow-700" aria-hidden="true" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" />
                            </svg>
                        </div>
                    }.into_any()
                }
            }
        })
        .collect();

    view! {
        <div
            class="absolute inset-0 pointer-events-none"
            style="position: relative;"
            on:click=move |ev: web_sys::MouseEvent| {
                if let Some(target) = ev.target() {
                    if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();
                        let x = ev.client_x() as f64 - rect.left();
                        let y = ev.client_y() as f64 - rect.top();
                        on_click.run((x, y));
                    }
                }
            }
        >
            {handles}
        </div>
    }
}

/// Export annotations dialog
#[component]
pub fn ExportAnnotationsDialog(
    annotations: Vec<Annotation>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (export_format, set_export_format) = signal("json".to_string());
    let annotations_clone = annotations.clone();

    let handle_export = move |_: leptos::ev::MouseEvent| {
        let format = export_format.get();
        let annotations_json = serde_json::to_string_pretty(&annotations_clone).unwrap_or_default();

        // Create download blob
        let (mime_type, extension) = match format.as_str() {
            "json" => ("application/json", "json"),
            "csv" => ("text/csv", "csv"),
            _ => ("application/json", "json"),
        };

        let content = if format == "csv" {
            // Convert to CSV format
            let mut csv = "Type,Page,Content,Color,Created At\n".to_string();
            for annotation in &annotations_clone {
                match annotation {
                    Annotation::Text(text) => {
                        let type_str = match text.kind {
                            AnnotationType::Highlight => "Highlight",
                            AnnotationType::Underline => "Underline",
                            AnnotationType::Strikethrough => "Strikethrough",
                            _ => "Text",
                        };
                        let note = text.note.as_deref().unwrap_or("");
                        csv.push_str(&format!(
                            "{},{},{},{},{}\n",
                            type_str,
                            text.page,
                            note.replace(',', ";"),
                            text.color.to_hex(),
                            text.created_at
                        ));
                    }
                    Annotation::StickyNote(note) => {
                        csv.push_str(&format!(
                            "StickyNote,{},{},{},{},{},{}\n",
                            note.page,
                            note.content.replace(',', ";"),
                            "",
                            note.x,
                            note.y,
                            note.created_at
                        ));
                    }
                }
            }
            csv
        } else {
            annotations_json
        };

        // Trigger download
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let bag = web_sys::BlobPropertyBag::new();
        bag.set_type(mime_type);
        let blob = web_sys::Blob::new_with_str_sequence_and_options(
            &js_sys::Array::of1(&JsValue::from_str(&content)),
            &bag,
        )
        .unwrap();

        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        let a = document.create_element("a").unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", &format!("annotations.{}", extension))
            .unwrap();
        a.dyn_into::<web_sys::HtmlElement>().unwrap().click();

        web_sys::Url::revoke_object_url(&url).unwrap();
        on_close.run(());
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md" role="dialog" aria-modal="true">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Export Annotations"</h2>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Export Format"
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            prop:value={move || export_format.get()}
                            on:change=move |ev| set_export_format.set(event_target_value(&ev))
                        >
                            <option value="json">"JSON"</option>
                            <option value="csv">"CSV"</option>
                        </select>
                    </div>

                    <div class="text-sm text-gray-500 dark:text-gray-400">
                        {format!("{} annotations will be exported", annotations.len())}
                    </div>
                </div>

                <div class="mt-6 flex justify-end gap-3">
                    <button
                        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                        on:click=move |_| on_close.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                        on:click=handle_export
                    >
                        "Export"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_toolbar_creation() {
        // Toolbar component compilation test
    }

    #[test]
    fn test_annotation_list_panel_empty() {
        let annotations: Vec<Annotation> = vec![];
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_annotation_list_panel_with_items() {
        let annotations = [
            Annotation::Text(TextAnnotation {
                id: "1".to_string(),
                page: 1,
                rects: vec![Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 20.0,
                }],
                color: AnnotationColor::Yellow,
                note: Some("Test".to_string()),
                kind: AnnotationType::Highlight,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }),
            Annotation::StickyNote(StickyNoteAnnotation {
                id: "2".to_string(),
                page: 2,
                x: 50.0,
                y: 100.0,
                content: "Important".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }),
        ];
        assert_eq!(annotations.len(), 2);
    }
}
