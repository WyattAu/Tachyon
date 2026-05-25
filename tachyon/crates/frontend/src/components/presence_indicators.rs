#![allow(dead_code)]
use crate::websocket::PresenceUserInfo;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub user_id: String,
    pub user_name: String,
    pub color: String,
    pub is_typing: bool,
    pub is_online: bool,
}

fn ensure_typing_style() {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if doc.get_element_by_id("collab-typing-style").is_none() {
                if let Ok(style) = doc.create_element("style") {
                    style.set_id("collab-typing-style");
                    style.set_text_content(Some(concat!(
                        "@keyframes collab-typing-dot{",
                        "0%,80%,100%{transform:scale(0)}",
                        "40%{transform:scale(1)}",
                        "}",
                        ".collab-typing-dot{",
                        "display:inline-block;width:4px;height:4px;",
                        "border-radius:50%;background-color:currentColor;",
                        "animation:collab-typing-dot 1.4s infinite ease-in-out both",
                        "}",
                        ".collab-typing-dot:nth-child(1){animation-delay:-.32s}",
                        ".collab-typing-dot:nth-child(2){animation-delay:-.16s}",
                        "@keyframes presence-pulse{",
                        "0%,100%{opacity:1}",
                        "50%{opacity:0.5}",
                        "}",
                        ".presence-online-dot{animation:presence-pulse 2s infinite}",
                    )));
                    let _ = doc.head().and_then(|head| head.append_child(&style).ok());
                }
            }
        }
    }
}

#[component]
pub fn PresenceIndicators(users: RwSignal<Vec<PresenceUser>>) -> impl IntoView {
    ensure_typing_style();

    view! {
        <div class="flex items-center gap-1">
            {move || {
                let list = users.get();
                let show = list.len().min(5);
                let extra = list.len().saturating_sub(5);

                let avatars: Vec<_> = list
                    .iter()
                    .take(show)
                    .map(|u| {
                        let ch = u
                            .user_name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        let bg = u.color.clone();
                        let tip = u.user_name.clone();
                        let typing = u.is_typing;
                        let online = u.is_online;

                        view! {
                            <div class="relative" title=tip>
                                <div
                                    class="w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-medium ring-2 ring-white dark:ring-gray-800"
                                    style=format!("background-color: {bg}")
                                >
                                    {ch}
                                </div>
                                <div
                                    class=if online { "absolute bottom-0 right-0 w-2.5 h-2.5 bg-green-500 rounded-full ring-1 ring-white dark:ring-gray-800 presence-online-dot" } else { "absolute bottom-0 right-0 w-2.5 h-2.5 bg-gray-400 rounded-full ring-1 ring-white dark:ring-gray-800" }
                                ></div>
                                {if typing {
                                    view! {
                                        <div class="absolute -bottom-1 -right-1 flex items-center gap-px bg-white dark:bg-gray-800 rounded-full px-0.5">
                                            <span class="collab-typing-dot"></span>
                                            <span class="collab-typing-dot"></span>
                                            <span class="collab-typing-dot"></span>
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    ().into_any()
                                }}
                            </div>
                        }
                    })
                    .collect();

                if extra > 0 {
                    view! {
                        <>
                            {avatars}
                            <div class="w-7 h-7 rounded-full flex items-center justify-center text-xs text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 ring-2 ring-white dark:ring-gray-800">
                                {format!("+{extra}")}
                            </div>
                        </>
                    }
                    .into_any()
                } else {
                    view! { <>{avatars}</> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn PresenceIndicator(
    #[prop(into)] user_name: String,
    #[prop(default = true)] online: bool,
    #[prop(default = false)] typing: bool,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    ensure_typing_style();

    let bg = color.unwrap_or_else(|| "#6b7280".to_string());
    let initial = user_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    view! {
        <div class="inline-flex items-center gap-1.5">
            <div class="relative">
                <div
                    class="w-6 h-6 rounded-full flex items-center justify-center text-white text-[10px] font-medium"
                    style=format!("background-color: {bg}")
                >
                    {initial}
                </div>
                <div class=if online {
                    "absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-green-500 rounded-full ring-1 ring-white presence-online-dot"
                } else {
                    "absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-gray-400 rounded-full ring-1 ring-white"
                }></div>
            </div>
            <span class="text-xs text-gray-700 dark:text-gray-300">{user_name}</span>
            {if typing {
                view! {
                    <span class="flex items-center gap-px text-gray-400">
                        <span class="collab-typing-dot"></span>
                        <span class="collab-typing-dot"></span>
                        <span class="collab-typing-dot"></span>
                    </span>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaboratorStatus {
    Viewing,
    Editing,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub user_id: String,
    pub user_name: String,
    pub status: CollaboratorStatus,
    pub cursor_line: Option<usize>,
    pub color: String,
}

impl From<PresenceUserInfo> for CollaboratorInfo {
    fn from(p: PresenceUserInfo) -> Self {
        let status = if p.selection.is_some() {
            CollaboratorStatus::Editing
        } else if p.cursor_position > 0 {
            CollaboratorStatus::Viewing
        } else {
            CollaboratorStatus::Idle
        };
        Self {
            user_id: p.user_id,
            user_name: p.user_name,
            status,
            cursor_line: Some(p.cursor_position),
            color: p.color.unwrap_or_else(|| "#6b7280".to_string()),
        }
    }
}

#[component]
pub fn CollaboratorList(
    #[prop(into)] collaborators: RwSignal<Vec<CollaboratorInfo>>,
) -> impl IntoView {
    ensure_typing_style();

    view! {
        <div class="space-y-1">
            <h4 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                "Collaborators"
            </h4>
            <div class="divide-y divide-gray-100 dark:divide-gray-700">
                {move || {
                    collaborators.get()
                        .into_iter()
                        .map(|c| {
                            let status_label = match &c.status {
                                CollaboratorStatus::Editing => "editing",
                                CollaboratorStatus::Viewing => "viewing",
                                CollaboratorStatus::Idle => "idle",
                            };
                            let status_class = match &c.status {
                                CollaboratorStatus::Editing => "text-green-600 dark:text-green-400",
                                CollaboratorStatus::Viewing => "text-blue-600 dark:text-blue-400",
                                CollaboratorStatus::Idle => "text-gray-400 dark:text-gray-500",
                            };
                            let dot_class = match &c.status {
                                CollaboratorStatus::Editing => "bg-green-500 presence-online-dot",
                                CollaboratorStatus::Viewing => "bg-blue-500",
                                CollaboratorStatus::Idle => "bg-gray-400",
                            };
                            let initial = c.user_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let bg = c.color.clone();
                            let name = c.user_name.clone();

                            view! {
                                <div class="flex items-center gap-2 py-1.5">
                                    <div class="relative">
                                        <div
                                            class="w-6 h-6 rounded-full flex items-center justify-center text-white text-[10px] font-medium"
                                            style=format!("background-color: {bg}")
                                        >
                                            {initial}
                                        </div>
                                        <div class=format!("absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full ring-1 ring-white dark:ring-gray-800 {}", dot_class)></div>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <p class="text-sm text-gray-900 dark:text-white truncate">{name}</p>
                                    </div>
                                    <span class=format!("text-[10px] font-medium {}", status_class)>
                                        {status_label}
                                    </span>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}
