#![allow(dead_code)]

use crate::api::ApiClient;
use crate::components::FocusTrap;
use crate::types::DocumentTemplate;
use leptos::prelude::*;

#[component]
pub fn TemplateSelector(
    on_select: Callback<DocumentTemplate>,
    category: Option<String>,
) -> impl IntoView {
    let api_client = ApiClient::default();
    let api_client_for_categories = api_client.clone();
    let (selected_category, set_selected_category) = signal(category);
    let (preview_template, set_preview_template) = signal(None::<DocumentTemplate>);

    let templates_resource = LocalResource::new(move || {
        let client = api_client.clone();
        let cat = selected_category.get();
        async move {
            client
                .list_templates(cat.as_deref())
                .await
                .unwrap_or_default()
        }
    });

    let categories_resource = LocalResource::new(move || {
        let client = api_client_for_categories.clone();
        async move { client.list_template_categories().await.unwrap_or_default() }
    });

    let on_select_for_grid = on_select;
    let on_select_for_modal = on_select;

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Templates"</h3>
            </div>

            <div class="flex">
                <div class="w-48 border-r border-gray-200 dark:border-gray-700 p-4">
                    <Suspense fallback={view! { <div>"Loading..."</div> }}>
                        {move || {
                            categories_resource.get().map(|categories| {
                                view! {
                                    <ul class="space-y-1">
                                        <li>
                                            <button
                                                class={move || format!(
                                                    "w-full text-left px-3 py-2 rounded-lg transition-colors {}",
                                                    if selected_category.get().is_none() {
                                                        "bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300"
                                                    } else {
                                                        "hover:bg-gray-100 dark:hover:bg-gray-700"
                                                    }
                                                )}
                                                on:click={move |_| set_selected_category.set(None)}
                                            >
                                                "All Templates"
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
                                                            "w-full text-left px-3 py-2 rounded-lg transition-colors {}",
                                                            if selected_category.get() == Some(cat_for_class.clone()) {
                                                                "bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300"
                                                            } else {
                                                                "hover:bg-gray-100 dark:hover:bg-gray-700"
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
                        }}
                    </Suspense>
                </div>

                <div class="flex-1 p-4">
                    <Suspense fallback={view! { <div>"Loading templates..."</div> }}>
                        {move || {
                            templates_resource.get().map(|templates| {
                                if templates.is_empty() {
                                    view! {
                                        <div class="text-center text-gray-500 dark:text-gray-400 py-8">
                                            "No templates available"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            {templates.into_iter().map(|template| {
                                                let tmpl_for_click = template.clone();
                                                let tmpl_for_use = template.clone();
                                                let on_select = on_select_for_grid;
                                                view! {
                                                    <div
                                                        class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg
                                                               hover:border-blue-500 cursor-pointer transition-colors"
                                                        on:click={move |_| set_preview_template.set(Some(tmpl_for_click.clone()))}
                                                    >
                                                        <h4 class="font-medium text-gray-900 dark:text-white">
                                                            {template.name}
                                                        </h4>
                                                        {if let Some(desc) = &template.description {
                                                            let desc = desc.clone();
                                                            view! {
                                                                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                                                                    {desc}
                                                                </p>
                                                            }.into_any()
                                                        } else {
                                                            view! { <div></div> }.into_any()
                                                        }}
                                                        <div class="mt-2 flex flex-wrap gap-1">
                                                            {template.tags.into_iter().map(|tag| {
                                                                view! {
                                                                    <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700
                                                                                 text-gray-600 dark:text-gray-300 rounded">
                                                                        {tag}
                                                                    </span>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                        <button
                                                            class="mt-3 px-3 py-1 text-sm bg-blue-600 text-white rounded
                                                                   hover:bg-blue-700 transition-colors"
                                                            on:click={move |ev| {
                                                                ev.stop_propagation();
                                                                on_select.run(tmpl_for_use.clone());
                                                            }}
                                                        >
                                                            "Use Template"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            {move || {
                preview_template.get().map(|template| {
                    let template_for_name = template.name.clone();
                    let template_for_content = template.content.clone();
                    let template_for_use = template.clone();
                    let on_select = on_select_for_modal;
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                            <FocusTrap active=Signal::derive(move || preview_template.get().is_some())>
                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-hidden" role="dialog" attr:aria-modal="true" attr:aria-label="Template preview">
                                    <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
                                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                            {template_for_name}
                                        </h3>
                                        <button
                                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                                            attr:aria-label="Close"
                                            on:click={move |_| set_preview_template.set(None)}
                                        >
                                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                            </svg>
                                        </button>
                                    </div>
                                    <div class="p-4 overflow-auto max-h-[60vh]">
                                        <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap font-mono">
                                            {template_for_content}
                                        </pre>
                                    </div>
                                    <div class="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2">
                                        <button
                                            class="px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600
                                                   rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                                            on:click={move |_| set_preview_template.set(None)}
                                        >
                                            "Cancel"
                                        </button>
                                        <button
                                            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                                            on:click={move |_| {
                                                on_select.run(template_for_use.clone());
                                                set_preview_template.set(None);
                                            }}
                                        >
                                            "Use Template"
                                        </button>
                                    </div>
                                </div>
                            </FocusTrap>
                        </div>
                    }
                })
            }}
        </div>
    }
}

#[component]
pub fn TemplateCard(
    template: DocumentTemplate,
    on_select: Callback<DocumentTemplate>,
) -> impl IntoView {
    let template_for_click = template.clone();

    view! {
        <div
            class="p-4 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700
                   hover:border-blue-500 cursor-pointer transition-colors"
            on:click={move |_| on_select.run(template_for_click.clone())}
        >
            <h4 class="font-medium text-gray-900 dark:text-white">{template.name}</h4>
            {if let Some(desc) = &template.description {
                let desc = desc.clone();
                view! {
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{desc}</p>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
            {if let Some(cat) = &template.category {
                let cat = cat.clone();
                view! {
                    <span class="mt-2 inline-block px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900
                                 text-blue-600 dark:text-blue-300 rounded">
                        {cat}
                    </span>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
