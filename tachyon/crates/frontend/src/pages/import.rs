use crate::api::ApiClient;
use crate::components::drop_zone::{DropZone, DroppedFile};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportSource {
    Obsidian,
    Notion,
    Confluence,
    MarkdownFiles,
}

impl ImportSource {
    fn label(&self) -> &'static str {
        match self {
            ImportSource::Obsidian => "Obsidian",
            ImportSource::Notion => "Notion",
            ImportSource::Confluence => "Confluence",
            ImportSource::MarkdownFiles => "Markdown Files",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ImportSource::Obsidian => "Import your Obsidian vault as a ZIP archive",
            ImportSource::Notion => "Import Notion pages from an export ZIP",
            ImportSource::Confluence => "Import Confluence spaces via XML export",
            ImportSource::MarkdownFiles => "Import standalone Markdown files",
        }
    }

    fn accept_types(&self) -> &'static str {
        match self {
            ImportSource::Confluence => ".xml",
            _ => ".zip,.md,.markdown",
        }
    }

    fn icon_svg(&self) -> &'static str {
        match self {
            ImportSource::Obsidian => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" /></svg>"#
            }
            ImportSource::Notion => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>"#
            }
            ImportSource::Confluence => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" /></svg>"#
            }
            ImportSource::MarkdownFiles => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>"#
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ImportConfig {
    convert_wikilinks: bool,
    infer_tags_from_paths: bool,
    api_key: String,
    confluence_url: String,
    confluence_user: String,
    confluence_token: String,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            convert_wikilinks: true,
            infer_tags_from_paths: true,
            api_key: String::new(),
            confluence_url: String::new(),
            confluence_user: String::new(),
            confluence_token: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportStep {
    Source,
    Upload,
    Config,
    Progress,
}

fn step_to_index(step: ImportStep) -> usize {
    match step {
        ImportStep::Source => 0,
        ImportStep::Upload => 1,
        ImportStep::Config => 2,
        ImportStep::Progress => 3,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ImportStatus {
    Idle,
    Uploading,
    Complete { documents_imported: u32 },
    Error(String),
}

fn format_size(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{} B", bytes)
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}

fn step_circle_class(current: ImportStep, step: ImportStep, index: usize) -> String {
    let current_idx = step_to_index(current);
    let _ = step;
    if index < current_idx {
        "flex items-center justify-center w-8 h-8 rounded-full bg-blue-600 text-white text-sm font-medium".to_string()
    } else if index == current_idx {
        "flex items-center justify-center w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/40 text-blue-600 dark:text-blue-400 text-sm font-medium ring-2 ring-blue-600".to_string()
    } else {
        "flex items-center justify-center w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400 text-sm font-medium".to_string()
    }
}

fn step_text_class(current: ImportStep, index: usize) -> &'static str {
    let current_idx = step_to_index(current);
    if index <= current_idx {
        "ml-2 text-sm font-medium text-gray-900 dark:text-white"
    } else {
        "ml-2 text-sm font-medium text-gray-500 dark:text-gray-400"
    }
}

#[component]
pub fn ImportPage() -> impl IntoView {
    let (current_step, set_current_step) = signal(ImportStep::Source);
    let (selected_source, set_selected_source) = signal(None::<ImportSource>);
    let (uploaded_file, set_uploaded_file) = signal(None::<DroppedFile>);
    let config = RwSignal::new(ImportConfig::default());
    let (import_status, set_import_status) = signal(ImportStatus::Idle);
    let (progress, set_progress) = signal(0.0f64);

    let on_select_source = Callback::new(move |source: ImportSource| {
        set_selected_source.set(Some(source));
        set_current_step.set(ImportStep::Upload);
    });

    let on_files = Callback::new(move |files: Vec<DroppedFile>| {
        if let Some(file) = files.into_iter().next() {
            set_uploaded_file.set(Some(file));
            set_current_step.set(ImportStep::Config);
        }
    });

    let on_start_import = Callback::new(move |_: leptos::ev::MouseEvent| {
        let file = uploaded_file.get();
        let source = selected_source.get();
        let cfg = config.get();

        let Some(file_info) = file else {
            set_import_status.set(ImportStatus::Error("No file selected".to_string()));
            return;
        };
        let Some(source) = source else {
            set_import_status.set(ImportStatus::Error("No source selected".to_string()));
            return;
        };

        set_current_step.set(ImportStep::Progress);
        set_import_status.set(ImportStatus::Uploading);
        set_progress.set(0.0);

        let api_endpoint = match source {
            ImportSource::Obsidian => "/api/v1/import/obsidian",
            ImportSource::Notion => "/api/v1/import/notion",
            ImportSource::Confluence => "/api/v1/import/confluence",
            ImportSource::MarkdownFiles => "/api/v1/import/obsidian",
        };

        let api = ApiClient::default();
        let file = file_info.file.clone();
        let set_status = set_import_status;
        let set_prog = set_progress;
        let endpoint = api_endpoint.to_string();

        spawn_local(async move {
            set_prog.set(25.0);
            match upload_and_import(&api, &endpoint, &file, &cfg).await {
                Ok(count) => {
                    set_prog.set(100.0);
                    set_status.set(ImportStatus::Complete {
                        documents_imported: count,
                    });
                }
                Err(e) => {
                    set_status.set(ImportStatus::Error(e));
                }
            }
        });
    });

    let on_back = Callback::new(move |_: leptos::ev::MouseEvent| match current_step.get() {
        ImportStep::Upload => {
            set_current_step.set(ImportStep::Source);
            set_uploaded_file.set(None);
        }
        ImportStep::Config => {
            set_current_step.set(ImportStep::Upload);
        }
        ImportStep::Progress => {
            set_current_step.set(ImportStep::Config);
        }
        ImportStep::Source => {}
    });

    let on_reset = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_current_step.set(ImportStep::Source);
        set_selected_source.set(None);
        set_uploaded_file.set(None);
        config.set(ImportConfig::default());
        set_import_status.set(ImportStatus::Idle);
        set_progress.set(0.0);
    });

    view! {
        <div class="max-w-4xl mx-auto">
            <div class="mb-8">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Import Documents"</h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                    "Import documents from other platforms into Tachyon."
                </p>
            </div>

            <ImportStepper current_step />

            <div class="mt-8">
                <Show when={move || current_step.get() == ImportStep::Source}>
                    <SourceSelection on_select=on_select_source />
                </Show>

                <Show when={move || current_step.get() == ImportStep::Upload}>
                    <FileUploadStep
                        source=selected_source
                        uploaded_file=uploaded_file
                        on_files=on_files
                        on_back=on_back
                    />
                </Show>

                <Show when={move || current_step.get() == ImportStep::Config}>
                    <ConfigurationStep
                        source=selected_source
                        config=config
                        on_start=on_start_import
                        on_back=on_back
                    />
                </Show>

                <Show when={move || current_step.get() == ImportStep::Progress}>
                    <ProgressStep
                        status=import_status
                        progress=progress
                        on_reset=on_reset
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn ImportStepper(current_step: ReadSignal<ImportStep>) -> impl IntoView {
    let steps = [
        (ImportStep::Source, "Source"),
        (ImportStep::Upload, "Upload"),
        (ImportStep::Config, "Configure"),
        (ImportStep::Progress, "Import"),
    ];

    let check_svg = "<svg class=\"w-4 h-4\" fill=\"currentColor\" viewBox=\"0 0 20 20\"><path fill-rule=\"evenodd\" d=\"M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z\" clip-rule=\"evenodd\" /></svg>";
    let arrow_svg = "<svg class=\"w-5 h-5 text-gray-300 dark:text-gray-600\" fill=\"currentColor\" viewBox=\"0 0 20 20\"><path fill-rule=\"evenodd\" d=\"M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z\" clip-rule=\"evenodd\" /></svg>";

    let step_data: Vec<_> = steps
        .into_iter()
        .enumerate()
        .map(|(i, (_step, label))| {
            let circle = step_circle_class(current_step.get_untracked(), _step, i);
            let text = step_text_class(current_step.get_untracked(), i);
            let is_done = step_to_index(current_step.get_untracked()) > i;
            let num = (i + 1).to_string();
            let is_last = i == steps.len() - 1;
            (circle, text, is_done, num, label.to_string(), is_last)
        })
        .collect();

    view! {
        <nav aria-label="Import progress" class="mb-8">
            <ol class="flex items-center">
                {step_data
                    .into_iter()
                    .enumerate()
                    .map(|(idx, (circle, text, is_done_init, num, label, is_last))| {
                        let circle_val = circle.clone();
                        let text_val = text;
                        let num_val = num;
                        let label_val = label;
                        let check = check_svg;
                        let arrow = arrow_svg;
                        let _step_idx = idx;
                        view! {
                            <li class="flex items-center">
                                <div class="flex items-center">
                                    <span class=circle_val>
                                        {if is_done_init {
                                            view! { <span inner_html=check /> }.into_any()
                                        } else {
                                            view! { <span>{num_val}</span> }.into_any()
                                        }}
                                    </span>
                                    <span class=text_val>{label_val}</span>
                                </div>
                                {if !is_last {
                                    view! { <div class="hidden sm:block ml-4" inner_html=arrow /> }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ol>
        </nav>
    }
}

#[component]
fn SourceSelection(on_select: Callback<ImportSource>) -> impl IntoView {
    let sources = [
        ImportSource::Obsidian,
        ImportSource::Notion,
        ImportSource::Confluence,
        ImportSource::MarkdownFiles,
    ];

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {sources.into_iter().map(|source| {
                let src = source;
                let on_click = on_select;
                view! {
                    <button
                        class="flex flex-col items-start p-6 bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-400 rounded-none transition-colors text-left group"
                        on:click=move |_| on_click.run(src)
                    >
                        <div class="text-blue-600 dark:text-blue-400 mb-3" inner_html=source.icon_svg() />
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400">
                            {source.label()}
                        </h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            {source.description()}
                        </p>
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn FileUploadStep(
    source: ReadSignal<Option<ImportSource>>,
    uploaded_file: ReadSignal<Option<DroppedFile>>,
    on_files: Callback<Vec<DroppedFile>>,
    on_back: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    let accept = move || {
        source
            .get()
            .map(|s| s.accept_types().to_string())
            .unwrap_or_else(|| "*".to_string())
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <button
                    on:click=move |ev| on_back.run(ev)
                    class="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                    </svg>
                    "Back"
                </button>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    "Step 2 of 4"
                </span>
            </div>

            <div class="bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-none p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    "Upload File"
                </h2>
                <DropZone
                    label="Drop your import file here".to_string()
                    accept=accept()
                    multiple=false
                    on_files=on_files
                />
            </div>

            <Show when={move || uploaded_file.get().is_some()}>
                <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-none p-4">
                    <div class="flex items-center gap-3">
                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-medium text-blue-900 dark:text-blue-100 truncate">
                                {move || uploaded_file.get().map(|f| f.name).unwrap_or_default()}
                            </p>
                            <p class="text-xs text-blue-600 dark:text-blue-400">
                                {move || {
                                    uploaded_file.get().map(|f| format_size(f.size)).unwrap_or_default()
                                }}
                            </p>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ConfigurationStep(
    source: ReadSignal<Option<ImportSource>>,
    config: RwSignal<ImportConfig>,
    on_start: Callback<leptos::ev::MouseEvent>,
    on_back: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <button
                    on:click=move |ev| on_back.run(ev)
                    class="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                    </svg>
                    "Back"
                </button>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    "Step 3 of 4"
                </span>
            </div>

            <div class="bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-none p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    "Configuration"
                </h2>

                <Show when={move || source.get() == Some(ImportSource::Obsidian)}>
                    <div class="space-y-4">
                        <ObsidianConfig config />
                    </div>
                </Show>

                <Show when={move || source.get() == Some(ImportSource::Notion)}>
                    <div class="space-y-4">
                        <NotionConfig config />
                    </div>
                </Show>

                <Show when={move || source.get() == Some(ImportSource::Confluence)}>
                    <div class="space-y-4">
                        <ConfluenceConfig config />
                    </div>
                </Show>

                <Show when={move || source.get() == Some(ImportSource::MarkdownFiles)}>
                    <div class="space-y-4">
                        <MarkdownConfig config />
                    </div>
                </Show>
            </div>

            <div class="flex justify-end">
                <button
                    on:click=move |ev| on_start.run(ev)
                    class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-none transition-colors font-medium"
                >
                    "Start Import"
                </button>
            </div>
        </div>
    }
}

#[component]
fn ObsidianConfig(config: RwSignal<ImportConfig>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <label class="flex items-center gap-3 cursor-pointer">
                <input
                    type="checkbox"
                    prop:checked={move || config.get().convert_wikilinks}
                    on:change=move |_| {
                        config.update(|c| c.convert_wikilinks = !c.convert_wikilinks);
                    }
                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                />
                <div>
                    <span class="text-sm font-medium text-gray-900 dark:text-white">"Convert wiki-links"</span>
                    <p class="text-xs text-gray-500 dark:text-gray-400">"Convert [[wiki-links]] to standard Markdown links"</p>
                </div>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
                <input
                    type="checkbox"
                    prop:checked={move || config.get().infer_tags_from_paths}
                    on:change=move |_| {
                        config.update(|c| c.infer_tags_from_paths = !c.infer_tags_from_paths);
                    }
                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                />
                <div>
                    <span class="text-sm font-medium text-gray-900 dark:text-white">"Infer tags from paths"</span>
                    <p class="text-xs text-gray-500 dark:text-gray-400">"Generate tags based on folder structure"</p>
                </div>
            </label>
        </div>
    }
}

#[component]
fn NotionConfig(config: RwSignal<ImportConfig>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div>
                <label for="notion-api-key" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    "Notion API Key (optional)"
                </label>
                <input
                    id="notion-api-key"
                    type="password"
                    placeholder="ntn_..."
                    prop:value={move || config.get().api_key}
                    on:input=move |ev| {
                        config.update(|c| c.api_key = event_target_value(&ev));
                    }
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                />
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    "Required for API-based import. For file export, leave empty."
                </p>
            </div>
        </div>
    }
}

#[component]
fn ConfluenceConfig(config: RwSignal<ImportConfig>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div>
                <label for="confluence-url" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    "Confluence URL"
                </label>
                <input
                    id="confluence-url"
                    type="url"
                    placeholder="https://your-domain.atlassian.net/wiki"
                    prop:value={move || config.get().confluence_url}
                    on:input=move |ev| {
                        config.update(|c| c.confluence_url = event_target_value(&ev));
                    }
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                />
            </div>
            <div>
                <label for="confluence-user" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    "Username (optional)"
                </label>
                <input
                    id="confluence-user"
                    type="text"
                    placeholder="user@example.com"
                    prop:value={move || config.get().confluence_user}
                    on:input=move |ev| {
                        config.update(|c| c.confluence_user = event_target_value(&ev));
                    }
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                />
            </div>
            <div>
                <label for="confluence-token" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    "API Token (optional)"
                </label>
                <input
                    id="confluence-token"
                    type="password"
                    placeholder="Your API token"
                    prop:value={move || config.get().confluence_token}
                    on:input=move |ev| {
                        config.update(|c| c.confluence_token = event_target_value(&ev));
                    }
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                />
            </div>
        </div>
    }
}

#[component]
fn MarkdownConfig(config: RwSignal<ImportConfig>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <label class="flex items-center gap-3 cursor-pointer">
                <input
                    type="checkbox"
                    prop:checked={move || config.get().infer_tags_from_paths}
                    on:change=move |_| {
                        config.update(|c| c.infer_tags_from_paths = !c.infer_tags_from_paths);
                    }
                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                />
                <div>
                    <span class="text-sm font-medium text-gray-900 dark:text-white">"Infer tags from paths"</span>
                    <p class="text-xs text-gray-500 dark:text-gray-400">"Generate tags based on folder structure"</p>
                </div>
            </label>
        </div>
    }
}

#[component]
fn ProgressStep(
    status: ReadSignal<ImportStatus>,
    progress: ReadSignal<f64>,
    on_reset: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    "Step 4 of 4"
                </span>
            </div>

            <div class="bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-none p-6">
                {move || match status.get() {
                    ImportStatus::Idle => {
                        view! { <p class="text-gray-500">"Preparing import..."</p> }.into_any()
                    }
                    ImportStatus::Uploading => {
                        let p = progress.get();
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-3">
                                    <div class="animate-spin rounded-full h-5 w-5 border-2 border-blue-600 border-t-transparent"></div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-white">
                                        "Uploading and processing..."
                                    </span>
                                </div>
                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
                                    <div
                                        class="bg-blue-600 h-2.5 rounded-full transition-all duration-300"
                                        style=format!("width: {}%", p)
                                        role="progressbar"
                                        aria-valuenow=p as i32
                                        aria-valuemin=0
                                        aria-valuemax=100
                                    ></div>
                                </div>
                                <p class="text-xs text-gray-500 dark:text-gray-400">
                                    "This may take a while for large archives."
                                </p>
                            </div>
                        }.into_any()
                    }
                    ImportStatus::Complete { documents_imported } => {
                        let plural = if documents_imported == 1 { "" } else { "s" };
                        let msg = format!("{} document{} imported successfully", documents_imported, plural);
                        let nav = navigate.clone();
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-3">
                                    <svg class="w-8 h-8 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                    </svg>
                                    <div>
                                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                            "Import Complete"
                                        </h3>
                                        <p class="text-sm text-gray-500 dark:text-gray-400">
                                            {msg}
                                        </p>
                                    </div>
                                </div>
                                <div class="flex gap-3">
                                    <button
                                        on:click=move |_| { nav("/documents", Default::default()); }
                                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-none transition-colors text-sm font-medium"
                                    >
                                        "View Documents"
                                    </button>
                                    <button
                                        on:click=move |ev| on_reset.run(ev)
                                        class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-none transition-colors text-sm font-medium"
                                    >
                                        "Import More"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                    ImportStatus::Error(ref msg) => {
                        let err_msg = msg.clone();
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-3">
                                    <svg class="w-8 h-8 text-red-500" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                    </svg>
                                    <div>
                                        <h3 class="text-lg font-semibold text-red-800 dark:text-red-300">
                                            "Import Failed"
                                        </h3>
                                        <p class="text-sm text-red-600 dark:text-red-400">
                                            {err_msg}
                                        </p>
                                    </div>
                                </div>
                                <button
                                    on:click=move |ev| on_reset.run(ev)
                                    class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-none transition-colors text-sm font-medium"
                                >
                                    "Try Again"
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

async fn upload_and_import(
    api: &ApiClient,
    endpoint: &str,
    file: &web_sys::File,
    _config: &ImportConfig,
) -> Result<u32, String> {
    use wasm_bindgen::JsCast;

    let form_data =
        web_sys::FormData::new().map_err(|e| format!("Failed to create form data: {:?}", e))?;
    form_data
        .append_with_str("file", &file.name())
        .map_err(|e| format!("Failed to append file name: {:?}", e))?;
    form_data
        .append_with_blob("file", file)
        .map_err(|e| format!("Failed to append file blob: {:?}", e))?;

    let origin = web_sys::window()
        .ok_or("No window")?
        .location()
        .origin()
        .map_err(|e| format!("Failed to get origin: {:?}", e))?;
    let url = format!("{}{}", origin, endpoint);

    let headers =
        web_sys::Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    if let Some(token) = api.get_auth_token() {
        headers
            .set("Authorization", &format!("Bearer {}", token))
            .map_err(|e| format!("Failed to set auth header: {:?}", e))?;
    }

    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    let headers_js: JsValue = headers.into();
    init.set_headers(&headers_js);
    let body_js: JsValue = form_data.into();
    init.set_body(&body_js);

    let request = web_sys::Request::new_with_str_and_init(&url, &init)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = js_sys::Reflect::get(&window, &JsValue::from_str("fetch"))
        .map_err(|e| format!("Failed to get fetch: {:?}", e))?;
    let fetch_fn: js_sys::Function = resp_value.unchecked_into();
    let resp_promise: js_sys::Promise = fetch_fn
        .call1(&window, &request.into())
        .map_err(|e| format!("Failed to call fetch: {:?}", e))?
        .unchecked_into();

    let resp_val: JsValue = wasm_bindgen_futures::JsFuture::from(resp_promise)
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    let resp: web_sys::Response = resp_val.unchecked_into();

    if !resp.ok() {
        let status = resp.status();
        let text_future = resp
            .text()
            .map_err(|e| format!("Failed to get response text: {:?}", e))?;
        let text_val: JsValue = wasm_bindgen_futures::JsFuture::from(text_future)
            .await
            .map_err(|e| format!("Failed to read text: {:?}", e))?;
        let text_str: String = text_val
            .dyn_into::<js_sys::JsString>()
            .map_err(|_| "Failed to cast to JsString".to_string())?
            .into();
        return Err(format!("HTTP {}: {}", status, text_str));
    }

    let json_future = resp
        .json()
        .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;
    let json_val: JsValue = wasm_bindgen_futures::JsFuture::from(json_future)
        .await
        .map_err(|e| format!("Failed to read JSON: {:?}", e))?;

    let json: serde_json::Value = serde_wasm_bindgen::from_value(json_val)
        .map_err(|e| format!("Failed to deserialize: {}", e))?;

    let count = json
        .get("documents_imported")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    Ok(count)
}
