#![allow(dead_code)]
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCursor {
    pub user_id: String,
    pub user_name: String,
    pub color: String,
    pub line: usize,
    pub col: usize,
    pub selection_start: Option<(usize, usize)>,
    pub selection_end: Option<(usize, usize)>,
    pub last_seen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwarenessState {
    pub cursors: Vec<RemoteCursor>,
    pub local_user_id: String,
}

const CURSOR_COLORS: &[&str] = &[
    "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
    "#BB8FCE", "#85C1E9",
];

pub fn get_cursor_color(user_id: &str) -> String {
    let hash = user_id.chars().map(|c| c as usize).sum::<usize>();
    CURSOR_COLORS[hash % CURSOR_COLORS.len()].to_string()
}

fn is_stale(last_seen: f64) -> bool {
    let now = js_sys::Date::now();
    now > 0.0 && last_seen > 0.0 && (now - last_seen) > 30_000.0
}

fn normalize_selection(
    start: (usize, usize),
    end: (usize, usize),
) -> ((usize, usize), (usize, usize)) {
    if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
        (start, end)
    } else {
        (end, start)
    }
}

#[component]
pub fn CollaborativeCursors(
    awareness: RwSignal<AwarenessState>,
    #[prop(default = true)] line_numbers: bool,
    #[prop(default = 8.0)] char_width: f64,
    #[prop(default = 22.0)] line_height: f64,
) -> impl IntoView {
    let gutter_w = if line_numbers { 50.0 } else { 0.0 };

    move || {
        let state = awareness.get();
        let active: Vec<&RemoteCursor> = state
            .cursors
            .iter()
            .filter(|c| c.user_id != state.local_user_id && !is_stale(c.last_seen))
            .collect();

        if active.is_empty() {
            return ().into_any();
        }

        active
            .into_iter()
            .map(|cursor| {
                let color = if cursor.color.is_empty() {
                    get_cursor_color(&cursor.user_id)
                } else {
                    cursor.color.clone()
                };
                let name = cursor.user_name.clone();
                let top = cursor.line as f64 * line_height;
                let left = gutter_w + cursor.col as f64 * char_width;

                let selection = match (cursor.selection_start, cursor.selection_end) {
                    (Some(s), Some(e)) if s != e => {
                        let ((tl, tc), (bl, bc)) = normalize_selection(s, e);
                        let st = tl as f64 * line_height;
                        let sl = gutter_w + tc as f64 * char_width;
                        let sh = (bl - tl + 1) as f64 * line_height;
                        let sw = if tl == bl {
                            (bc - tc) as f64 * char_width
                        } else {
                            0.0
                        };
                        let sc = color.clone();
                        view! {
                            <div
                                class="collab-selection"
                                style:position="absolute"
                                style:top=format!("{st}px")
                                style:left=format!("{sl}px")
                                style:width=format!("{sw}px")
                                style:height=format!("{sh}px")
                                style:background-color=sc
                                style:opacity="0.2"
                                style:pointer-events="none"
                            ></div>
                        }
                        .into_any()
                    }
                    _ => ().into_any(),
                };

                let flag_color = color.clone();
                let cursor_color = color;
                let flag_top = top - 20.0;

                view! {
                    <>
                        {selection}
                        <div
                            class="collab-cursor"
                            style:position="absolute"
                            style:top=format!("{top}px")
                            style:left=format!("{left}px")
                            style:pointer-events="none"
                            style:z-index="10"
                        >
                            <div
                                style:position="absolute"
                                style:top="0"
                                style:left="-1px"
                                style:width="2px"
                                style:height=format!("{line_height}px")
                                style:background-color=cursor_color
                            ></div>
                        </div>
                        <div
                            class="collab-cursor-flag"
                            style:position="absolute"
                            style:top=format!("{flag_top}px")
                            style:left=format!("{left}px")
                            style:pointer-events="none"
                            style:z-index="11"
                        >
                            <div
                                style:background-color=flag_color
                                style:color="#000"
                                style:padding="1px 6px"
                                style:border-radius="3px 3px 3px 0"
                                style:font-size="11px"
                                style:line-height="18px"
                                style:white-space="nowrap"
                                style:user-select="none"
                                style:font-weight="500"
                            >
                                {name}
                            </div>
                        </div>
                    </>
                }
                .into_any()
            })
            .collect::<Vec<_>>()
            .into_any()
    }
}

#[component]
pub fn CollaborationStatusBar(awareness: RwSignal<AwarenessState>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
            </svg>
            {move || {
                let state = awareness.get();
                let count = state
                    .cursors
                    .iter()
                    .filter(|c| c.user_id != state.local_user_id && !is_stale(c.last_seen))
                    .count();
                if count == 0 {
                    view! { <span>"No one else is editing"</span> }.into_any()
                } else if count == 1 {
                    view! { <span class="text-green-600 dark:text-green-400">"1 person editing"</span> }.into_any()
                } else {
                    view! { <span class="text-green-600 dark:text-green-400">{format!("{} people editing", count)}</span> }.into_any()
                }
            }}
        </div>
    }
}
