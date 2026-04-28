#![allow(dead_code)]
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub user_id: String,
    pub user_name: String,
    pub color: String,
    pub is_typing: bool,
}

fn ensure_typing_style() {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if doc.get_element_by_id("collab-typing-style").is_none() {
                if let Ok(style) = doc.create_element("style") {
                    style.set_id("collab-typing-style");
                    style.set_text_content(Some(
                        concat!(
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
                        ),
                    ));
                    let _ = doc.head().and_then(|head| head.append_child(&style).ok());
                }
            }
        }
    }
}

#[component]
pub fn PresenceIndicators(
    users: RwSignal<Vec<PresenceUser>>,
) -> impl IntoView {
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

                        view! {
                            <div class="relative" title=tip>
                                <div
                                    class="w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-medium ring-2 ring-white dark:ring-gray-800"
                                    style=format!("background-color: {bg}")
                                >
                                    {ch}
                                </div>
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
