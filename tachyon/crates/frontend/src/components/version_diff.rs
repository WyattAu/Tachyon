use crate::api::ApiClient;
use crate::api::versions::VersionDiffLine;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Side-by-side diff view component for comparing two documentation versions.
/// Displays line-by-line changes with additions highlighted in green and
/// deletions in red.
#[component]
pub fn VersionDiffView(
    version_a_id: String,
    version_b_id: String,
    document_slug: String,
) -> impl IntoView {
    let api_client = Rc::new(RefCell::new(ApiClient::default()));
    let (navigate_idx, set_navigate_idx) = signal(0usize);

    let diff_resource = LocalResource::new({
        let api_client = api_client.clone();
        let va = version_a_id.clone();
        let vb = version_b_id.clone();
        let slug = document_slug.clone();
        move || {
            let client = api_client.borrow().clone();
            let a = va.clone();
            let b = vb.clone();
            let s = slug.clone();
            async move { client.diff_doc_versions(&a, &b, &s).await.ok() }
        }
    });

    let total_changes = move || {
        diff_resource
            .get()
            .map(|diff| diff.map(|d| d.stats.added + d.stats.removed).unwrap_or(0))
            .unwrap_or(0)
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                <div>
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        "Version Diff"
                    </h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                        "Comparing "{version_a_id}" → " {version_b_id}" for "{document_slug}
                    </p>
                </div>
                <DiffStatsBadge />
            </div>

            <Suspense fallback=view! {
                <div class="p-8 text-center text-gray-500 dark:text-gray-400">
                    <svg class="w-8 h-8 mx-auto mb-3 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    "Loading diff..."
                </div>
            }>
                {move || {
                    diff_resource.get().map(|maybe_diff| {
                        match maybe_diff {
                            Some(diff) => {
                                if diff.stats.added == 0 && diff.stats.removed == 0 {
                                    view! {
                                        <div class="p-8 text-center">
                                            <svg class="w-12 h-12 mx-auto mb-3 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                            </svg>
                                            <p class="text-gray-700 dark:text-gray-300 font-medium">"No differences found"</p>
                                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"These versions are identical for this document"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let old = diff.old_lines;
                                    let new = diff.new_lines;
                                    view! {
                                        <DiffNavigation
                                            navigate_idx=navigate_idx
                                            set_navigate_idx=set_navigate_idx
                                            total_changes=total_changes
                                        />
                                        <SideBySideDiff old_lines=old new_lines=new />
                                    }.into_any()
                                }
                            }
                            None => view! {
                                <div class="p-8 text-center text-red-500">
                                    "Failed to load diff. Please try again."
                                </div>
                            }.into_any(),
                        }
                    }).unwrap_or_else(|| view! {
                        <div class="p-8 text-center text-gray-500">"Select versions to compare"</div>
                    }.into_any())
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn DiffStatsBadge() -> impl IntoView {
    // This is a placeholder; real stats come from the diff_resource.
    // We render it as part of the header.
    view! { <span></span> }
}

#[component]
fn DiffNavigation(
    navigate_idx: ReadSignal<usize>,
    set_navigate_idx: WriteSignal<usize>,
    total_changes: impl Fn() -> usize + 'static,
) -> impl IntoView {
    let total = total_changes();
    view! {
        <div class="px-4 py-2 bg-gray-50 dark:bg-gray-750 border-b border-gray-200 dark:border-gray-700 flex items-center gap-3 text-sm">
            <span class="text-gray-600 dark:text-gray-400">
                {format!("{} changes", total)}
            </span>
            <div class="flex items-center gap-1">
                <button
                    class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-500 disabled:opacity-30"
                    disabled=move || navigate_idx.get() == 0
                    on:click=move |_| set_navigate_idx.update(|i| *i = i.saturating_sub(1))
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
                    </svg>
                </button>
                <button
                    class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-500 disabled:opacity-30"
                    disabled=move || navigate_idx.get() >= total.saturating_sub(1)
                    on:click=move |_| set_navigate_idx.update(|i| *i = (*i + 1).min(total.saturating_sub(1)))
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </button>
            </div>
            <span class="text-gray-500 dark:text-gray-400">
                {move || format!("{}/{}", navigate_idx.get() + 1, total)}
            </span>
        </div>
    }
}

#[component]
fn SideBySideDiff(
    old_lines: Vec<VersionDiffLine>,
    new_lines: Vec<VersionDiffLine>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 divide-x divide-gray-200 dark:divide-gray-700 max-h-[600px] overflow-auto">
            <DiffPane
                label="Old Version"
                lines=old_lines
                side="old"
            />
            <DiffPane
                label="New Version"
                lines=new_lines
                side="new"
            />
        </div>
    }
}

#[component]
fn DiffPane(label: &'static str, lines: Vec<VersionDiffLine>, side: &'static str) -> impl IntoView {
    let pane_id = format!("diff-pane-{}", side);

    view! {
        <div>
            <div class="sticky top-0 z-10 bg-gray-100 dark:bg-gray-700 px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700">
                {label}
            </div>
            <div id=pane_id class="font-mono text-sm">
                {lines.into_iter().enumerate().map(|(i, line)| {
                    let bg_class = match line.line_type.as_str() {
                        "added" => "bg-green-50 dark:bg-green-900/20 border-l-2 border-green-400 dark:border-green-600",
                        "removed" => "bg-red-50 dark:bg-red-900/20 border-l-2 border-red-400 dark:border-red-600",
                        _ => "border-l-2 border-transparent",
                    };
                    let prefix = match line.line_type.as_str() {
                        "added" => "+",
                        "removed" => "-",
                        _ => " ",
                    };
                    let text_color = match line.line_type.as_str() {
                        "added" => "text-green-800 dark:text-green-200",
                        "removed" => "text-red-800 dark:text-red-200",
                        _ => "text-gray-700 dark:text-gray-300",
                    };
                    let display = if line.content.is_empty() {
                        " ".to_string()
                    } else {
                        line.content.clone()
                    };

                    view! {
                        <div class=format!("px-3 py-0.5 flex items-center {} {}", bg_class, text_color)>
                            <span class="text-gray-400 dark:text-gray-500 w-10 shrink-0 text-right select-none">{i + 1}</span>
                            <span class="w-5 shrink-0 text-center font-bold">{prefix}</span>
                            <span class="ml-2 flex-1 whitespace-pre-wrap break-all">{display}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::api::versions::{VersionDiffLine, VersionDiffResponse, VersionDiffStats};

    #[test]
    fn test_version_diff_line_types() {
        let added = VersionDiffLine {
            content: "+ new".to_string(),
            line_type: "added".to_string(),
        };
        let removed = VersionDiffLine {
            content: "- old".to_string(),
            line_type: "removed".to_string(),
        };
        let unchanged = VersionDiffLine {
            content: " same".to_string(),
            line_type: "unchanged".to_string(),
        };
        assert_eq!(added.line_type, "added");
        assert_eq!(removed.line_type, "removed");
        assert_eq!(unchanged.line_type, "unchanged");
    }

    #[test]
    fn test_version_diff_stats() {
        let stats = VersionDiffStats {
            added: 5,
            removed: 3,
            unchanged: 10,
        };
        assert_eq!(stats.added, 5);
        assert_eq!(stats.removed, 3);
        assert_eq!(stats.unchanged, 10);
    }

    #[test]
    fn test_version_diff_response_serialization() {
        let resp = VersionDiffResponse {
            document_slug: "test-doc".to_string(),
            old_lines: vec![VersionDiffLine {
                content: "old".to_string(),
                line_type: "removed".to_string(),
            }],
            new_lines: vec![VersionDiffLine {
                content: "new".to_string(),
                line_type: "added".to_string(),
            }],
            stats: VersionDiffStats {
                added: 1,
                removed: 1,
                unchanged: 0,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test-doc"));
        assert!(json.contains("added"));
        assert!(json.contains("removed"));
    }
}
