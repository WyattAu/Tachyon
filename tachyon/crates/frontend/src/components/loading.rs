#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(#[prop(default = 8)] size: usize) -> impl IntoView {
    let size_class = match size {
        0..=4 => "w-4 h-4 border-2",
        5..=8 => "w-8 h-8 border-2",
        9..=12 => "w-12 h-12 border-[3px]",
        _ => "w-8 h-8 border-2",
    };

    view! {
        <div class="flex items-center justify-center">
            <div class={format!(
                "{} rounded-full border-gray-300 dark:border-gray-600 border-t-blue-600 dark:border-t-blue-400 animate-spin",
                size_class
            )}></div>
        </div>
    }
}

#[component]
pub fn LoadingPage(#[prop(default = "Loading...".to_string())] message: String) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-20 gap-4">
            <div class="w-10 h-10 border-[3px] border-gray-300 dark:border-gray-600 border-t-blue-600 dark:border-t-blue-400 rounded-full animate-spin"></div>
            <p class="text-gray-500 dark:text-gray-400 text-sm">{message}</p>
        </div>
    }
}

#[component]
pub fn InlineLoading() -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-gray-300 dark:border-gray-600 border-t-blue-600 dark:border-t-blue-400 rounded-full animate-spin"></div>
            <span class="text-sm text-gray-500 dark:text-gray-400">"Loading..."</span>
        </div>
    }
}

#[component]
pub fn ButtonSpinner() -> impl IntoView {
    view! {
        <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
    }
}
