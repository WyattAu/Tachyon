// Templates Page
// Full gallery with CRUD management for document templates

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;
use crate::types::{DocumentTemplate, CreateTemplateRequest, UpdateTemplateRequest};

// ---------------------------------------------------------------------------
// Templates Page
// ---------------------------------------------------------------------------

/// Main templates page - gallery with category filtering and CRUD
#[component]
pub fn TemplatesPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let api_client_for_categories = api_client.clone();
    let api_client_for_templates = api_client.clone();

    let (selected_category, set_selected_category) = signal(None::<String>);
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (editing_template, set_editing_template) = signal(None::<DocumentTemplate>);
    let (preview_template, set_preview_template) = signal(None::<DocumentTemplate>);
    let (deleting_id, set_deleting_id) = signal(None::<String>);
    let (refresh_counter, set_refresh_counter) = signal(0u32);

    // Fetch categories
    let categories_resource = LocalResource::new(move || {
        let client = api_client_for_categories.clone();
        async move {
            client.list_template_categories().await.unwrap_or_default()
        }
    });

    // Fetch templates
    let templates_resource = LocalResource::new(move || {
        let client = api_client_for_templates.clone();
        let cat = selected_category.get();
        let _ = refresh_counter.get();
        async move {
            client.list_templates(cat.as_deref()).await.unwrap_or_default()
        }
    });

    // Category sidebar (extracted to avoid nested view! issue)
    let categories_view = move || {
        categories_resource.get().map(|categories| {
            view! {
                <ul class="space-y-0.5">
                    <li>
                        <button
                            class={move || format!(
                                "w-full text-left px-3 py-2 rounded-lg text-sm transition-colors {}",
                                if selected_category.get().is_none() {
                                    "bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 font-medium"
                                } else {
                                    "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                }
                            )}
                            on:click={move |_| set_selected_category.set(None)}
                        >
                            "All"
                        </button>
                    </li>
                    {categories.into_iter().map(|cat| {
                        let cat_for_class = cat.clone();
                        let cat_for_click = cat.clone();
                        let cat_for_display = cat.clone();
                        view! {
                            <li>
                                <button
                                    class={move || format!(
                                        "w-full text-left px-3 py-2 rounded-lg text-sm transition-colors {}",
                                        if selected_category.get() == Some(cat_for_class.clone()) {
                                            "bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 font-medium"
                                        } else {
                                            "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        }
                                    )}
                                    on:click={move |_| set_selected_category.set(Some(cat_for_click.clone()))}
                                >
                                    {cat_for_display}
                                </button>
                            </li>
                        }
                    }).collect::<Vec<_>>()}
                </ul>
            }
        })
    };

    // Template grid (extracted to avoid nested view! issue)
    let templates_view = move || {
        templates_resource.get().map(|templates| {
            if templates.is_empty() {
                view! {
                    <div class="text-center py-16">
                        <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4"
                             fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">
                            "No templates yet"
                        </h3>
                        <p class="text-gray-500 dark:text-gray-400 mb-4">
                            "Create your first template to get started"
                        </p>
                        <button
                            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                            on:click={move |_| set_show_create_modal.set(true)}
                        >
                            "Create Template"
                        </button>
                    </div>
                }.into_any()
            } else {
                let count = templates.len();
                view! {
                    <div>
                        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                            {format!("{} template{}", count, if count == 1 { "" } else { "s" })}
                        </p>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {templates.into_iter().map(|template| {
                                let t_preview = template.clone();
                                let t_edit = template.clone();
                                let t_delete_id = template.id.clone();
                                view! {
                                    <TemplateGridCard
                                        template=template
                                        on_preview={Callback::new(move |_| set_preview_template.set(Some(t_preview.clone())))}
                                        on_edit={Callback::new(move |_| set_editing_template.set(Some(t_edit.clone())))}
                                        on_delete={Callback::new(move |_| set_deleting_id.set(Some(t_delete_id.clone())))}
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }.into_any()
            }
        })
    };

    // Skeleton fallback
    let skeleton_grid = view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <TemplateCardSkeleton />
            <TemplateCardSkeleton />
            <TemplateCardSkeleton />
            <TemplateCardSkeleton />
            <TemplateCardSkeleton />
            <TemplateCardSkeleton />
        </div>
    };

    // Edit modal view
    let edit_modal_view = move || {
        editing_template.get().map(|template| {
            let t_id = template.id.clone();
            let t_name = template.name.clone();
            let t_desc = template.description.clone().unwrap_or_default();
            let t_content = template.content.clone();
            let t_cat = template.category.clone().unwrap_or_default();
            let t_tags = template.tags.join(", ");
            let save_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_editing_template.set(None);
            });
            let cancel_cb = Callback::new(move |_| set_editing_template.set(None));
            view! {
                <CreateEditModal
                    is_edit=true
                    edit_template_id={Some(t_id)}
                    initial_name={t_name}
                    initial_description={t_desc}
                    initial_content={t_content}
                    initial_category={t_cat}
                    initial_tags={t_tags}
                    on_save={save_cb}
                    on_cancel={cancel_cb}
                />
            }
        })
    };

    // Create modal view
    let create_modal_view = move || {
        show_create_modal.get().then(|| {
            let save_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_show_create_modal.set(false);
            });
            let cancel_cb = Callback::new(move |_| set_show_create_modal.set(false));
            view! {
                <CreateEditModal
                    is_edit=false
                    edit_template_id={None}
                    initial_name={String::new()}
                    initial_description={String::new()}
                    initial_content={String::new()}
                    initial_category={String::new()}
                    initial_tags={String::new()}
                    on_save={save_cb}
                    on_cancel={cancel_cb}
                />
            }
        })
    };

    // Preview modal view
    let preview_modal_view = move || {
        preview_template.get().map(|template| {
            let close_cb = Callback::new(move |_| set_preview_template.set(None));
            view! {
                <PreviewModal template=template on_close={close_cb} />
            }
        })
    };

    // Delete modal view
    let delete_modal_view = move || {
        deleting_id.get().map(|id| {
            let confirm_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_deleting_id.set(None);
            });
            let cancel_cb = Callback::new(move |_| set_deleting_id.set(None));
            view! {
                <DeleteConfirmModal
                    id=id
                    on_confirm={confirm_cb}
                    on_cancel={cancel_cb}
                />
            }
        })
    };

    view! {
        <div>
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Templates"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Manage document templates to standardize your workflow"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors 
                           flex items-center gap-2"
                    on:click={move |_| set_show_create_modal.set(true)}
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 4v16m8-8H4" />
                    </svg>
                    "New Template"
                </button>
            </div>

            // Content: sidebar + grid
            <div class="flex gap-6">
                <div class="w-52 flex-shrink-0">
                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3">
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2 px-2">
                            "Categories"
                        </h3>
                        <Suspense fallback={view! { <div class="px-2 text-sm text-gray-400">"Loading..."</div> }}>
                            {categories_view}
                        </Suspense>
                    </div>
                </div>

                <div class="flex-1">
                    <Suspense fallback={skeleton_grid}>
                        {templates_view}
                    </Suspense>
                </div>
            </div>

            // Modals
            {create_modal_view}
            {edit_modal_view}
            {preview_modal_view}
            {delete_modal_view}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Template Grid Card
// ---------------------------------------------------------------------------

#[component]
fn TemplateGridCard(
    template: DocumentTemplate,
    on_preview: Callback<()>,
    on_edit: Callback<()>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let category = template.category.clone().unwrap_or_default();
    let tag_count = template.tags.len();
    let updated = template.updated_at.split('T').next().unwrap_or("").to_string();

    let desc_view = if let Some(desc) = &template.description {
        let desc = desc.clone();
        view! {
            <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">{desc}</p>
        }.into_any()
    } else {
        view! { <div class="mt-1"></div> }.into_any()
    };

    let category_badge = if !category.is_empty() {
        let cat = category.clone();
        view! {
            <span class="inline-block px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900/50 
                         text-blue-700 dark:text-blue-300 rounded-full mb-2">{cat}</span>
        }.into_any()
    } else {
        view! { <div class="mb-2"></div> }.into_any()
    };

    let tags_view = if tag_count > 0 {
        let tags = template.tags.clone();
        view! {
            <div class="mt-3 flex flex-wrap gap-1">
                {tags.into_iter().map(|tag| {
                    view! {
                        <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 
                                     text-gray-600 dark:text-gray-300 rounded">{tag}</span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }.into_any()
    } else {
        view! { <div></div> }.into_any()
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 
                    hover:shadow-md transition-shadow group">
            <div class="p-4">
                {category_badge}
                <h3 class="font-semibold text-gray-900 dark:text-white truncate" title={template.name.clone()}>
                    {template.name.clone()}
                </h3>
                {desc_view}
                {tags_view}
                <p class="mt-3 text-xs text-gray-400 dark:text-gray-500">
                    {format!("Updated {}", updated)}
                </p>
            </div>

            <div class="px-4 pb-3 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                    class="flex-1 px-3 py-1.5 text-xs font-medium text-blue-600 dark:text-blue-400 
                           bg-blue-50 dark:bg-blue-900/30 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/50"
                    on:click={move |_| on_preview.run(())}
                >
                    "Preview"
                </button>
                <button
                    class="flex-1 px-3 py-1.5 text-xs font-medium text-gray-600 dark:text-gray-400 
                           bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600"
                    on:click={move |_| on_edit.run(())}
                >
                    "Edit"
                </button>
                <button
                    class="px-3 py-1.5 text-xs font-medium text-red-600 dark:text-red-400 
                           bg-red-50 dark:bg-red-900/30 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/50"
                    on:click={move |_| on_delete.run(())}
                >
                    "Delete"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Create / Edit Modal
// ---------------------------------------------------------------------------

#[component]
fn CreateEditModal(
    is_edit: bool,
    edit_template_id: Option<String>,
    initial_name: String,
    initial_description: String,
    initial_content: String,
    initial_category: String,
    initial_tags: String,
    on_save: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(initial_name);
    let (description, set_description) = signal(initial_description);
    let (content, set_content) = signal(initial_content);
    let (category, set_category) = signal(initial_category);
    let (tags, set_tags) = signal(initial_tags);
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let title = if is_edit { "Edit Template" } else { "Create Template" };

    let handle_submit = move |_| {
        let n = name.get();
        let c = content.get();
        if n.trim().is_empty() {
            set_error.set(Some("Name is required".to_string()));
            return;
        }
        if c.trim().is_empty() {
            set_error.set(Some("Content is required".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);

        let api = ApiClient::default();
        let tags_str = tags.get();
        let parsed_tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let cat_val = category.get();
        let desc_val = description.get();
        let tid = edit_template_id.clone();
        let save_cb = on_save;

        spawn_local(async move {
            let result = if let Some(template_id) = tid {
                api.update_template(&template_id, &UpdateTemplateRequest {
                    name: Some(n),
                    description: if desc_val.is_empty() { None } else { Some(desc_val) },
                    content: Some(c),
                    category: if cat_val.is_empty() { None } else { Some(cat_val) },
                    tags: if parsed_tags.is_empty() { None } else { Some(parsed_tags) },
                }).await
            } else {
                api.create_template(&CreateTemplateRequest {
                    name: n,
                    description: if desc_val.is_empty() { None } else { Some(desc_val) },
                    content: c,
                    category: if cat_val.is_empty() { None } else { Some(cat_val) },
                    tags: if parsed_tags.is_empty() { None } else { Some(parsed_tags) },
                }).await
            };
            set_submitting.set(false);
            match result {
                Ok(_) => save_cb.run(()),
                Err(e) => set_error.set(Some(format!("Failed: {}", e))),
            }
        });
    };

    // Error view (extracted before main view!)
    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 
                            text-red-700 dark:text-red-300 text-sm rounded-lg">
                    {e}
                </div>
            }
        })
    };

    let btn_label = move || if submitting.get() { "Saving..." } else { if is_edit { "Update" } else { "Create" } };
    let btn_class = move || format!(
        "px-4 py-2 text-sm text-white rounded-lg {}",
        if submitting.get() { "bg-blue-400 cursor-not-allowed" } else { "bg-blue-600 hover:bg-blue-700" }
    );

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_cancel.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h2>
                    <button
                        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg 
                               hover:bg-gray-100 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div class="px-6 py-4 space-y-4 overflow-y-auto max-h-[60vh]">
                    {error_view}

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Name" <span class="text-red-500">"*"</span>
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            placeholder="e.g. Meeting Notes"
                            prop:value={name.get()}
                            on:input={move |ev| { set_name.set(event_target_value(&ev)); }}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Description"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            placeholder="Brief description of this template"
                            prop:value={description.get()}
                            on:input={move |ev| { set_description.set(event_target_value(&ev)); }}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Category"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            placeholder="e.g. Engineering, Product, Research"
                            prop:value={category.get()}
                            on:input={move |ev| { set_category.set(event_target_value(&ev)); }}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Tags"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            placeholder="Comma-separated: sprint, retro, planning"
                            prop:value={tags.get()}
                            on:input={move |ev| { set_tags.set(event_target_value(&ev)); }}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Template Content" <span class="text-red-500">"*"</span>
                        </label>
                        <textarea
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none 
                                   font-mono text-sm"
                            rows="12"
                            placeholder="Template content (markdown supported)..."
                            prop:value={content.get()}
                            on:input={move |ev| { set_content.set(event_target_value(&ev)); }}
                        ></textarea>
                        <p class="mt-1 text-xs text-gray-400">
                            "Markdown is supported. Use this as the starting content for new documents."
                        </p>
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
                    <button
                        class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300 
                               dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}
                    >
                        "Cancel"
                    </button>
                    <button
                        class={btn_class}
                        disabled={submitting.get()}
                        on:click={handle_submit}
                    >
                        {btn_label}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Preview Modal
// ---------------------------------------------------------------------------

#[component]
fn PreviewModal(template: DocumentTemplate, on_close: Callback<()>) -> impl IntoView {
    let t_name = template.name.clone();
    let t_content = template.content.clone();
    let t_category = template.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
    let t_updated = template.updated_at.split('T').next().unwrap_or("").to_string();
    let t_tags = template.tags.clone();

    let tags_view = if !t_tags.is_empty() {
        let tags = t_tags.clone();
        view! {
            <div class="px-6 pt-4 flex flex-wrap gap-1">
                {tags.into_iter().map(|tag| {
                    view! {
                        <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 
                                     text-gray-600 dark:text-gray-300 rounded">{tag}</span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }.into_any()
    } else {
        view! { <div></div> }.into_any()
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_close.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-3xl max-h-[85vh] overflow-hidden"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <div>
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{t_name}</h2>
                        <div class="flex items-center gap-3 mt-1">
                            <span class="text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 px-2 py-0.5 rounded-full">
                                {t_category}
                            </span>
                            <span class="text-xs text-gray-400">
                                {format!("Updated {}", t_updated)}
                            </span>
                        </div>
                    </div>
                    <button
                        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg 
                               hover:bg-gray-100 dark:hover:bg-gray-700"
                        on:click={move |_| on_close.run(())}
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                {tags_view}

                <div class="px-6 py-4 overflow-y-auto max-h-[55vh]">
                    <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap font-mono bg-gray-50 
                                dark:bg-gray-900/50 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
                        {t_content}
                    </pre>
                </div>

                <div class="flex justify-end px-6 py-4 border-t border-gray-200 dark:border-gray-700">
                    <button
                        class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300 
                               dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click={move |_| on_close.run(())}
                    >
                        "Close"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Delete Confirmation Modal
// ---------------------------------------------------------------------------

#[component]
fn DeleteConfirmModal(id: String, on_confirm: Callback<()>, on_cancel: Callback<()>) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let handle_delete = move |_| {
        set_error.set(None);
        set_submitting.set(true);
        let api = ApiClient::default();
        let template_id = id.clone();
        let confirm_cb = on_confirm;
        spawn_local(async move {
            match api.delete_template(&template_id).await {
                Ok(_) => confirm_cb.run(()),
                Err(e) => {
                    set_error.set(Some(format!("Failed to delete: {}", e)));
                    set_submitting.set(false);
                }
            }
        });
    };

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 
                            dark:border-red-800 text-red-700 dark:text-red-300 text-sm rounded-lg">
                    {e}
                </div>
            }
        })
    };

    let btn_label = move || if submitting.get() { "Deleting..." } else { "Delete" };
    let btn_class = move || format!(
        "px-4 py-2 text-sm text-white rounded-lg {}",
        if submitting.get() { "bg-red-400 cursor-not-allowed" } else { "bg-red-600 hover:bg-red-700" }
    );

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_cancel.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="p-6">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="flex-shrink-0 w-10 h-10 bg-red-100 dark:bg-red-900/30 rounded-full 
                                    flex items-center justify-center">
                            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Delete Template"</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                "Are you sure? This action cannot be undone."
                            </p>
                        </div>
                    </div>

                    {error_view}

                    <div class="flex justify-end gap-3 mt-6">
                        <button
                            class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300 
                                   dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                            on:click={move |_| on_cancel.run(())}
                        >
                            "Cancel"
                        </button>
                        <button
                            class={btn_class}
                            disabled={submitting.get()}
                            on:click={handle_delete}
                        >
                            {btn_label}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Skeleton loading state
// ---------------------------------------------------------------------------

#[component]
fn TemplateCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/3 mb-3"></div>
            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-full mb-2"></div>
            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mb-4"></div>
            <div class="flex gap-2">
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
            </div>
        </div>
    }
}
