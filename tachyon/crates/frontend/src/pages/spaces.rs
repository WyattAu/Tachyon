// Spaces Page
// Space management - tree navigation, create/edit/delete, member management

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;
use crate::types::{Space, CreateSpaceRequest, UpdateSpaceRequest, SpaceMember, AddSpaceMemberRequest, UpdateSpaceMemberRequest};

// ---------------------------------------------------------------------------
// Spaces Page
// ---------------------------------------------------------------------------

#[component]
pub fn SpacesPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let api_for_spaces = api_client.clone();

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (editing_space, set_editing_space) = signal(None::<Space>);
    let (deleting_id, set_deleting_id) = signal(None::<String>);
    let (managing_members_id, set_managing_members_id) = signal(None::<String>);
    let (expanded_set, set_expanded_set) = signal(Vec::<String>::new());
    let (refresh_counter, set_refresh_counter) = signal(0u32);

    // Fetch spaces
    let spaces_resource = LocalResource::new(move || {
        let client = api_for_spaces.clone();
        let _rc = refresh_counter.get();
        async move {
            client.list_spaces(None).await.unwrap_or_default()
        }
    });

    // Toggle expansion
    let toggle_expand = move |sid: String| {
        set_expanded_set.update(|set| {
            if let Some(idx) = set.iter().position(|id| *id == sid) {
                set.remove(idx);
            } else {
                set.push(sid);
            }
        });
    };

    // Refresh
    let refresh = move || set_refresh_counter.update(|c| { *c += 1; });

    // Create modal close/saved callbacks
    let close_create = Callback::new(move |_| set_show_create_modal.set(false));
    let saved_create = Callback::new(move |_| { set_show_create_modal.set(false); refresh(); });

    // Edit modal close/saved callbacks
    let close_edit = Callback::new(move |_| set_editing_space.set(None));
    let saved_edit = Callback::new(move |_| { set_editing_space.set(None); refresh(); });

    // Delete confirm/cancel callbacks
    let cancel_delete = Callback::new(move |_| set_deleting_id.set(None));

    // Members close callback
    let close_members = Callback::new(move |_| set_managing_members_id.set(None));

    view! {
        <div class="max-w-4xl mx-auto p-6">
            // Header
            <div class="mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Spaces"</h1>
                <p class="text-gray-500 dark:text-gray-400 mt-1">
                    "Organize your documents into spaces"
                </p>
            </div>

            // Spaces list
            <Suspense fallback=move || view! { <div class="text-center py-16 text-gray-400">"Loading..."</div> }>
                {move || spaces_resource.get().map(|spaces| {
                    if spaces.is_empty() {
                        view! {
                            <div class="text-center py-16">
                                <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4"
                                     fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                                          d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                                </svg>
                                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">
                                    "No spaces yet"
                                </h3>
                                <p class="text-gray-500 dark:text-gray-400 mb-4">
                                    "Create a space to organize your documents"
                                </p>
                                <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                                    on:click={move |_| set_show_create_modal.set(true)}>
                                    "Create Space"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        let count = spaces.len();
                        let root: Vec<Space> = spaces.iter().filter(|s| s.parent_id.is_none()).cloned().collect();
                        view! {
                            <div>
                                <div class="flex items-center justify-between mb-4">
                                    <p class="text-sm text-gray-500 dark:text-gray-400">
                                        {move || format!("{} space{}", count, if count == 1 { "" } else { "s" })}
                                    </p>
                                    <button class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700"
                                        on:click={move |_| set_show_create_modal.set(true)}>
                                        "+ New Space"
                                    </button>
                                </div>
                                <div class="space-y-1">
                                    {
                                        root.into_iter().map(|space| {
                                            let sp = space.clone();
                                            let sp_edit = space.clone();
                                            let sp_del = space.clone();
                                            let sp_members = space.clone();
                                            let sp_toggle = space.id.clone();
                                            let sp_id = space.id.clone();
                                            let is_expanded = expanded_set.get().contains(&sp_id);
                                            let on_toggle = Callback::new(move |_| toggle_expand(sp_toggle.clone()));
                                            let on_edit = Callback::new(move |_| set_editing_space.set(Some(sp_edit.clone())));
                                            let on_delete = Callback::new(move |_| set_deleting_id.set(Some(sp_del.id.clone())));
                                            let on_members = Callback::new(move |_| set_managing_members_id.set(Some(sp_members.id.clone())));
                                            view! {
                                                <SpaceItem space=sp expanded=is_expanded
                                                    on_toggle=on_toggle on_edit=on_edit
                                                    on_delete=on_delete on_members=on_members>
                                                </SpaceItem>
                                            }
                                        }).collect::<Vec<_>>()
                                    }
                                </div>
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>

            // Create modal
            <Show when=move || show_create_modal.get()>
                <CreateSpaceModal on_close=close_create on_created=saved_create />
            </Show>

            // Edit modal
            <Show when=move || editing_space.get().is_some()>
                {move || editing_space.get().map(|space| {
                    view! {
                        <EditSpaceModal space=space on_close=close_edit on_saved=saved_edit />
                    }
                })}
            </Show>

            // Delete confirmation
            <Show when=move || deleting_id.get().is_some()>
                {move || deleting_id.get().map(|id| {
                    let on_confirm = Callback::new(move |_| {
                        let api = ApiClient::default();
                        let del_id = id.clone();
                        spawn_local(async move {
                            let _ = api.delete_space(&del_id).await;
                            set_deleting_id.set(None);
                            refresh();
                        });
                    });
                    view! {
                        <ConfirmModal title="Delete Space".to_string()
                            message="Are you sure? Documents in this space will become unorganized.".to_string()
                            confirm_label="Delete".to_string()
                            on_confirm=on_confirm on_cancel=cancel_delete />
                    }
                })}
            </Show>

            // Members modal
            <Show when=move || managing_members_id.get().is_some()>
                {move || managing_members_id.get().map(|sid| {
                    view! {
                        <MembersModal space_id=sid on_close=close_members />
                    }
                })}
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Space Item (tree row)
// ---------------------------------------------------------------------------

#[component]
fn SpaceItem(
    space: Space,
    expanded: bool,
    on_toggle: Callback<()>,
    on_edit: Callback<()>,
    on_delete: Callback<()>,
    on_members: Callback<()>,
) -> impl IntoView {
    let space_name = space.name.clone();
    let space_desc = space.description.clone();
    let space_color = space.color.clone();
    let space_icon = space.icon.clone();
    let space_visibility = space.visibility.clone();
    let space_is_default = space.is_default;
    let space_doc_count = space.document_count;

    let vis_class = match space_visibility.as_str() {
        "public" => "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300",
        "team" => "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300",
        _ => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
    };

    let desc_text = match space_desc {
        Some(d) if !d.is_empty() => d,
        _ => format!("{} document{}", space_doc_count, if space_doc_count == 1 { "" } else { "s" }),
    };

    let chevron_class = if expanded { "rotate-90" } else { "" };
    let default_badge = space_is_default;

    view! {
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 group cursor-pointer">
            // Expand chevron
            <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 flex-shrink-0"
                on:click=move |_| on_toggle.run(())>
                <svg class=format!("w-4 h-4 transition-transform {}", chevron_class)
                     fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
            </button>

            // Icon
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm flex-shrink-0"
                 style=format!("background-color: {}; color: white;", space_color)>
                {space_icon}
            </div>

            // Name + badges
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span class="font-medium text-gray-900 dark:text-white truncate">{space_name}</span>
                    {move || if default_badge {
                        view! {
                            <span class="text-xs px-1.5 py-0.5 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded">
                                "Personal"
                            </span>
                        }.into_any()
                    } else {
                        view! { <span class="hidden"></span> }.into_any()
                    }}
                    <span class={vis_class.to_string()}>{space_visibility}</span>
                </div>
                <p class="text-xs text-gray-500 dark:text-gray-400 truncate">{desc_text}</p>
            </div>

            // Doc count
            <span class="text-xs text-gray-400 dark:text-gray-500 flex-shrink-0">
                {space_doc_count}
            </span>

            // Actions
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                <button class="p-1 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 rounded"
                    title="Members" on:click=move |_| on_members.run(())>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                    </svg>
                </button>
                <button class="p-1 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 rounded"
                    title="Edit" on:click=move |_| on_edit.run(())>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                </button>
                <button class="p-1 text-gray-400 hover:text-red-600 dark:hover:text-red-400 rounded"
                    title="Delete" on:click=move |_| on_delete.run(())>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Create Space Modal
// ---------------------------------------------------------------------------

#[component]
fn CreateSpaceModal(
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (desc, set_desc) = signal(String::new());
    let (icon, set_icon) = signal("folder".to_string());
    let (color, set_color) = signal("#3B82F6".to_string());
    let (visibility, set_visibility) = signal("private".to_string());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let colors = ["#3B82F6", "#6366F1", "#8B5CF6", "#EC4899", "#EF4444", "#F59E0B", "#10B981", "#6B7280"];
    let icons = ["folder", "book", "star", "heart", "briefcase", "home", "code", "lightbulb"];

    let do_submit = move |_| {
        let n = name.get();
        if n.trim().is_empty() {
            set_error.set(Some("Name is required".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);
        let api = ApiClient::default();
        let req = CreateSpaceRequest {
            name: n,
            description: if desc.get().is_empty() { None } else { Some(desc.get()) },
            icon: Some(icon.get()),
            color: Some(color.get()),
            parent_id: None,
            visibility: Some(visibility.get()),
        };
        spawn_local(async move {
            match api.create_space(&req).await {
                Ok(_) => { on_created.run(()); }
                Err(e) => { set_error.set(Some(format!("{:?}", e))); set_submitting.set(false); }
            }
        });
    };

    let do_close = move |_| on_close.run(());

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Create Space"</h2>

                <Show when=move || error.get().is_some()>
                    <div class="mb-3 p-3 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 text-sm rounded-lg">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Name"</label>
                    <input type="text"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        placeholder="e.g., Engineering Docs"
                        prop:value={move || name.get()}
                        on:input={move |ev| set_name.set(event_target_value(&ev))} />
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                    <input type="text"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        placeholder="Optional"
                        prop:value={move || desc.get()}
                        on:input={move |ev| set_desc.set(event_target_value(&ev))} />
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Icon"</label>
                    <div class="flex gap-2 flex-wrap">
                        {icons.iter().map(|ic| {
                            let ic_s = ic.to_string();
                            let sel = icon.get();
                            let active = sel == ic_s;
                            let btn_cls = if active { "ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/30" } else { "bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600" };
                            let ic_label = ic_s.clone();
                            let ic_click = ic_s.clone();
                            view! {
                                <button class={btn_cls.to_string()} on:click={move |_| set_icon.set(ic_click.clone())}>
                                    <span>{ic_label}</span>
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Color"</label>
                    <div class="flex gap-2">
                        {colors.iter().map(|c| {
                            let c_s = c.to_string();
                            let sel = color.get();
                            let active = sel == c_s;
                            let btn_cls = if active { "ring-2 ring-offset-2 ring-gray-400 dark:ring-offset-gray-800 scale-110" } else { "hover:scale-110" };
                            let c_click = c_s.clone();
                            let c_style = format!("background-color: {}", c);
                            view! {
                                <button class={btn_cls.to_string()} style={c_style} on:click={move |_| set_color.set(c_click.clone())}></button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Visibility"</label>
                    <select class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        prop:value={move || visibility.get()}
                        on:change={move |ev| set_visibility.set(event_target_value(&ev))}>
                        <option value="private">"Private"</option>
                        <option value="team">"Team"</option>
                        <option value="public">"Public"</option>
                    </select>
                </div>

                <div class="flex justify-end gap-3">
                    <button class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        on:click=do_close>"Cancel"</button>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                        disabled={move || submitting.get()} on:click=do_submit>
                        {move || if submitting.get() { "Creating..." } else { "Create Space" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Edit Space Modal
// ---------------------------------------------------------------------------

#[component]
fn EditSpaceModal(
    space: Space,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(space.name.clone());
    let (desc, set_desc) = signal(space.description.clone().unwrap_or_default());
    let (icon, set_icon) = signal(space.icon.clone());
    let (color, set_color) = signal(space.color.clone());
    let (visibility, set_visibility) = signal(space.visibility.clone());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let colors = ["#3B82F6", "#6366F1", "#8B5CF6", "#EC4899", "#EF4444", "#F59E0B", "#10B981", "#6B7280"];
    let icons = ["folder", "book", "star", "heart", "briefcase", "home", "code", "lightbulb"];

    let do_submit = move |_| {
        let n = name.get();
        if n.trim().is_empty() {
            set_error.set(Some("Name is required".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);
        let api = ApiClient::default();
        let id = space.id.clone();
        let d = desc.get();
        let desc_val = if d.is_empty() { None } else { Some(d) };
        let req = UpdateSpaceRequest {
            name: Some(n),
            description: desc_val,
            icon: Some(icon.get()),
            color: Some(color.get()),
            visibility: Some(visibility.get()),
            parent_id: None,
            sort_order: None,
        };
        spawn_local(async move {
            match api.update_space(&id, &req).await {
                Ok(_) => { on_saved.run(()); }
                Err(e) => { set_error.set(Some(format!("{:?}", e))); set_submitting.set(false); }
            }
        });
    };

    let do_close = move |_| on_close.run(());

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Edit Space"</h2>

                <Show when=move || error.get().is_some()>
                    <div class="mb-3 p-3 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 text-sm rounded-lg">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Name"</label>
                    <input type="text"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        prop:value={move || name.get()}
                        on:input={move |ev| set_name.set(event_target_value(&ev))} />
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                    <input type="text"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        prop:value={move || desc.get()}
                        on:input={move |ev| set_desc.set(event_target_value(&ev))} />
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Icon"</label>
                    <div class="flex gap-2 flex-wrap">
                        {icons.iter().map(|ic| {
                            let ic_s = ic.to_string();
                            let sel = icon.get();
                            let active = sel == ic_s;
                            let btn_cls = if active { "ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/30" } else { "bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600" };
                            let ic_label = ic_s.clone();
                            let ic_click = ic_s.clone();
                            view! {
                                <button class={btn_cls.to_string()} on:click={move |_| set_icon.set(ic_click.clone())}>
                                    <span>{ic_label}</span>
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="mb-3">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Color"</label>
                    <div class="flex gap-2">
                        {colors.iter().map(|c| {
                            let c_s = c.to_string();
                            let sel = color.get();
                            let active = sel == c_s;
                            let btn_cls = if active { "ring-2 ring-offset-2 ring-gray-400 dark:ring-offset-gray-800 scale-110" } else { "hover:scale-110" };
                            let c_click = c_s.clone();
                            let c_style = format!("background-color: {}", c);
                            view! {
                                <button class={btn_cls.to_string()} style={c_style} on:click={move |_| set_color.set(c_click.clone())}></button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Visibility"</label>
                    <select class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        prop:value={move || visibility.get()}
                        on:change={move |ev| set_visibility.set(event_target_value(&ev))}>
                        <option value="private">"Private"</option>
                        <option value="team">"Team"</option>
                        <option value="public">"Public"</option>
                    </select>
                </div>

                <div class="flex justify-end gap-3">
                    <button class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        on:click=do_close>"Cancel"</button>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                        disabled={move || submitting.get()} on:click=do_submit>
                        {move || if submitting.get() { "Saving..." } else { "Save Changes" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Members Modal
// ---------------------------------------------------------------------------

#[component]
fn MembersModal(
    space_id: String,
    on_close: Callback<()>,
) -> impl IntoView {
    let (members, set_members) = signal(Vec::<SpaceMember>::new());
    let (loading, set_loading) = signal(true);
    let (new_uid, set_new_uid) = signal(String::new());
    let (new_role, set_new_role) = signal("viewer".to_string());
    let (error, set_error) = signal(None::<String>);
    let space_id_for_fetch = space_id.clone();
    let space_id_for_add = space_id.clone();
    let space_id_for_remove = space_id.clone();
    let space_id_for_change = space_id.clone();

    let fetch_members = move || {
        let api = ApiClient::default();
        let sid = space_id_for_fetch.clone();
        spawn_local(async move {
            if let Ok(m) = api.list_space_members(&sid).await { set_members.set(m) }
            set_loading.set(false);
        });
    };

    Effect::new(move |_| { fetch_members(); });

    let add_member = move |_| {
        let uid = new_uid.get();
        if uid.trim().is_empty() {
            set_error.set(Some("User ID required".to_string()));
            return;
        }
        set_error.set(None);
        let api = ApiClient::default();
        let sid = space_id_for_add.clone();
        let role = new_role.get();
        spawn_local(async move {
            match api.add_space_member(&sid, &AddSpaceMemberRequest { user_id: uid, role: Some(role) }).await {
                Ok(_) => {
                    if let Ok(m) = api.list_space_members(&sid).await { set_members.set(m) }
                    set_new_uid.set(String::new());
                }
                Err(e) => set_error.set(Some(format!("{:?}", e))),
            }
        });
    };

    let remove_member = Callback::new(move |uid: String| {
        let api = ApiClient::default();
        let sid = space_id_for_remove.clone();
        spawn_local(async move {
            let _ = api.remove_space_member(&sid, &uid).await;
            if let Ok(m) = api.list_space_members(&sid).await { set_members.set(m) }
        });
    });

    let change_role = Callback::new(move |(uid, role): (String, String)| {
        let api = ApiClient::default();
        let sid = space_id_for_change.clone();
        spawn_local(async move {
            let _ = api.update_space_member(&sid, &uid, &UpdateSpaceMemberRequest { role }).await;
            if let Ok(m) = api.list_space_members(&sid).await { set_members.set(m) }
        });
    });

    let do_close = move |_| on_close.run(());

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-lg mx-4 p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Manage Members"</h2>

                <Show when=move || error.get().is_some()>
                    <div class="mb-3 p-3 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 text-sm rounded-lg">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="flex gap-2 mb-4">
                    <input type="text"
                        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                        placeholder="User ID"
                        prop:value={move || new_uid.get()}
                        on:input={move |ev| set_new_uid.set(event_target_value(&ev))} />
                    <select class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                        prop:value={move || new_role.get()}
                        on:change={move |ev| set_new_role.set(event_target_value(&ev))}>
                        <option value="viewer">"Viewer"</option>
                        <option value="editor">"Editor"</option>
                        <option value="admin">"Admin"</option>
                    </select>
                    <button class="px-3 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700"
                        on:click=add_member>"Add"</button>
                </div>

                // Members list
                {move || {
                    let is_loading = loading.get();
                    let m = members.get();
                    if is_loading {
                        view! { <div class="text-center py-8 text-gray-400">"Loading..."</div> }.into_any()
                    } else if m.is_empty() {
                        view! { <div class="text-center py-8 text-gray-400">"No members yet"</div> }.into_any()
                    } else {
                        view! {
                            <div class="space-y-2 max-h-64 overflow-y-auto">
                                {m.into_iter().map(|member| {
                                    let rm_uid = member.user_id.clone();
                                    let ch_uid = member.user_id.clone();
                                    let member_role = member.role.clone();
                                    let display = member.display_name.clone()
                                        .or(member.username.clone())
                                        .unwrap_or_else(|| member.user_id.chars().take(8).collect());
                                    let initial = display.chars().next().unwrap_or('?').to_uppercase().to_string();
                                    view! {
                                        <div class="flex items-center justify-between px-3 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                            <div class="flex items-center gap-3 min-w-0">
                                                <div class="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-xs font-medium text-gray-600 dark:text-gray-300">
                                                    {initial}
                                                </div>
                                                <div class="min-w-0">
                                                    <p class="text-sm font-medium text-gray-900 dark:text-white truncate">{display}</p>
                                                    <p class="text-xs text-gray-400">{member_role.clone()}</p>
                                                </div>
                                            </div>
                                            <div class="flex items-center gap-2 flex-shrink-0">
                                                <select class="text-xs px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300"
                                                    prop:value={member_role}
                                                    on:change={move |ev| change_role.run((ch_uid.clone(), event_target_value(&ev)))}>
                                                    <option value="viewer">"Viewer"</option>
                                                    <option value="editor">"Editor"</option>
                                                    <option value="admin">"Admin"</option>
                                                </select>
                                                <button class="p-1 text-gray-400 hover:text-red-500 rounded"
                                                    on:click={move |_| remove_member.run(rm_uid.clone())}>
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                    </svg>
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}

                <div class="flex justify-end mt-4">
                    <button class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        on:click=do_close>"Close"</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Confirm Modal
// ---------------------------------------------------------------------------

#[component]
fn ConfirmModal(
    title: String,
    message: String,
    confirm_label: String,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-sm mx-4 p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">{title}</h2>
                <p class="text-gray-500 dark:text-gray-400 mb-6">{message}</p>
                <div class="flex justify-end gap-3">
                    <button class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                    <button class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
                        on:click={move |_| on_confirm.run(())}>{confirm_label}</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn event_target_value(ev: &leptos::ev::Event) -> String {
    ev.target()
        .and_then(|t| {
            use wasm_bindgen::JsCast;
            t.dyn_into::<web_sys::HtmlInputElement>().ok()
        })
        .map(|i| i.value())
        .unwrap_or_default()
}
