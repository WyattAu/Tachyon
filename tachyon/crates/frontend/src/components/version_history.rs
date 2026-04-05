// Version History Component
// Displays document version history with diff view and rollback functionality
//
// Note: Component props are used in closures but Rust's dead code analysis
// doesn't detect this due to Leptos macro expansion.

#![allow(dead_code)]

use leptos::prelude::*;
use crate::api::ApiClient;
use crate::types::DocumentVersion;
use std::sync::{Arc, Mutex};

/// Version History component - displays list of document versions with diff view
#[component]
pub fn VersionHistory(
    document_id: String,
    on_rollback: Option<Callback<String>>,
) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));
    let (selected_version, set_selected_version) = signal(None::<i32>);
    let (compare_version, set_compare_version) = signal(None::<i32>);
    let (show_diff, set_show_diff) = signal(false);

    let versions_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let client = api_client.lock().unwrap().clone();
            let doc_id = document_id.clone();
            async move {
                client.list_versions(&doc_id).await.unwrap_or_default()
            }
        }
    });

    let on_rollback_click = {
        move |version_id: String| {
            if let Some(callback) = on_rollback {
                callback.run(version_id);
            }
        }
    };

    let doc_id_for_list = document_id.clone();
    let doc_id_for_diff = document_id.clone();
    
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Version History"</h3>
                <button
                    type="button"
                    class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    on:click=move |_| set_show_diff.update(|v| *v = !*v)
                >
                    {move || if show_diff.get() { "Hide Diff" } else { "Compare Versions" }}
                </button>
            </div>

            <Suspense fallback=view! { <div class="p-4 text-gray-500">"Loading versions..."</div> }>
                <VersionList
                    _document_id=doc_id_for_list
                    versions_resource=versions_resource
                    selected_version=selected_version
                    set_selected_version=set_selected_version
                    compare_version=compare_version
                    set_compare_version=set_compare_version
                    show_diff=show_diff
                    on_rollback=on_rollback_click
                />
            </Suspense>

            <DiffViewSection
                document_id=doc_id_for_diff
                show_diff=show_diff
                selected_version=selected_version
                compare_version=compare_version
            />
        </div>
    }
}

#[component]
fn DiffViewSection(
    document_id: String,
    show_diff: ReadSignal<bool>,
    selected_version: ReadSignal<Option<i32>>,
    compare_version: ReadSignal<Option<i32>>,
) -> impl IntoView {
    move || {
        if show_diff.get() {
            let v1 = selected_version.get();
            let v2 = compare_version.get();
            
            if let (Some(ver1), Some(ver2)) = (v1, v2) {
                view! {
                    <div class="border-t border-gray-200 dark:border-gray-700">
                        <VersionDiffView
                            document_id=document_id.clone()
                            version1=ver1
                            version2=ver2
                        />
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="p-4 border-t border-gray-200 dark:border-gray-700 text-center text-gray-500 dark:text-gray-400">
                        "Select two versions to compare"
                    </div>
                }.into_any()
            }
        } else {
            view! { <div></div> }.into_any()
        }
    }
}

#[component]
fn VersionList(
    _document_id: String,
    versions_resource: LocalResource<Vec<DocumentVersion>>,
    selected_version: ReadSignal<Option<i32>>,
    set_selected_version: WriteSignal<Option<i32>>,
    compare_version: ReadSignal<Option<i32>>,
    set_compare_version: WriteSignal<Option<i32>>,
    show_diff: ReadSignal<bool>,
    on_rollback: impl Fn(String) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    move || {
        let versions = versions_resource.get();
        versions.map(|versions| {
            if versions.is_empty() {
                view! {
                    <div class="p-4 text-center text-gray-500 dark:text-gray-400">
                        "No version history available"
                    </div>
                }.into_any()
            } else {
                let on_rb = on_rollback.clone();
                view! {
                    <ul class="divide-y divide-gray-200 dark:divide-gray-700">
                        <For
                            each=move || versions.clone()
                            key=|v| v.id.clone()
                            let:version
                        >
                            <VersionItem
                                version=version
                                selected_version=selected_version
                                set_selected_version=set_selected_version
                                compare_version=compare_version
                                set_compare_version=set_compare_version
                                show_diff=show_diff
                                on_rollback=on_rb.clone()
                            />
                        </For>
                    </ul>
                }.into_any()
            }
        }).unwrap_or_else(|| view! { <div></div> }.into_any())
    }
}

#[component]
fn VersionItem(
    version: DocumentVersion,
    selected_version: ReadSignal<Option<i32>>,
    set_selected_version: WriteSignal<Option<i32>>,
    compare_version: ReadSignal<Option<i32>>,
    set_compare_version: WriteSignal<Option<i32>>,
    show_diff: ReadSignal<bool>,
    on_rollback: impl Fn(String) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let version_id = version.id.clone();
    let version_number = version.version_number;
    let commit_msg = version.commit_message.clone().unwrap_or_else(|| "No commit message".to_string());
    let created_at = format_timestamp(&version.created_at);
    let created_by = version.created_by.clone();
    
    let is_selected = move || selected_version.get() == Some(version_number);
    let is_compare = move || compare_version.get() == Some(version_number);

    view! {
        <li class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700">
            <div class="flex items-start justify-between">
                <div class="flex items-start gap-3 flex-1">
                    {move || if show_diff.get() {
                        view! {
                            <div class="flex gap-2 mt-1">
                                <input
                                    type="radio"
                                    name="version1"
                                    class="mt-1"
                                    checked=is_selected()
                                    on:click=move |_| set_selected_version.set(Some(version_number))
                                />
                                <input
                                    type="radio"
                                    name="version2"
                                    class="mt-1"
                                    checked=is_compare()
                                    on:click=move |_| set_compare_version.set(Some(version_number))
                                />
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}
                    
                    <div class="flex-1">
                        <div class="flex items-center gap-2">
                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                "v"{version_number}
                            </span>
                            <span class="text-sm text-gray-500 dark:text-gray-400">
                                {created_at}
                            </span>
                        </div>
                        <p class="mt-1 text-sm text-gray-700 dark:text-gray-300">
                            {commit_msg}
                        </p>
                        <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
                            "by "{created_by}
                        </p>
                    </div>
                </div>
                
                {move || if !show_diff.get() {
                    let vid = version_id.clone();
                    let on_rb = on_rollback.clone();
                    Some(view! {
                        <button
                            class="px-3 py-1 text-sm text-blue-600 dark:text-blue-400 hover:underline"
                            on:click=move |_| on_rb(vid.clone())
                        >
                            "Rollback"
                        </button>
                    })
                } else {
                    None
                }}
            </div>
        </li>
    }
}

/// Version Diff View component - displays side-by-side diff between two versions
#[component]
pub fn VersionDiffView(
    document_id: String,
    version1: i32,
    version2: i32,
) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));
    
    let diff_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let client = api_client.lock().unwrap().clone();
            let doc_id = document_id.clone();
            let v1 = version1;
            let v2 = version2;
            async move {
                let result1 = client.get_version(&doc_id, v1).await;
                let result2 = client.get_version(&doc_id, v2).await;
                
                match (result1, result2) {
                    (Ok(ver1), Ok(ver2)) => Some((ver1, ver2)),
                    _ => None,
                }
            }
        }
    });

    view! {
        <div class="p-4">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                "Comparing v"{version1}" with v"{version2}
            </h4>
            
            <Suspense fallback=view! { <div class="text-gray-500">"Loading diff..."</div> }>
                {move || {
                    diff_resource.get().map(|maybe_versions| {
                        match maybe_versions {
                            Some((v1, v2)) => {
                                let diff = compute_diff(&v1.content, &v2.content);
                                render_diff(diff)
                            }
                            None => view! {
                                <div class="text-red-500">"Failed to load versions for comparison"</div>
                            }.into_any()
                        }
                    }).unwrap_or_else(|| view! { <div class="text-gray-500">"Loading..."</div> }.into_any())
                }}
            </Suspense>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DiffLineType {
    Unchanged,
    Added,
    Removed,
}

struct DiffLine {
    content: String,
    line_type: DiffLineType,
}

struct DiffResult {
    old_lines: Vec<DiffLine>,
    new_lines: Vec<DiffLine>,
}

/// Simple line-based diff computation
fn compute_diff(old_content: &str, new_content: &str) -> DiffResult {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    
    let mut result_old = Vec::new();
    let mut result_new = Vec::new();
    
    let lcs = longest_common_subsequence(&old_lines, &new_lines);
    
    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut lcs_idx = 0;
    
    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if lcs_idx < lcs.len() {
            let lcs_line = lcs[lcs_idx];
            
            while old_idx < old_lines.len() && old_lines[old_idx] != lcs_line {
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: DiffLineType::Removed,
                });
                result_new.push(DiffLine {
                    content: String::new(),
                    line_type: DiffLineType::Unchanged,
                });
                old_idx += 1;
            }
            
            while new_idx < new_lines.len() && new_lines[new_idx] != lcs_line {
                result_old.push(DiffLine {
                    content: String::new(),
                    line_type: DiffLineType::Unchanged,
                });
                result_new.push(DiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: DiffLineType::Added,
                });
                new_idx += 1;
            }
            
            if old_idx < old_lines.len() && new_idx < new_lines.len() {
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: DiffLineType::Unchanged,
                });
                result_new.push(DiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: DiffLineType::Unchanged,
                });
                old_idx += 1;
                new_idx += 1;
                lcs_idx += 1;
            }
        } else {
            while old_idx < old_lines.len() {
                result_old.push(DiffLine {
                    content: old_lines[old_idx].to_string(),
                    line_type: DiffLineType::Removed,
                });
                result_new.push(DiffLine {
                    content: String::new(),
                    line_type: DiffLineType::Unchanged,
                });
                old_idx += 1;
            }
            
            while new_idx < new_lines.len() {
                result_old.push(DiffLine {
                    content: String::new(),
                    line_type: DiffLineType::Unchanged,
                });
                result_new.push(DiffLine {
                    content: new_lines[new_idx].to_string(),
                    line_type: DiffLineType::Added,
                });
                new_idx += 1;
            }
        }
    }
    
    DiffResult {
        old_lines: result_old,
        new_lines: result_new,
    }
}

/// Compute longest common subsequence of lines
fn longest_common_subsequence<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<&'a str> {
    let m = old.len();
    let n = new.len();
    
    if m == 0 || n == 0 {
        return Vec::new();
    }
    
    let mut dp = vec![vec![0; n + 1]; m + 1];
    
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    
    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            result.push(old[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    
    result.reverse();
    result
}

/// Render diff result as side-by-side view
fn render_diff(diff: DiffResult) -> AnyView {
    let old_lines = diff.old_lines;
    let new_lines = diff.new_lines;
    
    view! {
        <div class="grid grid-cols-2 gap-4 text-sm font-mono">
            <div class="border border-gray-200 dark:border-gray-700 rounded overflow-hidden">
                <div class="bg-gray-100 dark:bg-gray-700 px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-400">
                    "Old Version"
                </div>
                <div class="divide-y divide-gray-100 dark:divide-gray-800 max-h-96 overflow-auto">
                    {old_lines.into_iter().enumerate().map(|(i, line)| {
                        let bg_class = match line.line_type {
                            DiffLineType::Removed => "bg-red-50 dark:bg-red-900/20",
                            _ => "",
                        };
                        let prefix = match line.line_type {
                            DiffLineType::Removed => "-",
                            DiffLineType::Added => "+",
                            DiffLineType::Unchanged => " ",
                        };
                        let display = if line.content.is_empty() { " ".to_string() } else { line.content.clone() };
                        view! {
                            <div class=format!("px-3 py-0.5 flex {}", bg_class)>
                                <span class="text-gray-400 dark:text-gray-500 w-8 shrink-0">{i + 1}</span>
                                <span>{prefix}</span>
                                <span class="ml-2 flex-1 whitespace-pre-wrap break-all text-gray-700 dark:text-gray-300">
                                    {display}
                                </span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            
            <div class="border border-gray-200 dark:border-gray-700 rounded overflow-hidden">
                <div class="bg-gray-100 dark:bg-gray-700 px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-400">
                    "New Version"
                </div>
                <div class="divide-y divide-gray-100 dark:divide-gray-800 max-h-96 overflow-auto">
                    {new_lines.into_iter().enumerate().map(|(i, line)| {
                        let bg_class = match line.line_type {
                            DiffLineType::Added => "bg-green-50 dark:bg-green-900/20",
                            _ => "",
                        };
                        let prefix = match line.line_type {
                            DiffLineType::Removed => "-",
                            DiffLineType::Added => "+",
                            DiffLineType::Unchanged => " ",
                        };
                        let display = if line.content.is_empty() { " ".to_string() } else { line.content.clone() };
                        view! {
                            <div class=format!("px-3 py-0.5 flex {}", bg_class)>
                                <span class="text-gray-400 dark:text-gray-500 w-8 shrink-0">{i + 1}</span>
                                <span>{prefix}</span>
                                <span class="ml-2 flex-1 whitespace-pre-wrap break-all text-gray-700 dark:text-gray-300">
                                    {display}
                                </span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
        
        <div class="mt-3 flex gap-4 text-xs text-gray-500 dark:text-gray-400">
            <span class="flex items-center gap-1">
                <span class="w-3 h-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded"></span>
                " Added"
            </span>
            <span class="flex items-center gap-1">
                <span class="w-3 h-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded"></span>
                " Removed"
            </span>
        </div>
    }.into_any()
}

/// Format ISO timestamp to human-readable format
fn format_timestamp(timestamp: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        timestamp.to_string()
    }
}
