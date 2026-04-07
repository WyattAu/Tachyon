// Admin Roles Management Page
// Role management interface for administrators

use leptos::prelude::*;
use serde::Deserialize;
use leptos::task::spawn_local;
use crate::api::ApiClient;

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system: bool,
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

async fn fetch_roles() -> Result<Vec<Role>, String> {
    let client = ApiClient::default();
    let raw = client.list_roles().await.map_err(|e| e.to_string())?;
    serde_json::from_value(serde_json::Value::Array(raw)).map_err(|e| e.to_string())
}

async fn create_role_req(req: CreateRoleRequest) -> Result<Role, String> {
    let client = ApiClient::default();
    let body = serde_json::to_value(&req).map_err(|e| e.to_string())?;
    let raw = client.create_role(&body).await.map_err(|e| e.to_string())?;
    serde_json::from_value(raw).map_err(|e| e.to_string())
}

async fn delete_role_req(id: i64) -> Result<(), String> {
    let client = ApiClient::default();
    client.delete_role(&id.to_string()).await.map_err(|e| e.to_string())
}

#[component]
pub fn RolesPage() -> impl IntoView {
    let (roles, set_roles) = signal(Vec::<Role>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_role_name, set_new_role_name) = signal(String::new());
    let (new_role_description, set_new_role_description) = signal(String::new());
    let (selected_permissions, set_selected_permissions) = signal(Vec::<String>::new());

    let available_permissions = vec!["read", "write", "delete", "admin"];

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_roles().await {
                Ok(r) => {
                    set_roles.set(r);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    let toggle_permission = move |perm: String| {
        let mut perms = selected_permissions.get();
        if perms.contains(&perm) {
            perms.retain(|p| p != &perm);
        } else {
            perms.push(perm);
        }
        set_selected_permissions.set(perms);
    };

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let name = new_role_name.get();
        let description = new_role_description.get();
        let permissions = selected_permissions.get();

        if name.is_empty() {
            set_error.set(Some("Role name is required".to_string()));
            return;
        }

        spawn_local(async move {
            let req = CreateRoleRequest {
                name,
                description: if description.is_empty() { None } else { Some(description) },
                permissions,
            };
            match create_role_req(req).await {
                Ok(role) => {
                    let mut current_roles = roles.get();
                    current_roles.push(role);
                    set_roles.set(current_roles);
                    set_show_create_modal.set(false);
                    set_new_role_name.set(String::new());
                    set_new_role_description.set(String::new());
                    set_selected_permissions.set(Vec::new());
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    let handle_delete = move |id: i64| {
        spawn_local(async move {
            match delete_role_req(id).await {
                Ok(()) => {
                    let current_roles = roles.get();
                    let updated: Vec<Role> = current_roles.into_iter().filter(|r| r.id != id).collect();
                    set_roles.set(updated);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="p-6 max-w-6xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Role Management"</h1>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">"Manage roles and permissions for your team"</p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "Create Role"
                </button>
            </div>

            {move || error.get().map(|e| view! {
                <div class="mb-4 p-4 bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200 rounded-lg">
                    {e}
                </div>
            })}

            {move || if loading.get() {
                Some(view! {
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                })
            } else {
                None
            }}

            <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-900">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Name"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Description"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Permissions"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Type"</th>
                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                        {move || roles.get().into_iter().map(|role| view! {
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="text-sm font-medium text-gray-900 dark:text-white">
                                        {role.name.clone()}
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="text-sm text-gray-500 dark:text-gray-400">
                                        {role.description.clone().unwrap_or_else(|| "-".to_string())}
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="flex flex-wrap gap-1">
                                        {role.permissions.clone().into_iter().map(|perm| view! {
                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                                {perm}
                                            </span>
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    {if role.is_system {
                                        view! {
                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200">
                                                "System"
                                            </span>
                                        }
                                    } else {
                                        view! {
                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200">
                                                "Custom"
                                            </span>
                                        }
                                    }}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                    {if !role.is_system {
                                        view! {
                                            <button
                                                class="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300"
                                                on:click=move |_| handle_delete(role.id)
                                            >
                                                "Delete"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span class="text-gray-400">"-"</span> }.into_any()
                                    }}
                                </td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>

            {move || if show_create_modal.get() {
                Some(view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
                            <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">"Create New Role"</h2>

                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Name"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        placeholder="Enter role name"
                                        on:input=move |ev| set_new_role_name.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Description"
                                    </label>
                                    <textarea
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        rows="3"
                                        placeholder="Enter role description"
                                        on:input=move |ev| set_new_role_description.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        "Permissions"
                                    </label>
                                    <div class="space-y-2">
                                        {available_permissions.clone().into_iter().map(|perm| view! {
                                            <label class="flex items-center">
                                                <input
                                                    type="checkbox"
                                                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                    on:change=move |_| toggle_permission(perm.to_string())
                                                />
                                                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300 capitalize">
                                                    {perm.to_string()}
                                                </span>
                                            </label>
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            </div>

                            <div class="mt-6 flex justify-end gap-3">
                                <button
                                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
                                    on:click=move |_| set_show_create_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
                                    on:click=handle_create
                                >
                                    "Create Role"
                                </button>
                            </div>
                        </div>
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}
