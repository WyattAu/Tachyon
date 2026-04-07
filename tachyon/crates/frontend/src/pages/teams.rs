// Teams Page
// Team list and management interface

use leptos::prelude::*;
use serde::Deserialize;
use leptos::task::spawn_local;
use crate::api::ApiClient;

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[allow(dead_code)]
    pub owner_id: String,
    #[allow(dead_code)]
    pub avatar_url: Option<String>,
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String,
    pub member_count: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamMember {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub team_id: String,
    pub user_id: String,
    #[allow(dead_code)]
    pub role_id: i64,
    pub role_name: String,
    #[allow(dead_code)]
    pub joined_at: String,
    #[allow(dead_code)]
    pub invited_by: Option<String>,
}

async fn fetch_teams() -> Result<Vec<Team>, String> {
    let client = ApiClient::default();
    let raw = client.list_teams().await.map_err(|e| e.to_string())?;
    // Deserialize from serde_json::Value
    serde_json::from_value(serde_json::Value::Array(raw)).map_err(|e| e.to_string())
}

async fn fetch_team_members(team_id: &str) -> Result<Vec<TeamMember>, String> {
    let client = ApiClient::default();
    let raw = client.list_team_members(team_id).await.map_err(|e| e.to_string())?;
    serde_json::from_value(serde_json::Value::Array(raw)).map_err(|e| e.to_string())
}

async fn create_team_req(name: String, slug: String, description: Option<String>) -> Result<Team, String> {
    let client = ApiClient::default();
    let body = serde_json::json!({
        "name": name,
        "slug": slug,
        "description": description,
    });
    let raw = client.create_team(&body).await.map_err(|e| e.to_string())?;
    serde_json::from_value(raw).map_err(|e| e.to_string())
}

#[component]
pub fn TeamsPage() -> impl IntoView {
    let (teams, set_teams) = signal(Vec::<Team>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_team_name, set_new_team_name) = signal(String::new());
    let (new_team_slug, set_new_team_slug) = signal(String::new());
    let (new_team_description, set_new_team_description) = signal(String::new());

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_teams().await {
                Ok(t) => {
                    set_teams.set(t);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let name = new_team_name.get();
        let slug = new_team_slug.get();
        let description = new_team_description.get();

        if name.is_empty() || slug.is_empty() {
            set_error.set(Some("Name and slug are required".to_string()));
            return;
        }

        spawn_local(async move {
            let desc = if description.is_empty() { None } else { Some(description) };
            match create_team_req(name, slug, desc).await {
                Ok(team) => {
                    let mut current_teams = teams.get();
                    current_teams.push(team);
                    set_teams.set(current_teams);
                    set_show_create_modal.set(false);
                    set_new_team_name.set(String::new());
                    set_new_team_slug.set(String::new());
                    set_new_team_description.set(String::new());
                    set_error.set(None);
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
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Teams"</h1>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">"Manage your teams and team members"</p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "Create Team"
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

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {move || teams.get().into_iter().map(|team| view! {
                    <TeamCard team=team />
                }).collect::<Vec<_>>()}
            </div>

            {move || if teams.get().is_empty() && !loading.get() {
                Some(view! {
                    <div class="text-center py-12">
                        <div class="text-gray-500 dark:text-gray-400">
                            <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                            <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">"No teams"</h3>
                            <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Get started by creating a new team."</p>
                        </div>
                    </div>
                })
            } else {
                None
            }}

            {move || if show_create_modal.get() {
                Some(view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
                            <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">"Create New Team"</h2>

                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Name"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        placeholder="Team name"
                                        on:input=move |ev| set_new_team_name.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Slug"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        placeholder="team-slug"
                                        on:input=move |ev| set_new_team_slug.set(event_target_value(&ev))
                                    />
                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">"URL-friendly identifier (lowercase, hyphens only)"</p>
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Description"
                                    </label>
                                    <textarea
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        rows="3"
                                        placeholder="Brief description of the team"
                                        on:input=move |ev| set_new_team_description.set(event_target_value(&ev))
                                    />
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
                                    "Create Team"
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

#[component]
pub fn TeamCard(team: Team) -> impl IntoView {
    let _team_id = team.id.clone();
    let team_name = team.name.clone();
    let team_name_initial = team.name.clone();
    let team_slug = team.slug.clone();
    let team_desc = team.description.clone();
    let team_member_count = team.member_count;
    let (expanded, set_expanded) = signal(false);
    let (members, set_members) = signal(Vec::<TeamMember>::new());
    let (loading_members, set_loading_members) = signal(false);

    let toggle_expand = move |_| {
        let current = expanded.get();
        if !current && members.get().is_empty() {
            let team_id = team.id.clone();
            spawn_local(async move {
                set_loading_members.set(true);
                match fetch_team_members(&team_id).await {
                    Ok(m) => set_members.set(m),
                    Err(_) => {}
                }
                set_loading_members.set(false);
            });
        }
        set_expanded.set(!current);
    };

    view! {
        <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
            <div
                class="p-4 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                on:click=toggle_expand
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
                            <span class="text-blue-600 dark:text-blue-300 font-semibold">
                                {move || team_name_initial.chars().next().unwrap_or('?').to_uppercase().to_string()}
                            </span>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                {team_name.clone()}
                            </h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {team_slug.clone()}
                            </p>
                        </div>
                    </div>
                    <div class="flex items-center gap-2">
                        {move || team_member_count.map(|count| view! {
                            <span class="text-sm text-gray-500 dark:text-gray-400">
                                {count}" members"
                            </span>
                        })}
                        <svg
                            class=format!("w-5 h-5 text-gray-400 transition-transform {}", if expanded.get() { "rotate-180" } else { "" })
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </div>
                </div>

                {move || team_desc.clone().map(|desc| view! {
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        {desc}
                    </p>
                })}
            </div>

            {move || if expanded.get() {
                Some(view! {
                    <div class="border-t border-gray-200 dark:border-gray-700 p-4">
                        {move || if loading_members.get() {
                            view! {
                                <div class="flex justify-center py-4">
                                    <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
                                </div>
                            }.into_any()
                        } else if members.get().is_empty() {
                            view! {
                                <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
                                    "No members yet"
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {members.get().into_iter().map(|member| view! {
                                        <div class="flex items-center justify-between py-2">
                                            <div class="flex items-center gap-2">
                                                <div class="w-8 h-8 bg-gray-200 dark:bg-gray-600 rounded-full"></div>
                                                <div>
                                                    <p class="text-sm font-medium text-gray-900 dark:text-white">
                                                        {member.user_id.clone()}
                                                    </p>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400">
                                                        {member.role_name.clone()}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}

#[component]
pub fn TeamDetailPage() -> impl IntoView {
    let (team, set_team) = signal(None::<Team>);
    let (members, set_members) = signal(Vec::<TeamMember>::new());
    let (loading, set_loading) = signal(true);
    let (_error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            // TODO: Get team ID from route params
            match fetch_teams().await {
                Ok(teams) => {
                    if let Some(first_team) = teams.into_iter().next() {
                        let team_id = first_team.id.clone();
                        set_team.set(Some(first_team));

                        match fetch_team_members(&team_id).await {
                            Ok(m) => set_members.set(m),
                            Err(e) => set_error.set(Some(e)),
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="p-6 max-w-4xl mx-auto">
            {move || if loading.get() {
                Some(view! {
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                }.into_any())
            } else if let Some(t) = team.get() {
                Some(view! {
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                            {t.name.clone()}
                        </h1>
                        {t.description.clone().map(|desc| view! {
                            <p class="mt-2 text-gray-600 dark:text-gray-400">
                                {desc}
                            </p>
                        })}

                        <div class="mt-6">
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                "Members"
                            </h2>
                            <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
                                <ul class="divide-y divide-gray-200 dark:divide-gray-700">
                                    {members.get().into_iter().map(|member| view! {
                                        <li class="px-4 py-3 flex items-center justify-between">
                                            <div class="flex items-center gap-3">
                                                <div class="w-10 h-10 bg-gray-200 dark:bg-gray-600 rounded-full"></div>
                                                <div>
                                                    <p class="font-medium text-gray-900 dark:text-white">
                                                        {member.user_id.clone()}
                                                    </p>
                                                    <p class="text-sm text-gray-500 dark:text-gray-400">
                                                        "Joined " {member.joined_at.clone()}
                                                    </p>
                                                </div>
                                            </div>
                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                                {member.role_name.clone()}
                                            </span>
                                        </li>
                                    }).collect::<Vec<_>>()}
                                </ul>
                            </div>
                        </div>
                    </div>
                }.into_any())
            } else {
                Some(view! {
                    <div class="text-center py-12">
                        <p class="text-gray-500 dark:text-gray-400">"Team not found"</p>
                    </div>
                }.into_any())
            }}
        </div>
    }
}
