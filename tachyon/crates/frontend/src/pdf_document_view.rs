#![allow(dead_code)]

use crate::components::{
    AnnotationListPanel, AnnotationToolbar, ExportAnnotationsDialog, PdfViewer,
};
use crate::pdf_annotation_types::*;
use leptos::prelude::*;
use uuid::Uuid;

/// PDF document view component with annotation support
#[component]
pub fn PdfDocumentView(url: String, title: String) -> impl IntoView {
    let (annotations, set_annotations) = signal(Vec::<Annotation>::new());
    let (active_tool, set_active_tool) = signal(AnnotationType::Highlight);
    let (active_color, set_active_color) = signal(AnnotationColor::Yellow);
    let (show_annotation_panel, set_show_annotation_panel) = signal(true);
    let (show_export_dialog, set_show_export_dialog) = signal(false);
    let (current_page, set_current_page) = signal(1u32);
    let (pending_sticky_note, set_pending_sticky_note) = signal::<Option<(f64, f64)>>(None);

    // Handle text selection for annotation creation
    let handle_text_selection = move |(x, y): (f64, f64)| {
        let tool = active_tool.get();
        let color = active_color.get();

        match tool {
            AnnotationType::Highlight
            | AnnotationType::Underline
            | AnnotationType::Strikethrough => {
                // In a real implementation, we'd get the selected text rectangles from the PDF viewer
                // For now, we'll create a sample annotation
                let annotation = Annotation::Text(TextAnnotation {
                    id: Uuid::new_v4().to_string(),
                    page: current_page.get(),
                    rects: vec![Rect {
                        x: x,
                        y: y,
                        width: 100.0,
                        height: 20.0,
                    }],
                    color: color,
                    note: None,
                    kind: tool.clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                });
                set_annotations.update(|a| a.push(annotation));
            }
            AnnotationType::StickyNote => {
                set_pending_sticky_note.set(Some((x, y)));
            }
        }
    };

    // Handle sticky note creation
    let handle_sticky_note_save = move |content: String| {
        if let Some((x, y)) = pending_sticky_note.get() {
            let annotation = Annotation::StickyNote(StickyNoteAnnotation {
                id: Uuid::new_v4().to_string(),
                page: current_page.get(),
                x: x,
                y: y,
                content: content,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
            set_annotations.update(|a| a.push(annotation));
            set_pending_sticky_note.set(None);
        }
    };

    let handle_sticky_note_cancel = move |_: ()| {
        set_pending_sticky_note.set(None);
    };

    // Handle annotation selection
    let handle_annotation_select = move |id: String| {
        // In a real implementation, we'd scroll to the annotation
        web_sys::console::log_1(&format!("Selected annotation: {}", id).into());
    };

    // Handle annotation deletion
    let handle_annotation_delete = move |id: String| {
        set_annotations.update(|a| {
            a.retain(|annotation| annotation.id() != id);
        });
    };

    // Get annotation count for display
    let annotation_count = move || annotations.get().len();

    view! {
        <div class="flex h-full">
            // Main PDF viewer area
            <div class="flex-1 flex flex-col min-w-0">
                // Annotation toolbar
                <AnnotationToolbar
                    active_tool=active_tool
                    active_color=active_color
                    on_tool_change=Callback::new(move |tool: AnnotationType| set_active_tool.set(tool))
                    on_color_change=Callback::new(move |color: AnnotationColor| set_active_color.set(color))
                    on_create_annotation=Callback::new(move |_: Annotation| ())
                />

                // PDF viewer with annotation overlay
                <div class="flex-1 relative overflow-hidden">
                    <PdfViewer
                        url=url.clone()
                        title=title.clone()
                        on_close=None
                    />

                    // Annotation overlay (simplified - in real impl would be rendered on top of PDF pages)
                    <div class="absolute inset-0 pointer-events-auto" style="z-index: 10;">
                        // This would contain the actual annotation overlays in a real implementation
                    </div>
                </div>
            </div>

            // Annotation panel sidebar
            <Show when=move || show_annotation_panel.get()>
                <div class="w-80 border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
                    // Panel header
                    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
                            "Annotations"
                            <span class="ml-1 text-xs font-normal text-gray-500 dark:text-gray-400">
                                {move || format!("({})", annotation_count())}
                            </span>
                        </h3>
                        <div class="flex items-center gap-2">
                            <button
                                class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                on:click=move |_| set_show_export_dialog.set(true)
                                title="Export annotations"
                                aria-label="Export annotations"
                            >
                                <svg class="w-4 h-4 text-gray-500" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                            </button>
                            <button
                                class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                on:click=move |_| set_show_annotation_panel.update(|p| *p = !*p)
                                title="Toggle annotation panel"
                                aria-label="Toggle annotation panel"
                            >
                                <svg class="w-4 h-4 text-gray-500" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    // Annotation list
                    <div class="flex-1 overflow-y-auto">
                        <AnnotationListPanel
                            annotations=annotations.get()
                            on_select=Callback::new(handle_annotation_select)
                            on_delete=Callback::new(handle_annotation_delete)
                        />
                    </div>
                </div>
            </Show>

            // Show annotation panel button when hidden
            <Show when=move || !show_annotation_panel.get()>
                <button
                    class="fixed right-4 top-1/2 -translate-y-1/2 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-l-lg shadow-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors z-20"
                    on:click=move |_| set_show_annotation_panel.set(true)
                    title="Show annotations"
                    aria-label="Show annotations"
                >
                    <svg class="w-5 h-5 text-gray-500" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
                    </svg>
                </button>
            </Show>

            // Pending sticky note input
            {move || {
                if let Some((x, y)) = pending_sticky_note.get() {
                    view! {
                        <crate::components::StickyNoteInput
                            x=x
                            y=y
                            page=current_page.get()
                            on_save=Callback::new(handle_sticky_note_save)
                            on_cancel=Callback::new(handle_sticky_note_cancel)
                        />
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Export dialog
            {move || if show_export_dialog.get() {
                view! {
                    <ExportAnnotationsDialog
                        annotations=annotations.get()
                        on_close=Callback::new(move |_: ()| set_show_export_dialog.set(false))
                    />
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_document_view_creation() {
        // Component compilation test
    }

    #[test]
    fn test_annotation_creation() {
        let annotation = Annotation::Text(TextAnnotation {
            id: Uuid::new_v4().to_string(),
            page: 1,
            rects: vec![Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 20.0,
            }],
            color: AnnotationColor::Yellow,
            note: Some("Test note".to_string()),
            kind: AnnotationType::Highlight,
            created_at: chrono::Utc::now().to_rfc3339(),
        });

        assert!(!annotation.id().is_empty());
        assert_eq!(annotation.page(), 1);
    }
}
